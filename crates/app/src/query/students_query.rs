use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};

use crate::models::{AppDataFilters, Student, StudentId};

use super::Query;

#[derive(Debug, Default)]
pub struct StudentsQuery {
    student_id: Option<StudentId>,
}

impl From<Option<AppDataFilters>> for StudentsQuery {
    fn from(value: Option<AppDataFilters>) -> Self {
        Self {
            student_id: value.map(|f| f.student),
        }
    }
}

impl Query for StudentsQuery {
    type Value = Vec<Student>;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new("select id, name from students");
        if let Some(student_id) = &self.student_id {
            builder.push(" where id = ").push_bind(student_id.0);
        }
        let students = builder
            .build()
            .fetch_all(pool)
            .await?
            .iter()
            .map(Student::from_row)
            .collect::<Result<_, _>>()?;

        Ok(students)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use sqlx::SqlitePool;

    #[sqlx::test(fixtures("../../fixtures/students.sql"))]
    async fn query_unfiltered(pool: SqlitePool) {
        let query = StudentsQuery::default();
        let students = query.exec(&pool).await.unwrap();
        assert_eq!(students.len(), 2);
    }

    #[sqlx::test(fixtures("../../fixtures/students.sql"))]
    async fn query_filtered(pool: SqlitePool) {
        let query = StudentsQuery {
            student_id: Some(StudentId(29756)),
        };
        let students = query.exec(&pool).await.unwrap();
        assert_eq!(students.len(), 1);
        assert_eq!(students[0].name, "Bob");
    }
}
