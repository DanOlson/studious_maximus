use crate::models::Assignment;

use super::Query;

#[derive(Debug)]
pub struct AssignmentsQuery;

impl Query for AssignmentsQuery {
    type Value = Vec<Assignment>;

    async fn exec(&self, pool: &sqlx::SqlitePool) -> anyhow::Result<Self::Value> {
        let assignments = sqlx::query_as!(
            Assignment,
            r#"
            select id, student_id, course_id, name, due_at
            from assignments
            order by due_at desc nulls last"#
        )
        .fetch_all(pool)
        .await?;

        Ok(assignments)
    }
}
