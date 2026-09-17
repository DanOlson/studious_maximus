//! Test setup and support

use sqlx::SqlitePool;

use crate::models::{Assignment, RawDbSubmission, Submission};

pub async fn all_assignments(pool: &SqlitePool) -> Vec<Assignment> {
    sqlx::query_as::<_, Assignment>(
        r#"
        select a.id,
               d.student_id,
               a.course_id,
               a.name,
               d.due_at_utc as due_at,
               a.points_possible,
               a.grading_type
        from assignments a
        join student_assignment_dates d on d.assignment_id = a.id
        order by a.id asc, d.student_id asc
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

pub async fn all_submissions(pool: &SqlitePool) -> Vec<Submission> {
    sqlx::query_as::<_, RawDbSubmission>(
        r#"
        select id,
               student_id,
               assignment_id,
               grade,
               score,
               submitted_at_utc as submitted_at,
               graded_at_utc as graded_at,
               posted_at_utc as posted_at,
               late,
               missing
        from submissions
        order by id asc
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap()
    .iter()
    .map(Submission::from)
    .collect()
}
