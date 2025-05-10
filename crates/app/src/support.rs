//! Test setup and support

use sqlx::SqlitePool;

use crate::models::{Assignment, RawDbSubmission, Submission};

pub async fn all_assignments(pool: &SqlitePool) -> Vec<Assignment> {
    sqlx::query_as!(Assignment, "select * from assignments order by id asc")
        .fetch_all(pool)
        .await
        .unwrap()
}

pub async fn all_submissions(pool: &SqlitePool) -> Vec<Submission> {
    sqlx::query_as!(RawDbSubmission, "select * from submissions order by id asc")
        .fetch_all(pool)
        .await
        .unwrap()
        .iter()
        .map(Submission::from)
        .collect()
}
