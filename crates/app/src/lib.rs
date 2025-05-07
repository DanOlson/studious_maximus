use std::collections::HashMap;

#[cfg(feature = "write")]
pub use canvas::Client;
pub use db::SqlDatabase;
use lms::Lms;
#[cfg(feature = "write")]
use lms::canvas;
use lms::noop;
use models::EnrollmentStatus;
use sqlx::SqlitePool;

mod db;
mod lms;
pub mod models;
mod prelude;
mod query;

#[derive(Clone)]
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

    pub async fn get_students(&self) -> anyhow::Result<Vec<models::Student>> {
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

    pub async fn get_courses(&self) -> anyhow::Result<Vec<models::Course>> {
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

    pub async fn get_assignments(
        &self,
        due_on_or_after: chrono::NaiveDate,
    ) -> anyhow::Result<Vec<models::Assignment>> {
        let query = query::AssignmentsQuery { due_on_or_after };
        let assignments = self.database.query(&query).await?;

        Ok(assignments)
    }

    pub async fn update_assignments(&self) -> anyhow::Result<()> {
        let courses = self.get_courses().await?;
        for course in courses {
            tokio::try_join!(
                self.upsert_assignments(&course),
                self.upsert_submissions(&course),
            )?;
        }

        Ok(())
    }

    async fn upsert_assignments(&self, course: &models::Course) -> anyhow::Result<()> {
        let assignments = self
            .lms
            .get_course_assignments(course.student_id, course.id)
            .await?;
        let update = query::UpdateAssignments {
            assignments: assignments
                .into_iter()
                .map(|a| models::Assignment {
                    id: a.id as i64,
                    student_id: course.student_id,
                    course_id: course.id,
                    name: a.name,
                    due_at: a.due_at,
                    points_possible: a.points_possible,
                    grading_type: a.grading_type,
                })
                .collect(),
        };
        self.database.query(&update).await?;

        Ok(())
    }

    async fn upsert_submissions(&self, course: &models::Course) -> anyhow::Result<()> {
        let submissions = self
            .lms
            .get_course_submissions(course.id, course.student_id)
            .await?;
        let update = query::UpdateSubmissions {
            submissions: submissions
                .into_iter()
                .map(|s| models::Submission {
                    id: s.id as i64,
                    student_id: s.student_id as i64,
                    assignment_id: s.assignment_id as i64,
                    grade: s.grade,
                    score: s.score,
                    submitted_at: s.submitted_at,
                    graded_at: s.graded_at,
                    posted_at: s.posted_at,
                    late: s.late,
                    missing: s.missing,
                })
                .collect(),
        };
        self.database.query(&update).await?;

        Ok(())
    }

    pub async fn get_submissions(&self) -> anyhow::Result<Vec<models::Submission>> {
        let query = query::SubmissionsQuery;
        let res = self.database.query(&query).await?;

        Ok(res)
    }

    pub async fn get_assignments_with_submissions(
        &self,
    ) -> anyhow::Result<Vec<models::AssignmentWithSubmissions>> {
        let on_or_after = chrono::NaiveDate::parse_from_str("2024-08-01", "%Y-%m-%d").unwrap();
        let (assignments, submissions) =
            tokio::try_join!(self.get_assignments(on_or_after), self.get_submissions())?;
        let mut submissions_by_assignment_id: HashMap<i64, Vec<models::Submission>> =
            HashMap::new();

        for s in submissions {
            submissions_by_assignment_id
                .entry(s.assignment_id)
                .or_default()
                .push(s);
        }

        let x = assignments
            .into_iter()
            .map(|assignment| {
                let submissions = submissions_by_assignment_id
                    .remove(&assignment.id)
                    .map_or_else(Vec::new, |subs| subs);
                models::AssignmentWithSubmissions {
                    assignment,
                    submissions,
                }
            })
            .collect();

        Ok(x)
    }

    pub async fn get_all_data(&self) -> anyhow::Result<models::AllData> {
        let (students, courses, assignments) = tokio::try_join!(
            self.get_students(),
            self.get_courses(),
            self.get_assignments_with_submissions()
        )?;

        Ok(models::AllData {
            students,
            courses,
            assignments,
        })
    }
}

#[cfg(feature = "write")]
impl AppReadWrite {
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

impl AppReadonly {
    pub async fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let db_url = std::env::var("DATABASE_URL")?;
        let pool = SqlitePool::connect(&db_url).await?;
        let database = SqlDatabase::new(pool);
        let lms = crate::lms::noop::Noop;

        Ok(Self::new(lms, database))
    }
}

#[cfg(feature = "write")]
pub type AppReadWrite = App<canvas::Client, SqlDatabase>;

pub type AppReadonly = App<noop::Noop, SqlDatabase>;

#[cfg(test)]
mod tests;
