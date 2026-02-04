use clap::{Parser, Subcommand};

use crate::{
    cli::{create::create, list::list, remove::remove, serve::serve, status::status, stop::stop},
    status::set::set_status_initial,
};

pub mod create;
pub mod list;
pub mod remove;
pub mod serve;
pub mod status;
pub mod stop;

#[derive(Parser)]
#[command(name = "AutoPilot-rs")]
#[command(about = "a cross platform automation tool", version = "1.0")]
struct Cli {
    #[arg(long)]
    config_path: Option<String>,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Subcommand
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Serve AutoPilot-rs
    Serve,
    /// Stop AutoPilot-rs
    Stop,
    /// Create a new Job
    Create,
    /// Remove a Job
    Remove,
    /// List Jobs
    List,
    /// Status of AutoPilot-rs
    Status,
}

pub async fn handle_cli() {
    let cli = Cli::parse();
    // AutoPilot::prepare_logging();
    match &cli.command {
        Some(Commands::Serve) => {
            set_status_initial().expect("Failed to initialize status");
            serve(cli.config_path).await;
        }
        Some(Commands::Create) => {
            create();
        }
        Some(Commands::Remove) => {
            remove();
        }

        Some(Commands::Stop) => {
            stop(false);
        }
        Some(Commands::List) => {
            list();
        }
        Some(Commands::Status) => {
            status();
        }
        None => {
            return;
        }
    };
}
