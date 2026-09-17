use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};

use app::{
    AppReadonly,
    models::{ScheduleItem, ScheduleQueryFilters, SyncHealth},
    query::ScheduleStatusFilter,
};
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use serde::Deserialize;

#[derive(Clone)]
struct ServerState {
    app: Arc<AppReadonly>,
}

#[derive(Debug, Default, Deserialize)]
struct ScheduleParams {
    student_id: Option<i64>,
    start: Option<String>,
    end: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let app = Arc::new(AppReadonly::from_env().await?);
    let router = create_router(app);
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    let addr: SocketAddr = listener.local_addr()?;
    tracing::info!(%addr, "listening");

    axum::serve(listener, router).await?;
    Ok(())
}

fn create_router(app: Arc<AppReadonly>) -> Router {
    let state = ServerState { app };
    Router::new()
        .route("/", get(week_view))
        .route("/items/{id}", get(item_details))
        .route("/api/week", get(api_week))
        .route("/api/upcoming", get(api_upcoming))
        .route("/api/overdue", get(api_overdue))
        .route("/api/missing", get(api_missing))
        .route("/api/completed", get(api_completed))
        .route("/api/recently-graded", get(api_recently_graded))
        .route("/api/health", get(api_health))
        .with_state(state)
}

async fn week_view(
    State(state): State<ServerState>,
    Query(params): Query<ScheduleParams>,
) -> Result<Html<String>, AppError> {
    let (start, end) = school_week_range(params.start.clone(), params.end.clone());
    let filters = ScheduleQueryFilters {
        student_id: params.student_id,
        start: Some(start),
        end: Some(end),
    };
    let (items, health) = tokio::try_join!(
        state.app.get_schedule_items(filters, vec![]),
        state.app.get_sync_health()
    )?;

    Ok(Html(render_week(&items, &health)))
}

