use sqlx::SqlitePool;

use crate::prelude::Stable;

mod assignments_query;
mod courses_query;
mod students_query;
mod submissions_query;
mod update_assignments;
mod update_courses;
mod update_students;
mod update_submissions;

pub use assignments_query::AssignmentsQuery;
pub use courses_query::CoursesQuery;
pub use students_query::StudentsQuery;
pub use submissions_query::SubmissionsQuery;
pub use update_assignments::UpdateAssignments;
pub use update_courses::UpdateCourses;
pub use update_students::UpdateStudents;
pub use update_submissions::UpdateSubmissions;

pub trait Query: Stable {
    type Value;

    fn exec(&self, pool: &SqlitePool) -> impl Future<Output = anyhow::Result<Self::Value>> + Send;
}
