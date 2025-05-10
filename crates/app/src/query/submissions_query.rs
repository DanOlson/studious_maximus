use super::Query;
use crate::models::{RawDbSubmission, Submission};

#[derive(Debug)]
pub struct SubmissionsQuery;

impl Query for SubmissionsQuery {
    type Value = Vec<Submission>;

    async fn exec(&self, pool: &sqlx::SqlitePool) -> anyhow::Result<Self::Value> {
        let submissions = sqlx::query_as!(
            RawDbSubmission,
            r#"
                select id
                      ,student_id
                      ,assignment_id
                      ,grade
                      ,score
                      ,submitted_at
                      ,graded_at
                      ,posted_at
                      ,late
                      ,missing
                from submissions
                order by student_id, assignment_id
            "#
        )
        .fetch_all(pool)
        .await?
        .iter()
        .map(Submission::from)
        .collect();

        Ok(submissions)
    }
}
