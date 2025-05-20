use crate::{Result, models::StudentData};
use app::models::Student;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};

fn inc_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let param = h.param(0).unwrap().value().as_u64().unwrap();
    out.write(&(param + 1).to_string())?;
    Ok(())
}

pub fn render_student_data(student: &StudentData) -> Result<String> {
    let mut handlebars = Handlebars::new();

    handlebars.register_helper("inc", Box::new(inc_helper));
    handlebars.register_template_string("student", include_str!("../templates/student.hbs"))?;

    let out = handlebars.render("student", &student)?;
    Ok(out)
}

pub fn render_students(students: &Vec<Student>) -> Result<String> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("students", include_str!("../templates/students.hbs"))?;

    let out = handlebars.render("students", students)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use app::models::{Course, EnrollmentStatus, Student};

    use crate::models::Assignment;

    use super::*;

    fn build_student_data() -> StudentData {
        StudentData {
            student: Student {
                id: 42,
                name: "Pat".to_string(),
            },
            courses: vec![Course {
                id: 1,
                student_id: 42,
                name: "Chemistry".to_string(),
                enrollment_status: EnrollmentStatus::Active,
            }],
            assignments: vec![
                Assignment {
                    name: "Research Paper".to_string(),
                    course: "Chemistry".to_string(),
                    due_at: Some("2025-11-25".to_string()),
                    points_possible: Some(100.0),
                    grade: Some("A".to_string()),
                    score: Some(96.1),
                    submitted_at: Some("2025-10-13".to_string()),
                    graded_at: Some("2025-10-13".to_string()),
                    posted_at: Some("2025-10-13".to_string()),
                    late: false,
                    missing: false,
                },
                Assignment {
                    name: "Quiz".to_string(),
                    course: "Chemistry".to_string(),
                    due_at: Some("2025-11-25".to_string()),
                    points_possible: Some(100.0),
                    grade: None,
                    score: None,
                    submitted_at: None,
                    graded_at: None,
                    posted_at: None,
                    late: false,
                    missing: true,
                },
            ],
        }
    }

    #[test]
    fn test_render_ok() {
        let student_data = build_student_data();
        let out = render_student_data(&student_data).unwrap();
        let expected = r#"
Student: Pat

Enrolled Courses:
- Chemistry

Assignments:
1. Research Paper
    - Course: Chemistry
    - Due: 2025-11-25
    - Submitted: 2025-10-13
    - Graded: 2025-10-13
    - Grade: A
    - Score: 96.1
2. Quiz
    - Course: Chemistry
    - Due: 2025-11-25
    - Status: Missing!
"#
        .trim_start();

        assert_eq!(out, expected);
    }
}
