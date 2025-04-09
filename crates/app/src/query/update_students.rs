use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::models::Student;

use super::Query;

#[derive(Debug)]
pub struct UpdateStudents {
    pub students: Vec<Student>,
}

impl Query for UpdateStudents {
    type Value = ();

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        if self.students.is_empty() {
            return Ok(());
        }

        let mut builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("insert into students (id, name) values");
        for (i, student) in self.students.iter().enumerate() {
            builder
                .push(" (")
                .push_bind(student.id)
                .push(", ")
                .push_bind(&student.name)
                .push(")");

            // push a comma unless we're on the last iteration
            if i < self.students.len() - 1 {
                builder.push(",");
            }
        }

        builder.push(" on conflict(id) do nothing");

        builder.build().execute(pool).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    async fn get_count(pool: &SqlitePool) -> i64 {
        let result = sqlx::query("select count(1) as count from students")
            .fetch_one(pool)
            .await
            .unwrap();
        let count: i64 = result.get("count");

        count
    }

    #[sqlx::test]
    async fn it_inserts_records(pool: SqlitePool) {
        let update = UpdateStudents {
            students: vec![
                Student {
                    id: 1,
                    name: "Noah".to_string(),
                },
                Student {
                    id: 2,
                    name: "Asher".to_string(),
                },
            ],
        };
        update.exec(&pool).await.unwrap();
        // verify on conflict handing
        update.exec(&pool).await.unwrap();

        let row_count = get_count(&pool).await;
        assert_eq!(row_count, 2);
    }
}
