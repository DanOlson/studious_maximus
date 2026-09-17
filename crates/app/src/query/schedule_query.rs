use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};

use crate::models::{ScheduleItem, ScheduleQueryFilters, SyncHealth};

use super::Query;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleStatusFilter {
    Overdue,
    Missing,
    Completed,
    RecentlyGraded,
}

impl ScheduleStatusFilter {
    fn as_str(self) -> &'static str {
        match self {
            Self::Overdue => "overdue",
            Self::Missing => "missing",
            Self::Completed => "completed",
            Self::RecentlyGraded => "recently_graded",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScheduleItemsQuery {
    pub filters: ScheduleQueryFilters,
    pub statuses: Vec<ScheduleStatusFilter>,
    pub now_utc: String,
}

impl Query for ScheduleItemsQuery {
    type Value = Vec<ScheduleItem>;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            with schedule as (
                select si.id,
                       si.student_id,
                       st.name as student_name,
                       si.course_id,
                       c.name as course_name,
                       si.assignment_id,
                       si.kind,
                       si.title,
                       si.starts_at_utc,
                       si.ends_at_utc,
                       si.due_at_utc,
                       si.all_day_date,
                       si.school_timezone,
                       case
                           when sub.graded_at_utc is not null then 'recently_graded'
                           when sub.submitted_at_utc is not null or sub.grade is not null or sub.score is not null then 'completed'
                           when sub.missing = 1 then 'missing'
                           when si.due_at_utc is not null and si.due_at_utc < "#,
        );
        builder.push_bind(&self.now_utc);
        builder.push(
            r#" then 'overdue'
                           else si.status
                       end as status,
                       si.source,
                       si.source_canvas_id,
                       si.confidence,
                       si.evidence,
                       si.provenance_json,
                       si.active
                from schedule_items si
                left join students st on st.id = si.student_id
                left join courses c on c.id = si.course_id
                left join submissions sub on sub.student_id = si.student_id and sub.assignment_id = si.assignment_id
                where si.active = 1

                union all

                select -(d.student_id * 1000000000 + a.id) as id,
                       d.student_id,
                       st.name as student_name,
                       a.course_id,
                       c.name as course_name,
                       a.id as assignment_id,
                       'assignment' as kind,
                       a.name as title,
                       null as starts_at_utc,
                       null as ends_at_utc,
                       d.due_at_utc,
                       d.all_day_date,
                       'America/Chicago' as school_timezone,
                       case
                           when sub.graded_at_utc is not null then 'recently_graded'
                           when sub.submitted_at_utc is not null or sub.grade is not null or sub.score is not null then 'completed'
                           when sub.missing = 1 then 'missing'
                           when d.due_at_utc is not null and d.due_at_utc < "#,
        );
        builder.push_bind(&self.now_utc);
        builder.push(
            r#" then 'overdue'
                           else 'pending'
                       end as status,
                       d.source,
                       a.canvas_assignment_id as source_canvas_id,
                       1.0 as confidence,
                       null as evidence,
                       json_object('assignment_id', a.id, 'student_assignment_dates', d.source) as provenance_json,
                       1 as active
                from student_assignment_dates d
                join assignments a on a.id = d.assignment_id
                join students st on st.id = d.student_id
                join courses c on c.id = a.course_id
                left join submissions sub on sub.student_id = d.student_id and sub.assignment_id = d.assignment_id
                where not exists (
                    select 1 from schedule_items si
                    where si.active = 1 and si.student_id = d.student_id and si.assignment_id = d.assignment_id
                )
            )
            select * from schedule
            where 1 = 1
            "#,
        );

        if let Some(student_id) = self.filters.student_id {
            builder.push(" and student_id = ").push_bind(student_id);
        }
        if let Some(start) = &self.filters.start {
            builder
                .push(" and coalesce(due_at_utc, starts_at_utc, all_day_date) >= ")
                .push_bind(start);
        }
        if let Some(end) = &self.filters.end {
            builder
                .push(" and coalesce(due_at_utc, starts_at_utc, all_day_date) < ")
                .push_bind(end);
        }
        if !self.statuses.is_empty() {
            builder.push(" and status in (");
            let mut separated = builder.separated(", ");
            for status in &self.statuses {
                separated.push_bind(status.as_str());
            }
            separated.push_unseparated(")");
        }
        builder.push(" order by coalesce(due_at_utc, starts_at_utc, all_day_date), student_name, course_name, title");

        let rows = builder
            .build()
            .fetch_all(pool)
            .await?
            .iter()
            .map(ScheduleItem::from_row)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rows)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SyncHealthQuery;

impl Query for SyncHealthQuery {
    type Value = SyncHealth;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        let health = sqlx::query_as::<_, SyncHealth>(
            r#"
            select id,
                   sync_kind,
                   status,
                   started_at_utc,
                   finished_at_utc,
                   students_seen,
                   courses_seen,
                   assignments_seen,
                   submissions_seen,
                   schedule_items_seen,
                   error
            from sync_runs
            order by started_at_utc desc, id desc
            limit 1
            "#,
        )
        .fetch_optional(pool)
        .await?
        .unwrap_or_default();

