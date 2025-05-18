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
    let app = app::AppReadWrite::from_env()
        .await
        .expect("Failed to init app");

    match &cli.command {
        Some(Commands::Assignments { due_after: _ }) => {
            let results = app
                .get_assignments_with_submissions(None)
                .await
                .expect("to get data");
            println!("{results:?}");
        }
        None => {
            let data = app.get_app_data(None).await.expect("to get data");

            println!("{:?}\n", data);
        }
    }
}
