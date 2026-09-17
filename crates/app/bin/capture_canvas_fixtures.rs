use anyhow::{Context, bail};
use reqwest::{
    Client, Url,
    header::{HeaderMap, HeaderValue},
};
use serde_json::{Map, Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let base_url = env::var("CANVAS_BASE_URL").context("CANVAS_BASE_URL is required")?;
    let token = env::var("CANVAS_TOKEN").context("CANVAS_TOKEN is required")?;
    let out_dir = PathBuf::from(
        env::var("CANVAS_FIXTURE_DIR")
            .unwrap_or_else(|_| "plans/canvas-data-sources/fixtures/sanitized".to_string()),
    );
    let start_date =
        env::var("CANVAS_FIXTURE_START_DATE").unwrap_or_else(|_| "2025-02-03".to_string());
    let end_date = env::var("CANVAS_FIXTURE_END_DATE").unwrap_or_else(|_| "2025-02-10".to_string());

    fs::create_dir_all(&out_dir)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {token}"))?,
    );
    let client = Client::builder().default_headers(headers).build()?;

    let mut sanitizer = Sanitizer::default();

    let observees = fetch_json(&client, &base_url, "/api/v1/users/self/observees").await?;
    write_fixture(&out_dir, "observees.json", &observees, &mut sanitizer)?;

    let mut manifest = json!({
        "captured_from": sanitize_url(&base_url),
        "date_range": { "start_date": start_date, "end_date": end_date },
        "files": ["observees.json"],
        "notes": [
            "All scalar identifiers and obvious PII-bearing strings were sanitized by the capture tool.",
            "Review front page HTML and linked URLs manually before committing tenant-derived fixtures."
        ]
    });

    let mut course_ids = BTreeSet::new();
    let students = observees.as_array().cloned().unwrap_or_default();
    for (student_index, student) in students.iter().enumerate() {
        let Some(student_id) = student.get("id").and_then(Value::as_i64) else {
            continue;
        };
        let student_label = format!("student-{}", student_index + 1);

        let courses_path = format!(
            "/api/v1/users/{student_id}/courses?enrollment_state=active&include[]=term&per_page=100"
        );
        let courses = fetch_json(&client, &base_url, &courses_path).await?;
        let courses_name = format!("{student_label}-active-courses.json");
        write_fixture(&out_dir, &courses_name, &courses, &mut sanitizer)?;
        push_manifest_file(&mut manifest, &courses_name);

        if let Some(items) = courses.as_array() {
            for course in items {
                if let Some(id) = course.get("id").and_then(Value::as_i64) {
                    course_ids.insert(id);
                }
            }
        }

        let planner_path = format!(
            "/api/v1/planner/items?observed_user_id={student_id}&start_date={start_date}&end_date={end_date}&per_page=100"
        );
        let planner = fetch_json(&client, &base_url, &planner_path).await?;
        let planner_name = format!("{student_label}-planner-items.json");
        write_fixture(&out_dir, &planner_name, &planner, &mut sanitizer)?;
        push_manifest_file(&mut manifest, &planner_name);
    }

    for (course_index, course_id) in course_ids.into_iter().enumerate() {
        let course_label = format!("course-{}", course_index + 1);
        let events_path = format!(
            "/api/v1/calendar_events?type=event&all_events=true&context_codes[]=course_{course_id}&start_date={start_date}&end_date={end_date}&per_page=100"
        );
        let events = fetch_json(&client, &base_url, &events_path).await?;
        let events_name = format!("{course_label}-calendar-events.json");
        write_fixture(&out_dir, &events_name, &events, &mut sanitizer)?;
        push_manifest_file(&mut manifest, &events_name);

        let assignments_path = format!(
            "/api/v1/courses/{course_id}/assignments?include[]=overrides&include[]=submission&per_page=100"
        );
        let assignments = fetch_json(&client, &base_url, &assignments_path).await?;
        let assignments_name = format!("{course_label}-assignments.json");
        write_fixture(&out_dir, &assignments_name, &assignments, &mut sanitizer)?;
        push_manifest_file(&mut manifest, &assignments_name);

        let front_page_path = format!("/api/v1/courses/{course_id}/front_page");
        let front_page = fetch_json(&client, &base_url, &front_page_path).await?;
        let front_page_name = format!("{course_label}-front-page.json");
        write_fixture(&out_dir, &front_page_name, &front_page, &mut sanitizer)?;
        push_manifest_file(&mut manifest, &front_page_name);

        for (student_index, student) in students.iter().enumerate() {
            let Some(student_id) = student.get("id").and_then(Value::as_i64) else {
                continue;
            };
            let submissions_path = format!(
                "/api/v1/courses/{course_id}/students/submissions?student_ids[]={student_id}&per_page=100"
            );
            let submissions = fetch_json(&client, &base_url, &submissions_path).await?;
            let submissions_name = format!(
                "student-{}-submissions-{course_label}.json",
                student_index + 1
            );
            write_fixture(&out_dir, &submissions_name, &submissions, &mut sanitizer)?;
            push_manifest_file(&mut manifest, &submissions_name);
        }
    }

    write_fixture(&out_dir, "manifest.json", &manifest, &mut sanitizer)?;
    Ok(())
}

