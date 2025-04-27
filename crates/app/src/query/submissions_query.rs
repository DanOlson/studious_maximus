use super::Query;
use crate::models::Submission;
use serde::Deserialize;

#[derive(Debug)]
pub struct SubmissionsQuery;

#[derive(Deserialize)]
pub struct RawDbSubmission {
    pub id: i64,
    pub student_id: i64,
    pub assignment_id: i64,
    pub grade: Option<String>,
    pub score: Option<f64>,
    pub submitted_at: Option<String>,
    pub graded_at: Option<String>,
    pub posted_at: Option<String>,
    pub late: i64,
    pub missing: i64,
}

impl From<&RawDbSubmission> for Submission {
    fn from(value: &RawDbSubmission) -> Self {
        Submission {
            id: value.id,
            student_id: value.student_id,
            assignment_id: value.assignment_id,
            grade: value.grade.clone(),
            score: value.score,
            submitted_at: value.submitted_at.clone(),
            graded_at: value.graded_at.clone(),
            posted_at: value.posted_at.clone(),
            late: value.late == 1,
            missing: value.missing == 1,
        }
    }
}
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
