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
