pub mod canvas;
pub mod dto;

#[cfg_attr(test, mockall::automock)]
pub trait Lms {
    fn get_students(
        &self,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<dto::Student>>> + Send;

    fn get_active_courses(
        &self,
        account_id: i64,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<dto::Course>>> + Send;

    fn get_course_assignments(
        &self,
        account_id: i64,
        course_id: i64,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<dto::Assignment>>> + Send;
}
