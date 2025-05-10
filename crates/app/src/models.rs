use std::fmt::Display;

use serde::Deserialize;
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

#[derive(Debug)]
pub struct AssignmentWithSubmissions {
    pub assignment: Assignment,
    pub submissions: Vec<Submission>,
}

#[derive(Debug, FromRow)]
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

#[derive(Deserialize)]
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

#[derive(Debug)]
pub struct AllData {
    pub students: Vec<Student>,
    pub courses: Vec<Course>,
    pub assignments: Vec<AssignmentWithSubmissions>,
}

impl Display for AllData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Students:\n")?;
        for student in &self.students {
            f.write_str(&format!("\tname: {}\n", student.name))?;
            f.write_str(&format!("\tid: {}\n", student.id))?;
            f.write_str("\n")?;
        }
        f.write_str("\n")?;

        f.write_str("Courses:\n")?;
        for course in &self.courses {
            f.write_str(&format!("\tname: {}\n", course.name))?;
            f.write_str(&format!("\tid: {}\n", course.id))?;
            f.write_str(&format!("\tstudent_id: {}\n", course.student_id))?;
            f.write_str("\n")?;
        }
        f.write_str("\n")?;

        f.write_str("Assignments:\n")?;
        for assignment in &self.assignments {
            f.write_str(&format!("\tname: {}\n", &assignment.assignment.name))?;
            f.write_str(&format!("\tid: {}\n", assignment.assignment.id))?;
            f.write_str(&format!(
                "\tstudent_id: {}\n",
                assignment.assignment.student_id
            ))?;
            f.write_str(&format!(
                "\tcourse_id: {}\n",
                assignment.assignment.course_id
            ))?;
            if let Some(due_at) = &assignment.assignment.due_at {
                f.write_str(&format!("\tdue_at: {due_at}\n"))?;
            }
            if let Some(points_possible) = &assignment.assignment.points_possible {
                f.write_str(&format!("\tpoints_possible: {points_possible}\n"))?;
            }
            if let Some(grading_type) = &assignment.assignment.grading_type {
                f.write_str(&format!("\tgrading_type: {grading_type}\n"))?;
            }

            if assignment.submissions.is_empty() {
                f.write_str("\n")?;
                continue;
            }

            f.write_str("\tSubmissions:\n")?;
            for submission in &assignment.submissions {
                f.write_str(&format!("\t\tid: {}\n", submission.id))?;
                f.write_str(&format!("\t\tstudent_id: {}\n", submission.student_id))?;
                f.write_str(&format!(
                    "\t\tassignment_id: {}\n",
                    submission.assignment_id
                ))?;
                f.write_str(&format!(
                    "\t\tgrade: {}\n",
                    &submission
                        .grade
                        .as_ref()
                        .unwrap_or(&"Not graded".to_string())
                ))?;
                if let Some(score) = &submission.score {
                    f.write_str(&format!("\t\tscore: {score}\n"))?;
                }
                if let Some(submitted_at) = &submission.submitted_at {
                    f.write_str(&format!("\t\tsubmitted_at: {submitted_at}\n"))?;
                }
                if let Some(graded_at) = &submission.graded_at {
                    f.write_str(&format!("\t\tgraded_at: {graded_at}\n"))?;
                }
                if let Some(posted_at) = &submission.posted_at {
                    f.write_str(&format!("\t\tposted_at: {posted_at}\n"))?;
                }
                if submission.late {
                    f.write_str("\t\tLate!\n")?;
                }
                if submission.missing {
                    f.write_str("\t\tMissing!\n")?;
                }
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}
