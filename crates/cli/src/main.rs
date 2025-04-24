use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};

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
        Some(Commands::Assignments { due_after }) => {
            let due = due_after
                .clone()
                .map(|due| {
                    NaiveDate::parse_from_str(&due, "%Y-%m-%d").unwrap_or_else(|_| {
                        eprintln!("Could not parse due-after - using current date instead");
                        Local::now().date_naive()
                    })
                })
                .unwrap_or_else(|| Local::now().date_naive());
            let assignments = app.get_assignments(due).await.expect("");
            println!("{:?}", assignments);
        }
        None => {}
    }
}
