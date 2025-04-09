use sqlx::{FromRow, SqlitePool};

use crate::models::{Course, EnrollmentStatus};

use super::Query;

#[derive(Debug)]
pub struct CoursesQuery;

impl Query for CoursesQuery {
    type Value = Vec<Course>;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        let courses = sqlx::query_as!(
            RawDbCourse,
            r#"
            select id, student_id, name, enrollment_status
            from courses
            where enrollment_status = 'active'
            "#
        )
        .fetch_all(pool)
        .await?
        .iter()
        .map(Course::from)
        .collect::<Vec<Course>>();

        Ok(courses)
    }
}

#[derive(FromRow)]
struct RawDbCourse {
    pub id: i64,
    pub student_id: i64,
    pub name: String,
    #[allow(unused)]
    pub enrollment_status: String,
}

impl From<&RawDbCourse> for Course {
    fn from(value: &RawDbCourse) -> Self {
        Self {
            id: value.id,
            student_id: value.student_id,
            name: value.name.clone(),
            enrollment_status: EnrollmentStatus::Active,
        }
    }
}
