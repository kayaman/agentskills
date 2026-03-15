use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use crate::commands;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "skillz",
    about = "Agent skills package manager — install, manage, and discover AI agent skills.",
    version = VERSION
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a skill from GitHub or a local path
    Add {
        /// Skill source: owner/repo@skill, owner/repo, or local path
        source: String,
        /// Install to ~/.agents/skills/
        #[arg(short, long = "global")]
        global: bool,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// List installed skills (project + global)
    List {
        /// Show only global skills
        #[arg(short, long = "global")]
        global: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List installed skills (alias for list)
    Ls {
        /// Show only global skills
        #[arg(short, long = "global")]
        global: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove an installed skill
    Remove {
        /// Name of the skill to remove
        name: String,
        /// Remove from global skills
        #[arg(short, long = "global")]
        global: bool,
    },
    /// Remove an installed skill (alias for remove)
    Rm {
        /// Name of the skill to remove
        name: String,
        /// Remove from global skills
        #[arg(short, long = "global")]
        global: bool,
    },
    /// Check for updates and re-fetch changed skills
    Update {
        /// Skill name to update (all if omitted)
        name: Option<String>,
        /// Update global skills
        #[arg(short, long = "global")]
        global: bool,
    },
    /// Scaffold a new SKILL.md template
    Init {
        /// Skill name (defaults to current directory name)
        name: Option<String>,
        /// Output directory
        #[arg(short = 'd', long = "dir", default_value = ".")]
        dir: PathBuf,
    },
    /// Search GitHub for repos containing agent skills
    Find {
        /// Search query (e.g. 'react testing', 'deploy')
        query: String,
        /// Max results to show
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
    },
}

pub fn run() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Add {
            source,
            global,
            yes,
        } => commands::add::run(&source, global, yes),
        Commands::List { global, json } | Commands::Ls { global, json } => {
            commands::list::run(global, json)
        }
        Commands::Remove { name, global } | Commands::Rm { name, global } => {
            commands::remove::run(&name, global)
        }
        Commands::Update { name, global } => commands::update::run(name.as_deref(), global),
        Commands::Init { name, dir } => commands::init::run(name.as_deref(), &dir),
        Commands::Find { query, limit } => commands::find::run(&query, limit),
    };
    if let Err(e) = result {
        crate::console::error(&e.to_string());
        process::exit(1);
    }
}
