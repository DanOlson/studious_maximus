use sqlx::{FromRow, QueryBuilder, Sqlite};

use super::Query;
use crate::models::{AppDataFilters, RawDbSubmission, StudentId, Submission};

#[derive(Debug, Default)]
pub struct SubmissionsQuery {
    pub student_id: Option<StudentId>,
}

impl From<Option<AppDataFilters>> for SubmissionsQuery {
    fn from(value: Option<AppDataFilters>) -> Self {
        Self {
            student_id: value.map(|f| f.student),
        }
    }
}

impl Query for SubmissionsQuery {
    type Value = Vec<Submission>;

    async fn exec(&self, pool: &sqlx::SqlitePool) -> anyhow::Result<Self::Value> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
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
            "#,
        );

        if let Some(student_id) = &self.student_id {
            builder.push("where student_id = ").push_bind(student_id.0);
        }

        let submissions = builder
            .push("order by student_id, assignment_id")
            .build()
            .fetch_all(pool)
            .await?
            .iter()
            .map(RawDbSubmission::from_row)
            .map(|r| r.map(Submission::from))
            .collect::<Result<_, _>>()?;

        Ok(submissions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use sqlx::SqlitePool;

    #[sqlx::test(fixtures("../../fixtures/submissions.sql"))]
    async fn query_unfiltered(pool: SqlitePool) {
        let query = SubmissionsQuery::default();
        let submissions = query.exec(&pool).await.unwrap();

        assert_eq!(submissions.len(), 3);
    }

    #[sqlx::test(fixtures("../../fixtures/submissions.sql"))]
    async fn query_filtered(pool: SqlitePool) {
        let query = SubmissionsQuery {
            student_id: Some(StudentId(23)),
        };
        let submissions = query.exec(&pool).await.unwrap();

        assert_eq!(submissions.len(), 1);
    }
}
