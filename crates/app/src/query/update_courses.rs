use sqlx::QueryBuilder;

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

        QueryBuilder::new("insert into courses (id, student_id, name, enrollment_status)")
            .push_values(self.courses.iter(), |mut bld, course| {
                bld.push_bind(course.id);
                bld.push_bind(course.student_id);
                bld.push_bind(&course.name);
                bld.push_bind(course.enrollment_status.to_string().clone());
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
    use sqlx::SqlitePool;

    use crate::models::EnrollmentStatus;

    use super::*;

    async fn get_count(pool: &SqlitePool) -> i64 {
        sqlx::query_scalar("select count(1) as count from courses")
            .fetch_one(pool)
            .await
            .unwrap()
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
