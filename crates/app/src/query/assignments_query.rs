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
            select a.id,
                   d.student_id,
                   a.course_id,
                   a.name,
                   d.due_at_utc as due_at,
                   a.points_possible,
                   a.grading_type
            from assignments a
            join student_assignment_dates d on d.assignment_id = a.id
            "#,
        );

        if let Some(student_id) = &self.student_id {
            builder
                .push("where d.student_id = ")
                .push_bind(student_id.0);
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

    #[sqlx::test]
    async fn shared_assignment_keeps_student_specific_dates(pool: SqlitePool) {
        sqlx::query("insert into students (id, canvas_user_id, name) values (1001, 1001, 'A'), (1002, 1002, 'B')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "insert into courses (id, canvas_course_id, name) values (2001, 2001, 'Shared')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("insert into assignments (id, canvas_assignment_id, course_id, name) values (3001, 3001, 2001, 'Shared Work')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("insert into student_assignment_dates (student_id, assignment_id, due_at_utc, source) values (1001, 3001, '2025-02-04T21:59:00Z', 'planner'), (1002, 3001, '2025-02-05T21:59:00Z', 'planner')")
            .execute(&pool)
            .await
            .unwrap();

        let assignments = AssignmentsQuery::default().exec(&pool).await.unwrap();

        assert_eq!(assignments.len(), 2);
        assert!(
            assignments.iter().any(
                |a| a.student_id == 1001 && a.due_at.as_deref() == Some("2025-02-04T21:59:00Z")
            )
        );
        assert!(
            assignments.iter().any(
                |a| a.student_id == 1002 && a.due_at.as_deref() == Some("2025-02-05T21:59:00Z")
            )
        );
    }
}
