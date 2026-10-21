mod cli;

use tokio::{self};

#[tokio::main]
async fn main() {
    cli::handle_cli().await;
}
