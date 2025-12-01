use clap::{Parser, Subcommand};
use eyre::eyre;
use kenchiku_scaffold::{
    Scaffold,
    discovery::{discover_scaffold, find_all_scaffolds},
};
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show information about a scaffold.
    Show {
        /// Scaffold to show information about, either name or path.
        scaffold: String,
    },
    /// List all discovered scaffolds.
    List,
    /// Construct a scaffold by running it's construct function
    Construct {
        /// Scaffold to construct, either name or path.
        scaffold: String,
    },
    /// Runs a patch of a scaffold
    Patch {
        /// Patch to run, in the format of "<scaffold>:<patch_name>", for example
        /// "utils:add_logging"
        patch: String,
    },
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

    match cli.command {
        Commands::Show {
            scaffold: scaffold_name,
        } => {
            let scaffold_path =
                discover_scaffold(scaffold_name).ok_or(eyre!("Scaffold not found"))?;
            let scaffold = Scaffold::load(scaffold_path)?;
            scaffold.print();
        }
        Commands::List => {
            let found_scaffolds = find_all_scaffolds()
                .iter()
                .map(|path| Scaffold::load(path.to_path_buf()))
                .collect::<eyre::Result<Vec<Scaffold>>>()?;
            println!("Found scaffolds:\n======");
            for scaffold in found_scaffolds {
                scaffold.print();
                println!("======");
            }
        }
        Commands::Construct {
            scaffold: scaffold_name,
        } => {
            info!(scaffold_name, "Starting construction...");
            let scaffold_path =
                discover_scaffold(scaffold_name).ok_or(eyre!("Scaffold not found"))?;
            let scaffold = Scaffold::load(scaffold_path)?;
            scaffold.call_construct()?;
        }
        Commands::Patch { patch } => {
            let mut split = patch.split(":");
            let scaffold_name = split
                .next()
                .ok_or(eyre!("no scaffold name found in {}", patch))?;
            let patch_name = split.next().ok_or(eyre!(
                "no patch name found in {}, did you use the format '<scaffold>:<patch>'?",
                patch
            ))?;
            info!(scaffold_name, patch_name, "Starting patching...");
            let scaffold_path =
                discover_scaffold(scaffold_name.to_string()).ok_or(eyre!("Scaffold not found"))?;
            let scaffold = Scaffold::load(scaffold_path)?;
            scaffold.call_patch(patch_name)?;
        }
    }

    Ok(())
}
