mod bio;
mod config;
mod error;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("🧬 OpenLife v{} starting...", VERSION);
}

#[derive(Parser)]
#[command(name = "openlife")]
#[command(about = "🧬 The Best Bioinformatics AI Agent", long_about = None)]
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
    Gateway {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
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

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    
    let cli = Cli::parse();
    
    match cli.command {
        // 🧬 Bioinformatics commands - OpenLife's core feature
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
        
        // Version info with branding
        Commands::Version => {
            println!("🧬 OpenLife v{}", VERSION);
            println!();
            println!("   The Best Bioinformatics AI Agent");
            println!("   Built on ZeroClaw - The fastest AI framework");
            println!();
            println!("   Repository: https://github.com/openlife-ai/openlife");
            println!();
            println!("   Features:");
            println!("     • Pharmacogenomics (CPIC guidelines)");
            println!("     • Nutrigenomics & Ancestry");
            println!("     • Variant Annotation (VCF)");
            println!("     • Literature Synthesis");
            println!("     • Single-Cell Analysis");
            println!("     • Protein Structure Prediction");
        }
        
        // Gateway - Start the web interface
        Commands::Gateway { host, port } => {
            tracing::info!("Starting OpenLife Gateway on {}:{}", host, port);
            start_gateway(host, port).await?;
        }
        
        // Other ZeroClaw commands - delegate to zeroclaw CLI
        Commands::Onboard { interactive, force } => {
            run_zeroclaw_command(vec!["onboard", 
                if interactive { "--interactive" } else { "" },
                if force { "--force" } else { "" }
            ].into_iter().filter(|s| !s.is_empty()).collect()).await?;
        }
        
        Commands::Agent { message } => {
            if let Some(m) = message {
                run_zeroclaw_command(vec!["agent", "-m", &m]).await?;
            } else {
                run_zeroclaw_command(vec!["agent"]).await?;
            }
        }
        
        Commands::Daemon => run_zeroclaw_command(vec!["daemon"]).await?,
        Commands::Doctor => run_zeroclaw_command(vec!["doctor"]).await?,
        Commands::Status => run_zeroclaw_command(vec!["status"]).await?,
        Commands::Update => run_zeroclaw_command(vec!["update"]).await?,
        Commands::Estop => run_zeroclaw_command(vec!["estop"]).await?,
        Commands::Channel => run_zeroclaw_command(vec!["channel"]).await?,
        Commands::Cron => run_zeroclaw_command(vec!["cron"]).await?,
        Commands::Skill => run_zeroclaw_command(vec!["skill"]).await?,
    }

    Ok(())
}

async fn start_gateway(host: String, port: u16) -> Result<()> {
    println!("🌐 Starting OpenLife Gateway...");
    println!();
    println!("   URL: http://{}:{}", host, port);
    println!("   Dashboard: http://{}:{}/", host, port);
    println!();
    println!("   Press Ctrl+C to stop");
    println!();
    
    // 使用 zeroclaw CLI 启动 gateway（因为深度集成需要更多时间）
    run_zeroclaw_command(vec!["gateway"]).await?;
    
    Ok(())
}

async fn run_zeroclaw_command(args: Vec<&str>) -> Result<()> {
    use std::process::Command;
    
    let mut cmd = Command::new("zeroclaw");
    cmd.args(&args);
    
    // 对于 gateway 命令，只静默日志，保留关键输出
    let is_gateway = args.first().map(|a| *a == "gateway").unwrap_or(false);
    
    if is_gateway {
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        
        let mut child = cmd.spawn()?;
        
        use std::io::{BufRead, BufReader};
        
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
        
        let status = child.wait()?;
        std::process::exit(status.code().unwrap_or(1));
    } else {
        // 其他命令完全静默
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        
        let status = cmd.status()?;
        std::process::exit(status.code().unwrap_or(1));
    }
}
