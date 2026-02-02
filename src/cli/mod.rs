use clap::{Parser, Subcommand};
use log::info;

use crate::{
    autopilot::AutoPilot,
    cli::{list::list, serve::serve, stop::stop},
    state::set::set_state_initial,
};

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
    // AutoPilot::prepare_logging();
    match &cli.command {
        Some(Commands::New) => {}
        Some(Commands::Remove) => {}
        Some(Commands::Serve) => {
            set_state_initial().expect("Failed to initialize state");
            serve(cli.config_path).await;
        }
        Some(Commands::Stop) => {
            stop();
        }
        Some(Commands::List) => {
            list();
        }
        None => {
            return;
        }
    };
}
