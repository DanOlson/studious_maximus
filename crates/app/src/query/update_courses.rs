use sqlx::{QueryBuilder, Sqlite};

use crate::models::Course;

use super::Query;

#[derive(Debug)]
pub struct UpdateCourses {
    pub courses: Vec<Course>,
}

impl Query for UpdateCourses {
    type Value = ();

    async fn exec(&self, pool: &sqlx::SqlitePool) -> anyhow::Result<Self::Value> {
        if self.courses.is_empty() {
            return Ok(());
        }

        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "insert into courses (id, student_id, name, enrollment_status) values",
        );
        for (i, course) in self.courses.iter().enumerate() {
            builder
                .push(" (")
                .push_bind(course.id)
                .push(", ")
                .push_bind(course.student_id)
                .push(", ")
                .push_bind(&course.name)
                .push(", ")
                .push_bind(course.enrollment_status.to_string().clone())
                .push(")");

            if i < self.courses.len() - 1 {
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
    use sqlx::{Row, SqlitePool};

    use crate::models::EnrollmentStatus;

    use super::*;

    async fn get_count(pool: &SqlitePool) -> i64 {
        let result = sqlx::query("select count(1) as count from courses")
            .fetch_one(pool)
            .await
            .unwrap();
        let count: i64 = result.get("count");

        count
    }

    #[sqlx::test]
    fn insert_courses(pool: SqlitePool) {
        let update = UpdateCourses {
            courses: vec![
                Course {
                    id: 101,
                    student_id: 13,
                    name: "Douglas".to_string(),
                    enrollment_status: EnrollmentStatus::Active,
                },
                Course {
                    id: 202,
                    student_id: 13,
                    name: "Fredrick".to_string(),
                    enrollment_status: EnrollmentStatus::Active,
                },
            ],
        };
        let res = update.exec(&pool).await;
        assert!(res.is_ok());
        // verify conflict handling doesn't return Err
        update.exec(&pool).await.unwrap();

        let row_count = get_count(&pool).await;
        assert_eq!(row_count, 2);
    }
}
