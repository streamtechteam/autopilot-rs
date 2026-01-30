use clap::{Parser, Subcommand};
use log::info;

use crate::cli::{serve::serve, stop::stop};

pub mod create;
pub mod list;
pub mod remove;
pub mod serve;
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
    New,
    /// Remove a Job
    Remove,
    /// List Jobs
    List,
}

pub async fn handle_cli() {
    let cli = Cli::parse();
    info!("log");
    match &cli.command {
        Some(Commands::New) => {}
        Some(Commands::Remove) => {}
        Some(Commands::Serve) => {
            serve(cli.config_path).await;
        }
        Some(Commands::Stop) => {
            stop();
        }
        Some(Commands::List) => {}
        None => {}
    }
}
