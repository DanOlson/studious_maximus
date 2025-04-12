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
}
