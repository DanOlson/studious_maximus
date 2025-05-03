use super::{Lms, dto};

pub struct Noop;

impl Lms for Noop {
    async fn get_students(&self) -> anyhow::Result<Vec<dto::Student>> {
        Err(anyhow::anyhow!("Readonly mode!"))
    }

    async fn get_active_courses(&self, _account_id: i64) -> anyhow::Result<Vec<dto::Course>> {
        Err(anyhow::anyhow!("Readonly mode!"))
    }

    async fn get_course_assignments(
        &self,
        _account_id: i64,
        _course_id: i64,
    ) -> anyhow::Result<Vec<dto::Assignment>> {
        Err(anyhow::anyhow!("Readonly mode!"))
    }

    async fn get_course_submissions(
        &self,
        _course_id: i64,
        _student_id: i64,
    ) -> anyhow::Result<Vec<dto::Submission>> {
        Err(anyhow::anyhow!("Readonly mode!"))
    }
}
