mod bio;
mod config;
mod error;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("OpenLife v{} starting...", bio::OPENLIFE_VERSION);
}

#[derive(Parser)]
#[command(name = "openlife")]
#[command(about = "ZeroClaw-based Bioinformatics AI Agent", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Onboard {
        #[arg(long)]
        interactive: bool,
        #[arg(long)]
        force: bool,
    },
    Agent {
        #[arg(short, long)]
        message: Option<String>,
    },
    Daemon,
    Doctor,
    Status,
    Update,
    Estop,
    Channel,
    Cron,
    Skill,
    Bio {
        #[command(subcommand)]
        action: BioAction,
    },
    Version,
}

#[derive(Subcommand)]
enum BioAction {
    List,
    Info {
        skill_name: String,
    },
    Run {
        skill_name: String,
        #[arg(short, long)]
        input: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
    Install {
        skill_path: String,
    },
    Query {
        natural_language: String,
        #[arg(short, long)]
        input: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn run_zeroclaw(args: &[&str]) -> ! {
    let status = std::process::Command::new("zeroclaw")
        .args(args)
        .status()
        .expect("Failed to run zeroclaw");
    std::process::exit(status.code().unwrap_or(1));
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Bio { action } => {
            match action {
                BioAction::List => bio::list_skills().await?,
                BioAction::Info { skill_name } => bio::show_skill_info(&skill_name).await?,
                BioAction::Run { skill_name, input, output } => {
                    bio::run_skill(&skill_name, input.as_deref(), output.as_deref()).await?
                }
                BioAction::Install { skill_path } => bio::install_skill(&skill_path).await?,
                BioAction::Query { natural_language, input, output } => {
                    bio::query_with_natural_language(&natural_language, input.as_deref(), output.as_deref()).await?
                }
            }
        }
        Commands::Version => {
            println!("OpenLife v{} (based on ZeroClaw)", bio::OPENLIFE_VERSION);
        }
        Commands::Onboard { interactive, force } => {
            if interactive && force {
                run_zeroclaw(&["onboard", "--interactive", "--force"]);
            } else if interactive {
                run_zeroclaw(&["onboard", "--interactive"]);
            } else if force {
                run_zeroclaw(&["onboard", "--force"]);
            } else {
                run_zeroclaw(&["onboard"]);
            }
        }
        Commands::Agent { message } => {
            if let Some(m) = message {
                run_zeroclaw(&["agent", "-m", &m]);
            } else {
                run_zeroclaw(&["agent"]);
            }
        }
        Commands::Daemon => run_zeroclaw(&["daemon"]),
        Commands::Doctor => run_zeroclaw(&["doctor"]),
        Commands::Status => run_zeroclaw(&["status"]),
        Commands::Update => run_zeroclaw(&["update"]),
        Commands::Estop => run_zeroclaw(&["estop"]),
        Commands::Channel => run_zeroclaw(&["channel", "--help"]),
        Commands::Cron => run_zeroclaw(&["cron", "--help"]),
        Commands::Skill => run_zeroclaw(&["skill", "--help"]),
    }

    Ok(())
}
