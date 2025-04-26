use sqlx::{QueryBuilder, Sqlite, SqlitePool};

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

        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
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
            ) values
        "#,
        );
        for (i, submission) in self.submissions.iter().enumerate() {
            builder.push(" (");
            builder.push_bind(submission.id);
            builder.push(", ");
            builder.push_bind(submission.student_id);
            builder.push(", ");
            builder.push_bind(submission.assignment_id);
            builder.push(", ");
            builder.push_bind(&submission.grade);
            builder.push(", ");
            builder.push_bind(submission.score);
            builder.push(", ");
            builder.push_bind(&submission.submitted_at);
            builder.push(", ");
            builder.push_bind(&submission.graded_at);
            builder.push(", ");
            builder.push_bind(&submission.posted_at);
            builder.push(", ");
            builder.push_bind(submission.late);
            builder.push(", ");
            builder.push_bind(submission.missing);
            builder.push(")");

            // push a comma unless we're on the last iteration
            if i < self.submissions.len() - 1 {
                builder.push(", ");
            }
        }

        builder.push(
            r#"
            on conflict(id) do update
            set grade=excluded.grade,
                score=excluded.score,
                submitted_at=excluded.submitted_at,
                graded_at=excluded.graded_at,
                posted_at=excluded.posted_at,
                late=excluded.late,
                missing=excluded.missing"#,
        );
        builder.build().execute(pool).await?;

        Ok(())
    }
}
