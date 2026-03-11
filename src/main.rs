mod bio;
mod config;
mod error;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{Read, Write};

use std::net::TcpListener;
use std::process::Command;

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
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Version,
}

#[derive(Subcommand)]
enum BioAction {
    List,
    Info { skill_name: String },
    Run {
        skill_name: String,
        #[arg(short, long)]
        input: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
    Install { skill_path: String },
    Query {
        natural_language: String,
        #[arg(short, long)]
        input: Option<String>,
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    Show,
    Set {
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        model: Option<String>,
    },
    Init,
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

        Commands::Config { action } => handle_config(action)?,

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
            println!("     • +225 LabClaw Skills Integrated");
        }

        Commands::Gateway { host, port } => start_gateway(host, port)?,

        Commands::Agent { message } => {
            if let Some(m) = message {
                run_zeroclaw_command(vec!["agent", "-m", &m], false);
            } else {
                run_zeroclaw_command(vec!["agent"], false);
            }
        }

        Commands::Onboard { interactive, force } => {
            run_zeroclaw_command(
                vec![
                    "onboard",
                    if interactive { "--interactive" } else { "" },
                    if force { "--force" } else { "" },
                ]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect(),
                true,
            );
        }

        Commands::Daemon => run_zeroclaw_command(vec!["daemon"], true),
        Commands::Doctor => run_zeroclaw_command(vec!["doctor"], true),
        Commands::Status => run_zeroclaw_command(vec!["status"], true),
        Commands::Update => run_zeroclaw_command(vec!["update"], true),
        Commands::Estop => run_zeroclaw_command(vec!["estop"], true),
        Commands::Channel => run_zeroclaw_command(vec!["channel"], true),
        Commands::Cron => run_zeroclaw_command(vec!["cron"], true),
        Commands::Skill => run_zeroclaw_command(vec!["skill"], true),
    }

    Ok(())
}

fn handle_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => {
            let config = config::OpenLifeConfig::load()?;
            println!("🧬 OpenLife Configuration");
            println!();
            println!("  Config file: {:?}", config::OpenLifeConfig::config_path());
            println!();
            println!("  LLM Provider: {}", config.llm.provider);
            println!("  Model: {}", config.llm.model);
            println!("  API Key: {}", if config.llm.api_key.is_some() { "(configured)" } else { "(not set)" });
            if let Some(base_url) = &config.llm.base_url {
                println!("  Base URL: {}", base_url);
            }
            println!();
            println!("  Skills dir: {:?}", config.bio.skills_dir);
            println!("  Cache dir: {:?}", config.bio.cache_dir);
            println!();
            println!("  Gateway: http://{}:{}", config.gateway.host, config.gateway.port);
            println!();
            println!("  ZeroClaw: {}", if std::path::Path::new("/home/andy/.zeroclaw/config.toml").exists() { "✅ configured" } else { "❌ not configured" });
        }
        ConfigAction::Set { provider, api_key, model } => {
            let mut config = config::OpenLifeConfig::load()?;
            if let Some(p) = &provider {
                if let Some(key) = &api_key {
                    config.set_api_key(p, key);
                } else {
                    config.llm.provider = p.clone();
                }
            }
            if let Some(key) = &api_key {
                config.llm.api_key = Some(key.clone());
            }
            if let Some(m) = model {
                config.llm.model = m;
            }
            config.save()?;
            sync_to_zeroclaw(&config)?;
            println!("✅ Configuration saved and synced to ZeroClaw!");
        }
        ConfigAction::Init => {
            let config = config::OpenLifeConfig::default();
            config.save()?;
            println!("✅ OpenLife configuration initialized!");
            println!("   Config file: {:?}", config::OpenLifeConfig::config_path());
            println!();
            println!("   Next steps:");
            println!("     1. Set your API key:");
            println!("        openlife config set --provider openrouter --api-key YOUR_KEY");
        }
    }
    Ok(())
}

fn sync_to_zeroclaw(config: &config::OpenLifeConfig) -> Result<()> {
    let zeroclaw_config = std::path::Path::new("/home/andy/.zeroclaw/config.toml");
    if !zeroclaw_config.exists() {
        println!("⚠️  ZeroClaw config not found, skipping sync");
        return Ok(());
    }
    let content = std::fs::read_to_string(zeroclaw_config)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    for line in lines.iter_mut() {
        if line.starts_with("default_provider") {
            *line = format!("default_provider = \"{}\"", config.llm.provider);
        }
        if line.starts_with("default_model") {
            *line = format!("default_model = \"{}\"", config.llm.model);
        }
    }
    let updated = lines.join("\n");
    std::fs::write(zeroclaw_config, updated)?;
    println!("🔄 Synced to ZeroClaw config");
    Ok(())
}

fn start_gateway(host: String, port: u16) -> Result<()> {
    println!("🌐 Starting OpenLife Gateway...");
    println!();
    println!("   URL: http://{}:{}", host, port);
    println!();
    println!("   🧬 No pairing required - just start chatting!");
    println!("   🧪 LabClaw Skills: 225+ bioinformatics skills available");
    println!("   Press Ctrl+C to stop");
    println!();

    let html_content = include_str!("dashboard.html");
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)?;

    println!("🧬 OpenLife Gateway is running!");
    println!("   Open http://{}/ in your browser", addr);
    println!();

    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buffer = [0; 8192];
        stream.read(&mut buffer).ok();
        let request = String::from_utf8_lossy(&buffer);
        let request_line = request.lines().next().unwrap_or("");
        let path = request_line.split_whitespace().nth(1).unwrap_or("/");
        let response = handle_request(path, &request, html_content);
        stream.write_all(response.as_bytes()).ok();
        stream.flush().ok();
    }
    Ok(())
}

fn handle_request(path: &str, request: &str, html: &str) -> String {
    match path {
        "/" | "/index.html" => {
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", html.len(), html)
        }
        "/api/status" => {
            let status = r#"{"status":"running","version":"0.1.0","pairing":false}"#;
            format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", status.len(), status)
        }
        "/api/chat" if request.contains("POST") => {
            let body = request.split("\r\n\r\n").nth(1).unwrap_or("{}");
            let message = if body.contains("\"message\"") {
                body.split("\"message\"").nth(1)
                    .and_then(|s| s.split(':').nth(1))
                    .and_then(|s| s.trim().strip_prefix('"').and_then(|s| s.strip_suffix('"')))
                    .unwrap_or("Hello")
                    .to_string()
            } else {
                "Hello".to_string()
            };

            let output = Command::new("zeroclaw")
                .arg("agent")
                .arg("-m")
                .arg(&message)
                .output();

            let response = match output {
                Ok(o) => {
                    if o.status.success() {
                        String::from_utf8_lossy(&o.stdout).to_string()
                    } else {
                        String::from_utf8_lossy(&o.stderr).to_string()
                    }
                }
                Err(_) => "Error: Could not connect to agent. Make sure ZeroClaw is configured.".to_string()
            };

            let json = format!(r#"{{"response":"{}"}}"#, response.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n").chars().take(2000).collect::<String>());
            format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", json.len(), json)
        }
        _ => "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
    }
}

fn run_zeroclaw_command(args: Vec<&str>, silent: bool) {
    let mut cmd = Command::new("zeroclaw");
    cmd.args(&args);
    if silent {
        cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    } else {
        cmd.stdin(std::process::Stdio::inherit()).stdout(std::process::Stdio::inherit()).stderr(std::process::Stdio::inherit());
    }
    let status = cmd.status().unwrap_or_default();
    std::process::exit(status.code().unwrap_or(if silent { 1 } else { 0 }));
}
