use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Student {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug, FromRow, Serialize, Deserialize)]
pub struct Course {
    pub id: i64,
    pub student_id: i64,
    pub name: String,
    pub enrollment_status: EnrollmentStatus,
}

#[derive(Clone, Debug, FromRow, Serialize, Deserialize)]
pub struct Assignment {
    pub id: i64,
    pub student_id: i64,
    pub course_id: i64,
    pub name: String,
    pub due_at: Option<String>,
    pub points_possible: Option<f64>,
    pub grading_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssignmentWithSubmissions {
    pub assignment: Assignment,
    pub submissions: Vec<Submission>,
}

#[derive(Clone, Debug, FromRow, Serialize, Deserialize)]
pub struct Submission {
    pub id: i64,
    pub student_id: i64,
    pub assignment_id: i64,
    pub grade: Option<String>,
    pub score: Option<f64>,
    pub submitted_at: Option<String>,
    pub graded_at: Option<String>,
    pub posted_at: Option<String>,
    pub late: bool,
    pub missing: bool,
}

#[derive(Deserialize, FromRow)]
pub struct RawDbSubmission {
    pub id: i64,
    pub student_id: i64,
    pub assignment_id: i64,
    pub grade: Option<String>,
    pub score: Option<f64>,
    pub submitted_at: Option<String>,
    pub graded_at: Option<String>,
    pub posted_at: Option<String>,
    pub late: i64,
    pub missing: i64,
}

impl From<RawDbSubmission> for Submission {
    fn from(value: RawDbSubmission) -> Self {
        Submission {
            id: value.id,
            student_id: value.student_id,
            assignment_id: value.assignment_id,
            grade: value.grade.clone(),
            score: value.score,
            submitted_at: value.submitted_at.clone(),
            graded_at: value.graded_at.clone(),
            posted_at: value.posted_at.clone(),
            late: value.late == 1,
            missing: value.missing == 1,
        }
    }
}

impl From<&RawDbSubmission> for Submission {
    fn from(value: &RawDbSubmission) -> Self {
        Submission {
            id: value.id,
            student_id: value.student_id,
            assignment_id: value.assignment_id,
            grade: value.grade.clone(),
            score: value.score,
            submitted_at: value.submitted_at.clone(),
            graded_at: value.graded_at.clone(),
            posted_at: value.posted_at.clone(),
            late: value.late == 1,
            missing: value.missing == 1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct StudentId(pub i64);

#[derive(Debug, Clone)]
pub struct AppDataFilters {
    pub student: StudentId,
}

#[derive(Debug)]
pub struct AppData {
    pub students: Vec<Student>,
    pub courses: Vec<Course>,
    pub assignments: Vec<AssignmentWithSubmissions>,
}
