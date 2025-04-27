use std::fmt::Display;

use app::models;
use clap::{Parser, Subcommand};

#[derive(Debug)]
struct AllData {
    pub students: Vec<models::Student>,
    pub courses: Vec<models::Course>,
    pub assignments: Vec<models::AssignmentWithSubmissions>,
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

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Assignments {
        #[arg(short, long)]
        due_after: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let app = app::App::from_env().await.expect("Failed to init app");

    match &cli.command {
        Some(Commands::Assignments { due_after: _ }) => {
            let results = app
                .get_assignments_with_submissions()
                .await
                .expect("to get data");
            println!("{results:?}");
        }
        None => {
            let students = app.get_students().await.expect("to get students");
            let courses = app.get_courses().await.expect("to get courses");
            let assignments = app
                .get_assignments_with_submissions()
                .await
                .expect("to get assignments");

            let data = AllData {
                students,
                courses,
                assignments,
            };

            println!("{}\n", data);
        }
    }
}
