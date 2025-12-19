use std::{collections::HashMap, env::current_dir, path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand};
use eyre::eyre;
use inquire::Confirm;
use kenchiku_common::Context;
use kenchiku_scaffold::{
    Scaffold,
    discovery::{discover_scaffold, find_all_scaffolds},
};
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
struct Cli {
    /// Increases verbosity/decreases log level. -v -> info, -vv -> debug, -vvv -> trace
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
        /// The path where the scaffold will be generated. Defaults to the current directory.
        output: Option<String>,
        /// Auto confirm actions, use multiple times to auto confirm more dangerous actions.
        #[arg(short, long, action = clap::ArgAction::Count)]
        confirm_all: u8,
        /// Force will overwrite existing files in the output path.
        #[arg(short, long)]
        force: bool,
        /// Values to set before running. Can be repeated.
        #[arg(short('s'), long("set"), value_name = "VALUE")]
        values: Vec<String>,
    },
    /// Runs a patch of a scaffold
    Patch {
        /// Patch to run, in the format of "<scaffold>:<patch_name>", for example
        /// "utils:add_logging"
        patch: String,
        /// The path where the patch will run. Defaults to the current directory.
        output: Option<String>,
        /// Auto confirm actions, use multiple times to auto confirm more dangerous actions.
        #[arg(short, long, action = clap::ArgAction::Count)]
        confirm_all: u8,
        /// Values to set before running. Can be repeated.
        #[arg(short('s'), long("set"), value_name = "VALUE")]
        values: Vec<String>,
    },
    /// Starts the MCP server (stdio)
    Mcp,
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
        .with_writer(std::io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| eyre::eyre!("Failed to set tracing subscriber: {}", e))?;

    info!(VERSION, "Kenchiku running");

    let prompt_value = Arc::new(
        |value_type: String,
         description: String,
         choices: Option<Vec<String>>,
         default: Option<String>|
         -> eyre::Result<String> {
            Ok(match value_type.as_str() {
                "enum" => {
                    let choices =
                        choices.ok_or_else(|| eyre::eyre!("choices required for enum"))?;
                    let mut select = inquire::Select::new(&description, choices.clone());
                    if let Some(def) = &default {
                        let cursor = choices
                            .iter()
                            .position(|el| el == def)
                            .ok_or_else(|| eyre::eyre!("default not in choices"))?;
                        select = select.with_starting_cursor(cursor);
                    }
                    select.prompt()?
                }
                "bool" => {
                    let msg = format!("{} (y/n)", description);
                    let mut query = inquire::Confirm::new(&msg);
                    if let Some(def) = &default {
                        query = inquire::Confirm::new(&description)
                            .with_default(def == "true")
                            .with_placeholder(if def == "true" { "y" } else { "n" });
                    }
                    query.prompt().map(|b| b.to_string())?
                }
                _ => {
                    let mut text = inquire::Text::new(&description);
                    if let Some(def) = &default {
                        text = text.with_default(&def).with_placeholder(&def);
                    }
                    text.prompt()?
                }
            })
        },
    );

    match cli.command {
        Commands::Show {
            scaffold: scaffold_name,
        } => {
            let scaffold_path =
                discover_scaffold(scaffold_name).ok_or(eyre!("Scaffold not found"))?;
            let scaffold = Scaffold::load(scaffold_path)?;
            let mut stdout = std::io::stdout();
            scaffold.print(&mut stdout, true)?;
        }
        Commands::List => {
            let found_scaffolds = find_all_scaffolds()
                .iter()
                .map(|path| Scaffold::load(path.to_path_buf()))
                .collect::<eyre::Result<Vec<Scaffold>>>()?;
            let mut stdout = std::io::stdout();
            use std::io::Write;
            writeln!(stdout, "Found scaffolds:\n======")?;
            for scaffold in found_scaffolds {
                scaffold.print(&mut stdout, false)?;
                writeln!(stdout, "======")?;
            }
        }
        Commands::Construct {
            scaffold: scaffold_name,
            output,
            confirm_all,
            force,
            values,
        } => {
            info!(scaffold_name, ?values, "Starting construction...");
            let scaffold_path =
                discover_scaffold(scaffold_name).ok_or(eyre!("Scaffold not found"))?;
            let scaffold = Scaffold::load(scaffold_path)?;
            let out_path = output.map(PathBuf::from).unwrap_or(current_dir()?);
            let mut temp_dir = tempfile::tempdir()?;
            let context = Context {
                working_dir: temp_dir.path().to_path_buf(),
                confirm_all,
                output: out_path,
                scaffold_dir: scaffold.path.clone(),
                confirm_fn: Arc::new(|message: String| {
                    // TODO: handle ctrl c
                    Ok(Confirm::new(&message).with_default(false).prompt()?)
                }),
                allow_overwrite: force,
                values_meta: scaffold.meta.values.clone(),
                values: values
                    .iter()
                    .map(|val| {
                        val.split_once("=")
                            .map(|vals| (vals.0.to_string(), vals.1.to_string()))
                            .ok_or_else(|| eyre!("Invalid value format: {}", val))
                    })
                    .collect::<Result<HashMap<String, String>, _>>()?,
                prompt_value,
            };
            scaffold.construct(context)?;
            // only disable cleanup if we constructed successfully
            temp_dir.disable_cleanup(true);
        }
        Commands::Patch {
            patch,
            output,
            confirm_all,
            values,
        } => {
            let mut split = patch.split(":");
            let scaffold_name = split
                .next()
                .ok_or(eyre!("no scaffold name found in {}", patch))?;
            let patch_name = split.next().ok_or(eyre!(
                "no patch name found in {}, did you use the format '<scaffold>:<patch>'?",
                patch
            ))?;
            info!(scaffold_name, patch_name, ?values, "Starting patching...");
            let scaffold_path =
                discover_scaffold(scaffold_name.to_string()).ok_or(eyre!("Scaffold not found"))?;
            let scaffold = Scaffold::load(scaffold_path)?;
            let out_path = output.map(PathBuf::from).unwrap_or(current_dir()?);
            let context = Context {
                working_dir: current_dir()?,
                confirm_all,
                output: out_path,
                scaffold_dir: scaffold.path.clone(),
                confirm_fn: Arc::new(|message: String| {
                    Ok(Confirm::new(&message).with_default(false).prompt()?)
                }),
                values_meta: scaffold
                    .meta
                    .patches
                    .get(patch_name)
                    .expect("patch to exist here")
                    .values
                    .clone(),
                values: values
                    .iter()
                    .map(|val| {
                        val.split_once("=")
                            .map(|vals| (vals.0.to_string(), vals.1.to_string()))
                            .ok_or_else(|| eyre!("Invalid value format: {}", val))
                    })
                    .collect::<Result<HashMap<String, String>, _>>()?,
                prompt_value,
                ..Default::default()
            };
            scaffold.call_patch(patch_name, context)?;
        }
        Commands::Mcp => {
            kenchiku_mcp::server::run_blocking()?;
        }
    }

    Ok(())
}
