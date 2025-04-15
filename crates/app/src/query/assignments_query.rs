use crate::models::Assignment;

use super::Query;

#[derive(Debug)]
pub struct AssignmentsQuery {
    pub due_on_or_after: chrono::NaiveDate,
}

impl Query for AssignmentsQuery {
    type Value = Vec<Assignment>;

    async fn exec(&self, pool: &sqlx::SqlitePool) -> anyhow::Result<Self::Value> {
        let due = self.due_on_or_after.format("%Y-%m-%d").to_string();
        let assignments = sqlx::query_as!(
            Assignment,
            r#"
            select id, student_id, course_id, name, due_at
            from assignments
            where due_at >= ?
            order by due_at desc nulls last"#,
            due
        )
        .fetch_all(pool)
        .await?;

        Ok(assignments)
    }
}