async fn item_details(
    State(state): State<ServerState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Html<String>, AppError> {
    let items = state
        .app
        .get_schedule_items(ScheduleQueryFilters::default(), vec![])
        .await?;
    let item = items.into_iter().find(|item| item.id == id);

    Ok(Html(render_item_details(item.as_ref())))
}

async fn api_week(
    State(state): State<ServerState>,
    Query(params): Query<ScheduleParams>,
) -> Result<axum::Json<Vec<ScheduleItem>>, AppError> {
    let (start, end) = school_week_range(params.start, params.end);
    api_items(state, params.student_id, Some(start), Some(end), vec![]).await
}

async fn api_upcoming(
    State(state): State<ServerState>,
    Query(params): Query<ScheduleParams>,
) -> Result<axum::Json<Vec<ScheduleItem>>, AppError> {
    api_items(state, params.student_id, params.start, params.end, vec![]).await
}

async fn api_overdue(
    State(state): State<ServerState>,
    Query(params): Query<ScheduleParams>,
) -> Result<axum::Json<Vec<ScheduleItem>>, AppError> {
    api_items(
        state,
        params.student_id,
        params.start,
        params.end,
        vec![ScheduleStatusFilter::Overdue],
    )
    .await
}

async fn api_missing(
    State(state): State<ServerState>,
    Query(params): Query<ScheduleParams>,
) -> Result<axum::Json<Vec<ScheduleItem>>, AppError> {
    api_items(
        state,
        params.student_id,
        params.start,
        params.end,
        vec![ScheduleStatusFilter::Missing],
    )
    .await
}

async fn api_completed(
    State(state): State<ServerState>,
    Query(params): Query<ScheduleParams>,
) -> Result<axum::Json<Vec<ScheduleItem>>, AppError> {
    api_items(
        state,
        params.student_id,
        params.start,
        params.end,
        vec![ScheduleStatusFilter::Completed],
    )
    .await
}

async fn api_recently_graded(
    State(state): State<ServerState>,
    Query(params): Query<ScheduleParams>,
) -> Result<axum::Json<Vec<ScheduleItem>>, AppError> {
    api_items(
        state,
        params.student_id,
        params.start,
        params.end,
        vec![ScheduleStatusFilter::RecentlyGraded],
    )
    .await
}

async fn api_health(State(state): State<ServerState>) -> Result<axum::Json<SyncHealth>, AppError> {
    Ok(axum::Json(state.app.get_sync_health().await?))
}

async fn api_items(
    state: ServerState,
    student_id: Option<i64>,
    start: Option<String>,
    end: Option<String>,
    statuses: Vec<ScheduleStatusFilter>,
) -> Result<axum::Json<Vec<ScheduleItem>>, AppError> {
    Ok(axum::Json(
        state
            .app
            .get_schedule_items(
                ScheduleQueryFilters {
                    student_id,
                    start,
                    end,
                },
                statuses,
            )
            .await?,
    ))
}

fn school_week_range(start: Option<String>, end: Option<String>) -> (String, String) {
    let start_date = start
        .and_then(|date| NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok())
        .unwrap_or_else(|| {
            let today = Utc::now().date_naive();
            today - Duration::days(today.weekday().num_days_from_monday() as i64)
        });
    let end_date = end
        .and_then(|date| NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok())
        .unwrap_or_else(|| start_date + Duration::days(7));

    (start_date.to_string(), end_date.to_string())
}

fn render_week(items: &[ScheduleItem], health: &SyncHealth) -> String {
    let mut by_day: BTreeMap<String, Vec<&ScheduleItem>> = BTreeMap::new();
    for item in items {
        by_day.entry(item_day(item)).or_default().push(item);
    }

    let freshness = match (&health.status, &health.finished_at_utc, &health.error) {
        (Some(status), finished, Some(error)) => format!(
            "Last sync: {status} at {} ({error})",
            finished.as_deref().unwrap_or("unknown")
        ),
        (Some(status), finished, None) => format!(
            "Last sync: {status} at {}",
            finished.as_deref().unwrap_or("in progress")
        ),
        _ => "No sync runs recorded".to_string(),
    };

    let mut body = String::new();
    body.push_str("<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Studious</title><style>");
    body.push_str(STYLES);
    body.push_str("</style></head><body><main><header><h1>School week</h1><p class=\"freshness\">");
    body.push_str(&escape(&freshness));
    body.push_str("</p></header>");

    if by_day.is_empty() {
        body.push_str("<section class=\"empty\">No schedule items found for this range.</section>");
    }

    for (day, day_items) in by_day {
        body.push_str("<section class=\"day\"><h2>");
        body.push_str(&escape(&day));
        body.push_str("</h2><div class=\"cards\">");
        for item in day_items {
            body.push_str(&render_card(item));
        }
        body.push_str("</div></section>");
    }

    body.push_str("</main></body></html>");
    body
}

fn render_card(item: &ScheduleItem) -> String {
    format!(
        "<article class=\"card status-{status}\"><h3><a href=\"/items/{id}\">{title}</a></h3><dl><dt>Student</dt><dd>{student}</dd><dt>Course</dt><dd>{course}</dd><dt>Kind</dt><dd>{kind}</dd><dt>Status</dt><dd>{status}</dd><dt>Source</dt><dd>{source}</dd><dt>Confidence</dt><dd>{confidence}</dd></dl></article>",
        id = item.id,
        title = escape(&item.title),
        student = escape(item.student_name.as_deref().unwrap_or("All students")),
        course = escape(item.course_name.as_deref().unwrap_or("No course")),
        kind = escape(&item.kind),
        status = escape(&item.status),
        source = escape(&item.source),
        confidence = item
            .confidence
            .map(|c| format!("{:.0}%", c * 100.0))
            .unwrap_or_else(|| "unknown".to_string()),
    )
}

fn render_item_details(item: Option<&ScheduleItem>) -> String {
    match item {
        Some(item) => format!(
            "<!doctype html><html><head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{title}</title><style>{styles}</style></head><body><main><a href=\"/\">← Week</a><h1>{title}</h1><pre>{details}</pre></main></body></html>",
            title = escape(&item.title),
            styles = STYLES,
            details = escape(&format!("{:#?}", item)),
        ),
        None => "<!doctype html><html><body><main><a href=\"/\">← Week</a><h1>Item not found</h1></main></body></html>".to_string(),
    }
}

fn item_day(item: &ScheduleItem) -> String {
    item.all_day_date
        .clone()
        .or_else(|| {
            item.due_at_utc
                .as_ref()
                .map(|date| date.chars().take(10).collect())
        })
        .or_else(|| {
            item.starts_at_utc
                .as_ref()
                .map(|date| date.chars().take(10).collect())
        })
        .unwrap_or_else(|| "Undated".to_string())
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[derive(Debug)]
struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(error = %self.0, "request failed");
        (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
    }
}

const STYLES: &str = r#"
:root { color-scheme: light dark; font-family: system-ui, -apple-system, Segoe UI, sans-serif; }
body { margin: 0; background: Canvas; color: CanvasText; }
main { width: min(72rem, calc(100% - 2rem)); margin: 0 auto; padding: 1rem 0 3rem; }
header { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between; gap: .75rem; }
h1 { margin-bottom: .25rem; }
.freshness { color: #64748b; }
.day { margin-top: 1.5rem; }
.cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(17rem, 1fr)); gap: 1rem; }
.card { border: 1px solid #cbd5e1; border-left: .4rem solid #64748b; border-radius: .75rem; padding: 1rem; background: color-mix(in srgb, Canvas 94%, CanvasText 6%); }
.status-overdue, .status-missing { border-left-color: #dc2626; }
.status-completed, .status-recently_graded { border-left-color: #16a34a; }
.card h3 { margin: 0 0 .75rem; }
.card a { color: inherit; }
dl { display: grid; grid-template-columns: max-content 1fr; gap: .35rem .75rem; margin: 0; }
dt { font-weight: 700; color: #64748b; }
dd { margin: 0; }
.empty { border: 1px dashed #94a3b8; border-radius: .75rem; padding: 2rem; text-align: center; }
pre { white-space: pre-wrap; overflow-wrap: anywhere; }
@media (max-width: 40rem) { main { width: calc(100% - 1rem); } .cards { grid-template-columns: 1fr; } dl { grid-template-columns: 1fr; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn item(title: &str, day: &str) -> ScheduleItem {
        ScheduleItem {
            id: 1,
            student_id: Some(1),
            student_name: Some("Alice".to_string()),
            course_id: Some(2),
            course_name: Some("Math".to_string()),
            assignment_id: Some(3),
            kind: "assignment".to_string(),
            title: title.to_string(),
            starts_at_utc: None,
            ends_at_utc: None,
            due_at_utc: Some(format!("{day}T12:00:00Z")),
            all_day_date: None,
            school_timezone: "America/Chicago".to_string(),
            status: "overdue".to_string(),
            source: "assignment".to_string(),
            source_canvas_id: Some(3),
            confidence: Some(1.0),
            evidence: None,
            provenance_json: "{}".to_string(),
            active: 1,
        }
    }

    #[test]
    fn week_renderer_groups_by_day_and_escapes_content() {
        let html = render_week(&[item("<Essay>", "2025-02-04")], &SyncHealth::default());

        assert!(html.contains("2025-02-04"));
        assert!(html.contains("&lt;Essay&gt;"));
        assert!(html.contains("status-overdue"));
        assert!(html.contains("Alice"));
    }

    #[test]
    fn explicit_week_range_is_preserved() {
        let (start, end) = school_week_range(Some("2025-02-03".to_string()), None);

        assert_eq!(start, "2025-02-03");
        assert_eq!(end, "2025-02-10");
    }
}
