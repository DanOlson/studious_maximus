use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};

use crate::models::{AppDataFilters, Course, EnrollmentStatus, StudentId};

use super::Query;

#[derive(Debug, Default)]
pub struct CoursesQuery {
    student_id: Option<StudentId>,
}

impl From<Option<AppDataFilters>> for CoursesQuery {
    fn from(value: Option<AppDataFilters>) -> Self {
        Self {
            student_id: value.map(|f| f.student),
        }
    }
}

impl Query for CoursesQuery {
    type Value = Vec<Course>;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            select id, student_id, name, enrollment_status
            from courses
            where enrollment_status = 'active'
            "#,
        );
        if let Some(student_id) = &self.student_id {
            builder.push(" and student_id = ").push_bind(student_id.0);
        }
        let courses = builder
            .build()
            .fetch_all(pool)
            .await?
            .iter()
            .map(RawDbCourse::from_row)
            .map(|raw| raw.map(Course::from))
            .collect::<Result<_, _>>()?;

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

impl From<RawDbCourse> for Course {
    fn from(value: RawDbCourse) -> Self {
        Self {
            id: value.id,
            student_id: value.student_id,
            name: value.name.clone(),
            enrollment_status: EnrollmentStatus::Active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use sqlx::SqlitePool;

    #[sqlx::test(fixtures("../../fixtures/courses.sql"))]
    async fn query_unfiltered(pool: SqlitePool) {
        let query = CoursesQuery::default();
        let courses = query.exec(&pool).await.unwrap();

        assert_eq!(courses.len(), 7);
    }

    #[sqlx::test(fixtures("../../fixtures/courses.sql"))]
    async fn query_filtered(pool: SqlitePool) {
        let query = CoursesQuery {
            student_id: Some(StudentId(234)),
        };
        let courses = query.exec(&pool).await.unwrap();

        assert_eq!(courses.len(), 3);
        assert!(courses.iter().all(|c| c.student_id == 234));
    }
}