async fn fetch_json(client: &Client, base_url: &str, path: &str) -> anyhow::Result<Value> {
    let mut pages = Vec::new();
    let mut next_url = Some(Url::parse(base_url)?.join(path)?);

    while let Some(url) = next_url {
        let response = client.get(url.clone()).send().await?;
        if !response.status().is_success() {
            bail!("Canvas request failed: {} {}", response.status(), url);
        }
        let headers = response.headers().clone();
        pages.push(response.json::<Value>().await?);
        next_url = parse_next_link(headers.get("link"));
    }

    let merged = if pages.len() == 1 {
        pages.pop().expect("one page")
    } else {
        Value::Array(
            pages
                .into_iter()
                .flat_map(|page| match page {
                    Value::Array(items) => items,
                    other => vec![other],
                })
                .collect(),
        )
    };
    Ok(merged)
}

fn write_fixture(
    out_dir: &Path,
    file_name: &str,
    value: &Value,
    sanitizer: &mut Sanitizer,
) -> anyhow::Result<()> {
    fs::write(
        out_dir.join(file_name),
        serde_json::to_string_pretty(&sanitizer.sanitize(value.clone()))?,
    )?;
    Ok(())
}

fn push_manifest_file(manifest: &mut Value, file_name: &str) {
    if let Some(files) = manifest.get_mut("files").and_then(Value::as_array_mut) {
        files.push(Value::String(file_name.to_string()));
    }
}

#[derive(Default)]
struct Sanitizer {
    ids: BTreeMap<String, i64>,
    next_id: i64,
}

impl Sanitizer {
    fn sanitize(&mut self, value: Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.into_iter()
                    .map(|(key, value)| {
                        let sanitized = match key.as_str() {
                            "id" | "user_id" | "course_id" | "assignment_id" | "student_id"
                            | "grader_id" | "author_id" | "page_id" => self.sanitize_id(value),
                            "student_ids" => self.sanitize(value),
                            "name" | "short_name" | "sortable_name" | "title" => {
                                Value::String(format!("Sample {}", key.replace('_', " ")))
                            }
                            "html_url" | "url" | "api_url" => value
                                .as_str()
                                .map(sanitize_url)
                                .map(Value::String)
                                .unwrap_or(Value::Null),
                            "body" | "description" | "message" => self.sanitize_htmlish(value),
                            _ => self.sanitize(value),
                        };
                        (key, sanitized)
                    })
                    .collect::<Map<_, _>>(),
            ),
            Value::Array(items) => {
                Value::Array(items.into_iter().map(|item| self.sanitize(item)).collect())
            }
            other => other,
        }
    }

    fn sanitize_id(&mut self, value: Value) -> Value {
        match value {
            Value::Number(number) => {
                let key = number.to_string();
                let next = self.next_id;
                let sanitized = self.ids.entry(key).or_insert_with(|| {
                    self.next_id += 1;
                    10_000 + next
                });
                Value::Number((*sanitized).into())
            }
            Value::String(raw) => {
                let next = self.next_id;
                let sanitized = self.ids.entry(raw).or_insert_with(|| {
                    self.next_id += 1;
                    10_000 + next
                });
                Value::String(format!("sample-id-{sanitized}"))
            }
            other => other,
        }
    }

    fn sanitize_htmlish(&mut self, value: Value) -> Value {
        match value {
            Value::String(text) => Value::String(
                text.replace(|c: char| c.is_ascii_digit(), "#")
                    .replace('@', "[at]"),
            ),
            other => self.sanitize(other),
        }
    }
}

fn sanitize_url(raw: &str) -> String {
    let Ok(mut url) = Url::parse(raw) else {
        return "https://canvas.example.invalid/redacted".to_string();
    };
    url.set_scheme("https").ok();
    url.set_host(Some("canvas.example.invalid")).ok();
    url.set_query(None);
    url.to_string()
}

fn parse_next_link(link_header: Option<&reqwest::header::HeaderValue>) -> Option<Url> {
    let header_value = link_header?.to_str().ok()?;
    for part in header_value.split(',') {
        let sections: Vec<&str> = part.trim().split(';').collect();
        if sections.len() < 2 {
            continue;
        }
        if sections[1].trim() == r#"rel="next""# {
            let url_str = sections[0]
                .trim()
                .trim_start_matches('<')
                .trim_end_matches('>');
            return Url::parse(url_str).ok();
        }
    }
    None
}
