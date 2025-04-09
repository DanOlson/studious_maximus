use sqlx::SqlitePool;

use crate::models::Student;

use super::Query;

#[derive(Debug)]
pub struct StudentsQuery;

impl Query for StudentsQuery {
    type Value = Vec<Student>;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        let students = sqlx::query_as!(Student, "select id, name from students")
            .fetch_all(pool)
            .await?;

        Ok(students)
    }
}
