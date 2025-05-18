use std::collections::HashMap;

use app::models::{AppData, Course, Student};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Assignment {
    pub name: String,
    pub course: String,
    pub due_at: Option<String>,
    pub points_possible: Option<f64>,
    // submission fields
    pub grade: Option<String>,
    pub score: Option<f64>,
    pub submitted_at: Option<String>,
    pub graded_at: Option<String>,
    pub posted_at: Option<String>,
    pub late: bool,
    pub missing: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
    pub data: Vec<StudentData>,
}

#[derive(Serialize, Deserialize)]
pub struct StudentData {
    pub student: Student,
    pub courses: Vec<Course>,
    pub assignments: Vec<Assignment>,
}

impl From<AppData> for Output {
    fn from(value: AppData) -> Self {
        let mut data = vec![];

        for student in value.students.into_iter() {
            let courses = value
                .courses
                .iter()
                .filter(|c| c.student_id == student.id)
                .cloned()
                .collect::<Vec<Course>>();
            let mut course_names_by_id: HashMap<i64, String> =
                courses.iter().map(|c| (c.id, c.name.clone())).collect();
            let assignments = value
                .assignments
                .iter()
                .filter(|c| c.assignment.student_id == student.id)
                .map(|a| {
                    let submission = a.submissions.first().unwrap().to_owned();
                    Assignment {
                        name: a.assignment.name.clone(),
                        course: course_names_by_id
                            .remove(&a.assignment.course_id)
                            .unwrap_or_default(),
                        due_at: a.assignment.due_at.clone(),
                        points_possible: a.assignment.points_possible,
                        grade: submission.grade,
                        score: submission.score,
                        submitted_at: submission.submitted_at,
                        graded_at: submission.graded_at,
                        posted_at: submission.posted_at,
                        late: submission.late,
                        missing: submission.missing,
                    }
                })
                .collect::<Vec<Assignment>>();
            let student_data = StudentData {
                student,
                courses,
                assignments,
            };
            data.push(student_data);
        }

        Self { data }
    }
}
