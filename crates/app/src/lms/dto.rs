use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Student {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Course {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Assignment {
    pub id: i32,
    pub due_at: Option<String>,
    pub name: String,
    pub points_possible: Option<f64>,
    pub grading_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Submission {
    pub id: i32,
    #[serde(rename(deserialize = "user_id"))]
    pub student_id: i32,
    pub assignment_id: i32,
    pub grade: Option<String>,
    pub score: Option<f64>,
    pub submitted_at: Option<String>,
    pub graded_at: Option<String>,
    pub posted_at: Option<String>,
    pub workflow_state: Option<String>,
    pub late: bool,
    pub missing: bool,
}
