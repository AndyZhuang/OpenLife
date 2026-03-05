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
    Gateway,
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
    use std::process::Command;
    
    let mut cmd = Command::new("zeroclaw");
    cmd.args(args);
    
    // 对于 gateway 命令，只静默日志，保留关键输出
    let is_gateway = !args.is_empty() && args[0] == "gateway";
    
    if is_gateway {
        cmd.stderr(std::process::Stdio::piped());
        
        let child = cmd.spawn();
        match child {
            Ok(mut child) => {
                use std::io::{BufRead, BufReader};
                
                // 获取 stdout
                if let Some(stdout) = child.stdout.take() {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            // 只显示关键信息行
                            if line.contains("listening") 
                                || line.contains("Web Dashboard")
                                || line.contains("PAIRING")
                                || line.contains("┌")
                                || line.contains("│")
                                || line.contains("└")
                                || line.contains("🦀")
                                || line.contains("🌐")
                                || line.contains("🔐")
                                || line.contains("Press Ctrl") {
                                println!("{}", line);
                            }
                        }
                    }
                }
                
                // 等待进程
                let status = child.wait();
                match status {
                    Ok(s) => std::process::exit(s.code().unwrap_or(1)),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: Failed to run zeroclaw: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // 其他命令完全静默
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        
        let child = cmd.spawn();
        
        match child {
            Ok(mut child) => {
                match child.wait() {
                    Ok(status) => {
                        std::process::exit(status.code().unwrap_or(1));
                    }
                    Err(e) => {
                        eprintln!("Error: Failed to wait for zeroclaw: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: Failed to run zeroclaw: {}", e);
                eprintln!("Make sure ZeroClaw is installed: cargo install zeroclaw");
                std::process::exit(1);
            }
        }
    }
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
        Commands::Gateway => run_zeroclaw(&["gateway"]),
    }

    Ok(())
}
