use std::fmt::Display;

use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Student {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, FromRow)]
pub struct Course {
    pub id: i64,
    pub student_id: i64,
    pub name: String,
    pub enrollment_status: EnrollmentStatus,
}

#[derive(Debug, FromRow)]
pub struct Assignment {
    pub id: i64,
    pub student_id: i64,
    pub course_id: i64,
    pub name: String,
    pub due_at: Option<String>,
    pub points_possible: Option<f64>,
    pub grading_type: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct Submission {
    pub id: i64,
    pub student_id: i64,
    pub assignment_id: i64,
    pub grade: Option<String>,
    pub score: Option<f32>,
    pub submitted_at: Option<String>,
    pub graded_at: Option<String>,
    pub posted_at: Option<String>,
    pub late: bool,
    pub missing: bool,
}

#[derive(Debug)]
pub enum EnrollmentStatus {
    Active,
    Pending,
    Completed,
}

impl Display for EnrollmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            EnrollmentStatus::Active => "active",
            EnrollmentStatus::Pending => "pending",
            EnrollmentStatus::Completed => "completed",
        };
        f.write_str(data)
    }
}
