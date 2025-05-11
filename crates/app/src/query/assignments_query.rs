use sqlx::{FromRow, QueryBuilder, Sqlite};

use crate::models::{AppDataFilters, Assignment, StudentId};

use super::Query;

#[derive(Debug, Default)]
pub struct AssignmentsQuery {
    pub student_id: Option<StudentId>,
}

impl From<Option<AppDataFilters>> for AssignmentsQuery {
    fn from(value: Option<AppDataFilters>) -> Self {
        Self {
            student_id: value.map(|f| f.student),
        }
    }
}

impl Query for AssignmentsQuery {
    type Value = Vec<Assignment>;

    async fn exec(&self, pool: &sqlx::SqlitePool) -> anyhow::Result<Self::Value> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            select id, student_id, course_id, name, due_at, points_possible, grading_type
            from assignments
            "#,
        );

        if let Some(student_id) = &self.student_id {
            builder.push("where student_id = ").push_bind(student_id.0);
        }
        let assignments = builder
            .build()
            .fetch_all(pool)
            .await?
            .iter()
            .map(Assignment::from_row)
            .collect::<Result<_, _>>()?;

        Ok(assignments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use sqlx::SqlitePool;

    #[sqlx::test(fixtures("../../fixtures/assignments.sql"))]
    async fn query_unfiltered(pool: SqlitePool) {
        let query = AssignmentsQuery::default();
        let assignments = query.exec(&pool).await.unwrap();

        assert_eq!(assignments.len(), 6);
    }

    #[sqlx::test(fixtures("../../fixtures/assignments.sql"))]
    async fn query_filtered(pool: SqlitePool) {
        let query = AssignmentsQuery {
            student_id: Some(StudentId(123)),
        };
        let assignments = query.exec(&pool).await.unwrap();

        assert_eq!(assignments.len(), 3);
    }
}
