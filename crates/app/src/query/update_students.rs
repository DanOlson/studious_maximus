use sqlx::{QueryBuilder, SqlitePool};

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

        QueryBuilder::new("insert into students (id, name)")
            .push_values(self.students.iter(), |mut bld, student| {
                bld.push_bind(student.id).push_bind(&student.name);
            })
            .push(" on conflict(id) do nothing")
            .build()
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn get_count(pool: &SqlitePool) -> i64 {
        sqlx::query_scalar("select count(1) as count from students")
            .fetch_one(pool)
            .await
            .unwrap()
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
