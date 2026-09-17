#[cfg(feature = "write")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let base_url = std::env::var("CANVAS_BASE_URL")?;
    let token = std::env::var("CANVAS_TOKEN")?;
    let client = app::Client::new(base_url, &token);

    let students = client.get_students().await?;
    println!("Observed students:");
    for student in &students {
        println!("  {}  {}", student.id, student.name);
    }

    println!();
    println!("Active courses by observed student:");
    for student in students {
        let courses = client.get_active_courses(student.id as i64).await?;
        println!("{} ({})", student.name, student.id);
        for course in courses {
            println!("  {}  {}", course.id, course.name);
        }
    }

    Ok(())
}

#[cfg(not(feature = "write"))]
fn main() {
    eprintln!("list_canvas_ids requires the app crate's `write` feature");
    std::process::exit(1);
}
