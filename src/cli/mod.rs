use clap::{Parser, Subcommand};

use crate::{
    cli::{create::create, list::list, remove::remove, serve::serve, status::status, stop::stop},
    fs::{CONFIG_PATH, set_all_paths, set_autopilot_path},
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
    handle_dir(cli.config_path.clone());
    match &cli.command {
        Some(Commands::Serve) => {
            serve(cli.verbose).await;
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

fn handle_dir(config_path: Option<String>) {
    // println!("cli config path : {}", config_path.clone().unwrap());

    set_autopilot_path(config_path.clone()).expect("Failed to setup dirs");
    if let Err(e) = set_all_paths(false) {
        eprintln!("Failed to set up directories: {}", e);
        std::process::exit(1);
    }

    // println!("CONFIG_PATH {}", CONFIG_PATH.get().unwrap());
}
