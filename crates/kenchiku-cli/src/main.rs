use std::env::current_dir;

use clap::Parser;
use kenchiku_scaffold::Scaffold;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    let filter = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from(filter))
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| eyre::eyre!("Failed to set tracing subscriber: {}", e))?;

    info!(VERSION, "Kenchiku running");

    let scaffold = Scaffold::load(current_dir()?)?;
    println!("Scaffold:");
    println!(" desc: {}", scaffold.meta.description);
    println!(" patches:");
    for (name, patch) in scaffold.meta.patches {
        println!("  {}: {}", name, patch.description);
    }

    Ok(())
}
