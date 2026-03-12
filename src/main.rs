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
    Chains,
    Search {
        query: String,
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
                    BioAction::Chains => {
                        bio::list_chains();
                    }
                    BioAction::Search { query } => {
                        bio::search_skills(&query).await?
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
            println!();
            println!("   New in v{}:", VERSION);
            println!("     • Enhanced Orchestrator with multi-intent detection");
            println!("     • Skill chain composition for complex workflows");
            println!("     • Conversation context and memory");
            println!("     • LLM-assisted intent classification (optional)");
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
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
            let zeroclaw_config = format!("{}/.zeroclaw/config.toml", home);
            println!("  ZeroClaw: {}", if std::path::Path::new(&zeroclaw_config).exists() { "✅ configured" } else { "❌ not configured" });
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
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let zeroclaw_config = format!("{}/.zeroclaw/config.toml", home);
    let zeroclaw_config = std::path::Path::new(&zeroclaw_config);
    
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
        "/api/skills" => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let skills_json = rt.block_on(async {
                get_skills_json().await
            });
            format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", skills_json.len(), skills_json)
        }
        "/api/chains" => {
            let chains_json = get_chains_json();
            format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", chains_json.len(), chains_json)
        }
        "/api/query" if request.contains("POST") => {
            let body = request.split("\r\n\r\n").nth(1).unwrap_or("{}");
            let query = extract_json_string(body, "query");
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                handle_query(&query).await
            });
            
            format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", result.len(), result)
        }
        "/api/run" if request.contains("POST") => {
            let body = request.split("\r\n\r\n").nth(1).unwrap_or("{}");
            let skill = extract_json_string(body, "skill");
            let input = extract_json_string(body, "input");
            let output = extract_json_string(body, "output");
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                handle_run(&skill, &input, &output).await
            });
            
            format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", result.len(), result)
        }
        "/api/chat" if request.contains("POST") => {
            let body = request.split("\r\n\r\n").nth(1).unwrap_or("{}");
            let message = extract_json_string(body, "message");

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
                Err(_) => "Error: Could not connect to agent.".to_string()
            };

            let json = format!(r#"{{"response":"{}"}}"#, escape_json(&response));
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

fn extract_json_string(body: &str, key: &str) -> String {
    let pattern = format!("\"{}\"", key);
    if let Some(after_key) = body.split(&pattern).nth(1) {
        if let Some(after_colon) = after_key.split(':').nth(1) {
            let trimmed = after_colon.trim();
            if trimmed.starts_with('"') {
                if let Some(end) = trimmed[1..].find('"') {
                    return trimmed[1..end+1].to_string();
                }
            } else if let Some(end) = trimmed.find(|c: char| c == ',' || c == '}') {
                return trimmed[..end].trim().to_string();
            }
        }
    }
    String::new()
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .chars()
        .take(4000)
        .collect()
}

async fn get_skills_json() -> String {
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    use bio::{SkillRegistry, SkillManifest};
    
    static REGISTRY: Lazy<Mutex<SkillRegistry>> = Lazy::new(|| {
        Mutex::new(SkillRegistry::new(bio::get_skills_dir()))
    });
    
    let mut registry = REGISTRY.lock().unwrap();
    let _ = registry.ensure_loaded();
    
    let skills: Vec<serde_json::Value> = registry
        .all()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "version": s.version,
                "description": s.description,
                "tags": s.tags,
                "inputs": s.inputs.len(),
                "outputs": s.outputs.len()
            })
        })
        .collect();
    
    serde_json::to_string(&skills).unwrap_or_else(|_| "[]".to_string())
}

fn get_chains_json() -> String {
    use bio::orchestrator::chain::ChainBuilder;
    
    let builder = ChainBuilder::new();
    let chains: Vec<serde_json::Value> = builder
        .list_chains()
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
                "description": c.description,
                "steps": c.steps.iter().map(|s| serde_json::json!({
                    "name": s.name,
                    "skill": s.skill
                })).collect::<Vec<_>>(),
                "estimated_time": c.estimated_time
            })
        })
        .collect();
    
    serde_json::to_string(&chains).unwrap_or_else(|_| "[]".to_string())
}

async fn handle_query(query: &str) -> String {
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    use bio::{EnhancedOrchestrator, SkillRegistry, SkillExecutor, BioIntent};
    
    static ORCHESTRATOR: Lazy<Mutex<EnhancedOrchestrator>> = Lazy::new(|| {
        Mutex::new(EnhancedOrchestrator::new())
    });
    
    static REGISTRY: Lazy<Mutex<SkillRegistry>> = Lazy::new(|| {
        Mutex::new(SkillRegistry::new(bio::get_skills_dir()))
    });
    
    let mut orchestrator = ORCHESTRATOR.lock().unwrap();
    let plan = orchestrator.route(query).await;
    
    let mut registry = REGISTRY.lock().unwrap();
    let _ = registry.ensure_loaded();
    
    let intent_name = plan.primary_intent.display_name().to_string();
    let confidence = plan.confidence;
    let skill_name = plan.primary_intent.skill_name().to_string();
    
    let response = if plan.primary_intent == BioIntent::Unknown {
        format!("I couldn't determine the analysis type. Try being more specific about what you want to analyze.")
    } else if skill_name.is_empty() {
        format!("Detected intent: {} but no skill is available for this.", intent_name)
    } else {
        // Check if skill exists in registry
        if let Some(skill) = registry.get(&skill_name) {
            format!("Detected intent: {} with {:.0}% confidence. Ready to run skill: {}", intent_name, confidence * 100.0, skill.name)
        } else {
            format!("Detected intent: {} with {:.0}% confidence. Install the {} skill to perform this analysis.", intent_name, confidence * 100.0, skill_name)
        }
    };
    
    serde_json::json!({
        "intent": intent_name,
        "confidence": confidence,
        "skill": skill_name,
        "response": response
    }).to_string()
}

async fn handle_run(skill_name: &str, input: &str, output: &str) -> String {
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    use bio::{SkillRegistry, SkillExecutor};
    use std::path::PathBuf;
    
    static REGISTRY: Lazy<Mutex<SkillRegistry>> = Lazy::new(|| {
        Mutex::new(SkillRegistry::new(bio::get_skills_dir()))
    });
    
    let mut registry = REGISTRY.lock().unwrap();
    let _ = registry.ensure_loaded();
    
    let skill = match registry.get(skill_name) {
        Some(s) => s.clone(),
        None => {
            return serde_json::json!({
                "success": false,
                "error": format!("Skill '{}' not found", skill_name)
            }).to_string();
        }
    };
    
    let executor = SkillExecutor::new(std::env::temp_dir());
    
    let input_path = if input.is_empty() { None } else { Some(PathBuf::from(input)) };
    let output_path = if output.is_empty() { None } else { Some(PathBuf::from(output)) };
    
    match executor.execute(&skill, input_path.as_deref(), output_path.as_deref()) {
        Ok(result) => {
            serde_json::json!({
                "success": result.success,
                "output": result.output,
                "error": result.error,
                "duration_ms": result.duration_ms
            }).to_string()
        }
        Err(e) => {
            serde_json::json!({
                "success": false,
                "error": e.to_string()
            }).to_string()
        }
    }
}