        Ok(health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn filters_assignment_fallback_by_range_and_calculates_status(pool: SqlitePool) {
        seed_assignment(&pool, None, 0).await;

        let items = ScheduleItemsQuery {
            filters: ScheduleQueryFilters {
                student_id: Some(1),
                start: Some("2025-02-01".to_string()),
                end: Some("2025-02-10".to_string()),
            },
            statuses: vec![ScheduleStatusFilter::Overdue],
            now_utc: "2025-02-05T00:00:00Z".to_string(),
        }
        .exec(&pool)
        .await
        .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Essay");
        assert_eq!(items[0].status, "overdue");
        assert_eq!(items[0].student_name.as_deref(), Some("Alice"));
    }

    #[sqlx::test]
    async fn completed_status_comes_from_submission(pool: SqlitePool) {
        seed_assignment(&pool, Some("2025-02-03T12:00:00Z"), 0).await;

        let items = ScheduleItemsQuery {
            filters: ScheduleQueryFilters::default(),
            statuses: vec![ScheduleStatusFilter::Completed],
            now_utc: "2025-02-05T00:00:00Z".to_string(),
        }
        .exec(&pool)
        .await
        .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, "completed");
    }

    #[sqlx::test]
    async fn canonical_schedule_item_wins_over_assignment_fallback(pool: SqlitePool) {
        seed_assignment(&pool, None, 0).await;
        sqlx::query("insert into schedule_items (id, student_id, course_id, assignment_id, kind, title, due_at_utc, status, source, confidence, provenance_json) values (99, 1, 10, 100, 'assignment', 'Planner Essay', '2025-02-04T20:00:00Z', 'pending', 'planner', 0.9, '{}')")
            .execute(&pool)
            .await
            .unwrap();

        let items = ScheduleItemsQuery {
            filters: ScheduleQueryFilters::default(),
            statuses: vec![],
            now_utc: "2025-02-01T00:00:00Z".to_string(),
        }
        .exec(&pool)
        .await
        .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, 99);
        assert_eq!(items[0].title, "Planner Essay");
    }

    #[sqlx::test]
    async fn sync_health_returns_latest_run(pool: SqlitePool) {
        sqlx::query("insert into sync_runs (id, sync_kind, status, started_at_utc, finished_at_utc, students_seen) values (1, 'canvas_full', 'failed', '2025-01-01T00:00:00Z', '2025-01-01T00:01:00Z', 0), (2, 'canvas_full', 'succeeded', '2025-01-02T00:00:00Z', '2025-01-02T00:01:00Z', 2)")
            .execute(&pool)
            .await
            .unwrap();

        let health = SyncHealthQuery.exec(&pool).await.unwrap();

        assert_eq!(health.id, Some(2));
        assert_eq!(health.status.as_deref(), Some("succeeded"));
        assert_eq!(health.students_seen, 2);
    }

    async fn seed_assignment(pool: &SqlitePool, submitted_at: Option<&str>, missing: i64) {
        sqlx::query("insert into students (id, canvas_user_id, name) values (1, 1, 'Alice')")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query(
            "insert into courses (id, canvas_course_id, name) values (10, 10, 'Language Arts')",
        )
        .execute(pool)
        .await
        .unwrap();
        sqlx::query("insert into student_course_enrollments (student_id, course_id, enrollment_status) values (1, 10, 'active')")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("insert into assignments (id, canvas_assignment_id, course_id, name) values (100, 100, 10, 'Essay')")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("insert into student_assignment_dates (student_id, assignment_id, due_at_utc, source) values (1, 100, '2025-02-04T20:00:00Z', 'assignment')")
            .execute(pool)
            .await
            .unwrap();
        if submitted_at.is_some() || missing == 1 {
            sqlx::query("insert into submissions (id, canvas_submission_id, student_id, assignment_id, submitted_at_utc, missing) values (500, 500, 1, 100, ?, ?)")
                .bind(submitted_at)
                .bind(missing)
                .execute(pool)
                .await
                .unwrap();
        }
    }
}
