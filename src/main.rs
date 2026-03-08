mod bio;
mod config;
mod error;

use anyhow::Result;
use clap::{Parser, Subcommand};

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Bio { action } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
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
                Ok::<(), anyhow::Error>(())
            })?;
        }
        
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
        
        Commands::Gateway { host, port } => {
            start_dashboard(host, port)?;
        }
        
        Commands::Onboard { interactive, force } => {
            run_zeroclaw_command(vec!["onboard", 
                if interactive { "--interactive" } else { "" },
                if force { "--force" } else { "" }
            ].into_iter().filter(|s| !s.is_empty()).collect());
        }
        
        Commands::Agent { message } => {
            if let Some(m) = message {
                run_zeroclaw_command(vec!["agent", "-m", &m]);
            } else {
                run_zeroclaw_command(vec!["agent"]);
            }
        }
        
        Commands::Daemon => run_zeroclaw_command(vec!["daemon"]),
        Commands::Doctor => run_zeroclaw_command(vec!["doctor"]),
        Commands::Status => run_zeroclaw_command(vec!["status"]),
        Commands::Update => run_zeroclaw_command(vec!["update"]),
        Commands::Estop => run_zeroclaw_command(vec!["estop"]),
        Commands::Channel => run_zeroclaw_command(vec!["channel"]),
        Commands::Cron => run_zeroclaw_command(vec!["cron"]),
        Commands::Skill => run_zeroclaw_command(vec!["skill"]),
    }

    Ok(())
}

fn start_dashboard(host: String, port: u16) -> Result<()> {
    use std::net::TcpListener;
    use std::io::{Read, Write};
    
    println!("🌐 Starting OpenLife Dashboard...");
    println!();
    println!("   URL: http://{}:{}", host, port);
    println!("   Press Ctrl+C to stop");
    println!();
    
    let html_content = include_str!("dashboard.html");
    
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)?;
    
    println!("🧬 OpenLife Dashboard is running!");
    println!("   Local: http://{}/", addr);
    println!();
    
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buffer = [0; 2048];
        
        stream.read(&mut buffer).ok();
        
        let request = String::from_utf8_lossy(&buffer);
        
        let response = if request.starts_with("GET / ") || request.starts_with("GET /index.html") {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html_content.len(),
                html_content
            )
        } else if request.starts_with("GET /api/status") {
            let status = r#"{"status":"running","version":"0.1.0","skills":14}"#;
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status.len(),
                status
            )
        } else if request.starts_with("GET /api/skills") {
            let skills = r#"{"skills":["pharmgx-reporter","nutrigx-advisor","equity-scorer","gwas-database","clinvar-database","pubmed-database","chembl-database","cosmic-database","uniprot-database","ensembl-database","string-database","kegg-database"]}"#;
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                skills.len(),
                skills
            )
        } else {
            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
        };
        
        stream.write_all(response.as_bytes()).ok();
        stream.flush().ok();
    }
    
    Ok(())
}

fn run_zeroclaw_command(args: Vec<&str>) {
    use std::process::Command;
    
    let mut cmd = Command::new("zeroclaw");
    cmd.args(&args);
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::null());
    
    let status = cmd.status().unwrap_or_default();
    std::process::exit(status.code().unwrap_or(1));
}
