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
pub struct FrontPage {
    #[serde(default)]
    pub page_id: Option<i64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub published: Option<bool>,
    #[serde(default)]
    pub front_page: Option<bool>,
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
