use sqlx::{QueryBuilder, SqlitePool};

use crate::models::Submission;

use super::Query;

#[derive(Debug)]
pub struct UpdateSubmissions {
    pub submissions: Vec<Submission>,
}

impl Query for UpdateSubmissions {
    type Value = ();

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        if self.submissions.is_empty() {
            return Ok(());
        }

        QueryBuilder::new(
            r#"
            insert into submissions (
                id,
                student_id,
                assignment_id,
                grade,
                score,
                submitted_at,
                graded_at,
                posted_at,
                late,
                missing
            )
            "#,
        )
        .push_values(self.submissions.iter(), |mut bld, sub| {
            bld.push_bind(sub.id);
            bld.push_bind(sub.student_id);
            bld.push_bind(sub.assignment_id);
            bld.push_bind(&sub.grade);
            bld.push_bind(sub.score);
            bld.push_bind(&sub.submitted_at);
            bld.push_bind(&sub.graded_at);
            bld.push_bind(&sub.posted_at);
            bld.push_bind(sub.late);
            bld.push_bind(sub.missing);
        })
        .push(
            r#"
            on conflict(id) do update
            set grade=excluded.grade,
                score=excluded.score,
                submitted_at=excluded.submitted_at,
                graded_at=excluded.graded_at,
                posted_at=excluded.posted_at,
                late=excluded.late,
                missing=excluded.missing
            "#,
        )
        .build()
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::support::all_submissions;

    use super::*;

    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn insert_new_submissions(pool: SqlitePool) {
        let submissions = vec![
            Submission {
                id: 1,
                student_id: 11,
                assignment_id: 111,
                grade: None,
                score: None,
                submitted_at: None,
                graded_at: None,
                posted_at: None,
                late: false,
                missing: false,
            },
            Submission {
                id: 2,
                student_id: 11,
                assignment_id: 123,
                grade: Some("A+".to_string()),
                score: Some(100.0),
                submitted_at: Some("2025-05-01".to_string()),
                graded_at: Some("2025-05-03".to_string()),
                posted_at: Some("2025-05-03".to_string()),
                late: false,
                missing: false,
            },
        ];
        let upsert = UpdateSubmissions { submissions };
        upsert.exec(&pool).await.unwrap();

        let submissions = all_submissions(&pool).await;

        assert_eq!(submissions.len(), 2);
    }

    #[sqlx::test(fixtures("../../fixtures/submissions.sql"))]
    async fn update_existing_submissions(pool: SqlitePool) {
        let submissions = vec![
            Submission {
                id: 1,
                student_id: 22,
                assignment_id: 333,
                grade: Some("A".to_string()),
                score: Some(94.5),
                submitted_at: Some("2025-05-05".to_string()),
                graded_at: Some("2025-05-06".to_string()),
                posted_at: Some("2025-05-06".to_string()),
                late: false,
                missing: false,
            },
            Submission {
                id: 2,
                student_id: 22,
                assignment_id: 456,
                grade: Some("A".to_string()),
                score: Some(95.5),
                submitted_at: Some("2025-05-05".to_string()),
                graded_at: Some("2025-05-06".to_string()),
                posted_at: Some("2025-05-06".to_string()),
                late: false,
                missing: false,
            },
        ];
        let upsert = UpdateSubmissions { submissions };
        upsert.exec(&pool).await.unwrap();

        let submissions = all_submissions(&pool).await;

        assert_eq!(submissions[0].id, 1);
        assert_eq!(submissions[0].grade, Some("A".to_string()));
        assert_eq!(submissions[0].score, Some(94.5));
        assert_eq!(submissions[0].submitted_at, Some("2025-05-05".to_string()));
        assert_eq!(submissions[0].graded_at, Some("2025-05-06".to_string()));
        assert_eq!(submissions[0].posted_at, Some("2025-05-06".to_string()));
        assert!(!submissions[0].late);
        assert!(!submissions[0].missing);

        assert_eq!(submissions[1].id, 2);
        assert_eq!(submissions[1].grade, Some("A".to_string()));
        assert_eq!(submissions[1].score, Some(95.5));
        assert_eq!(submissions[1].submitted_at, Some("2025-05-05".to_string()));
        assert_eq!(submissions[1].graded_at, Some("2025-05-06".to_string()));
        assert_eq!(submissions[1].posted_at, Some("2025-05-06".to_string()));
        assert!(!submissions[1].late);
        assert!(!submissions[1].missing);
    }
}
