use db::SqlDatabase;
#[allow(unused)]
use lms::Lms;
use lms::canvas;
use models::{Course, EnrollmentStatus, Student};
use sqlx::SqlitePool;

mod db;
mod lms;
mod models;
mod prelude;
mod query;

pub struct App<L, D>
where
    L: Lms,
    D: db::Db,
{
    lms: L,
    database: D,
}

impl<L, D> App<L, D>
where
    L: Lms,
    D: db::Db,
{
    pub fn new(lms: L, database: D) -> Self {
        Self { lms, database }
    }

    pub async fn get_students(&self) -> anyhow::Result<Vec<Student>> {
        let query = query::StudentsQuery;

        let students = self.database.query(&query).await?;

        Ok(students)
    }

    pub async fn update_students(&self) -> anyhow::Result<()> {
        let students = self.lms.get_students().await?;
        let update = query::UpdateStudents {
            students: students
                .into_iter()
                .map(|s| models::Student {
                    id: s.id as i64,
                    name: s.name,
                })
                .collect(),
        };
        self.database.query(&update).await?;

        Ok(())
    }

    pub async fn get_courses(&self) -> anyhow::Result<Vec<Course>> {
        let query = query::CoursesQuery;
        let courses = self.database.query(&query).await?;

        Ok(courses)
    }

    pub async fn update_courses(&self) -> anyhow::Result<()> {
        let students = self.get_students().await?;
        for student in students {
            let courses = self.lms.get_active_courses(student.id).await?;
            let update = query::UpdateCourses {
                courses: courses
                    .into_iter()
                    .map(|c| models::Course {
                        id: c.id as i64,
                        student_id: student.id,
                        name: c.name,
                        enrollment_status: EnrollmentStatus::Active,
                    })
                    .collect(),
            };
            self.database.query(&update).await?;
        }

        Ok(())
    }
}

impl App<canvas::Client, SqlDatabase> {
    pub async fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let db_url = std::env::var("DATABASE_URL")?;
        let canvas_token = std::env::var("CANVAS_TOKEN")?;
        let canvas_base_url = std::env::var("CANVAS_BASE_URL")?;
        let pool = SqlitePool::connect(&db_url).await?;
        let database = SqlDatabase::new(pool);
        let lms = crate::lms::canvas::Client::new(canvas_base_url, &canvas_token);

        Ok(Self::new(lms, database))
    }
}

#[cfg(test)]
mod tests;
