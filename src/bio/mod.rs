pub mod orchestrator;
pub mod core;
pub mod skill_registry;

pub use orchestrator::{BioIntent, EnhancedOrchestrator, RoutingPlan};
pub use skill_registry::{SkillRegistry, SkillLoader, SkillManifest, SkillExecutor};

pub const OPENLIFE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_skills_dir() -> std::path::PathBuf {
    let dirs = directories::ProjectDirs::from("ai", "openlife", "openlife")
        .expect("Failed to get project directories");
    dirs.data_dir().join("skills")
}

/// List all installed skills
pub async fn list_skills() -> anyhow::Result<()> {
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    
    static REGISTRY: Lazy<Mutex<SkillRegistry>> = Lazy::new(|| {
        Mutex::new(SkillRegistry::new(get_skills_dir()))
    });
    
    let mut registry = REGISTRY.lock().unwrap();
    registry.ensure_loaded()?;
    
    let skills: Vec<_> = registry.all().collect();
    
    if skills.is_empty() {
        println!("No skills installed. Use `openlife bio install <path>` to install a skill.");
        println!("\nBundled skills available:");
        println!("  • skills/pharmgx-reporter - Pharmacogenomic analysis");
        println!("  • skills/nutrigx-advisor - Nutrigenomics recommendations");
        println!("  • skills/equity-scorer - Diversity scoring");
        println!("  • skills/vcf-annotator - Variant annotation");
        println!("  • skills/labclaw/bio/* - LabClaw bioinformatics skills");
        return Ok(());
    }
    
    println!("\n📦 Installed Bio-Skills ({}):", skills.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let mut skills: Vec<_> = skills.into_iter().collect();
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    
    for skill in skills {
        println!("  {} - {}", skill.name, skill.description);
        if !skill.tags.is_empty() {
            println!("    Tags: {}", skill.tags.join(", "));
        }
    }
    
    Ok(())
}

/// Show detailed info for a skill
pub async fn show_skill_info(skill_name: &str) -> anyhow::Result<()> {
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    
    static REGISTRY: Lazy<Mutex<SkillRegistry>> = Lazy::new(|| {
        Mutex::new(SkillRegistry::new(get_skills_dir()))
    });
    
    let mut registry = REGISTRY.lock().unwrap();
    registry.ensure_loaded()?;
    
    let skill = registry.get(skill_name)
        .ok_or_else(|| anyhow::anyhow!("Skill '{}' not found. Use `openlife bio list` to see available skills.", skill_name))?;
    
    println!("\n┌─────────────────────────────────────────┐");
    println!("│ {} v{}", skill.name, skill.version);
    println!("├─────────────────────────────────────────┤");
    println!("│ {}", skill.description);
    println!("└─────────────────────────────────────────┘");
    
    if let Some(ref author) = skill.author {
        println!("\n👤 Author: {}", author);
    }
    
    if let Some(ref license) = skill.license {
        println!("📜 License: {}", license);
    }
    
    if !skill.tags.is_empty() {
        println!("\n🏷️  Tags: {}", skill.tags.join(", "));
    }
    
    if !skill.inputs.is_empty() {
        println!("\n📥 Inputs:");
        for input in &skill.inputs {
            let formats = if input.format.is_empty() {
                String::new()
            } else {
                format!(" ({})", input.format.join(", "))
            };
            println!("  • {} [{}]{} - {}", input.name, input.input_type, formats, input.description);
        }
    }
    
    if !skill.outputs.is_empty() {
        println!("\n📤 Outputs:");
        for output in &skill.outputs {
            println!("  • {} [{}] - {}", output.name, output.output_type, output.description);
        }
    }
    
    if let Some(ref script) = skill.script_path {
        println!("\n🐍 Script: {}", script.display());
    }
    
    if !skill.dependencies.is_empty() {
        println!("\n📦 Dependencies: {}", skill.dependencies.join(", "));
    }
    
    println!("\n💡 Usage:");
    println!("   openlife bio run {} --input <file> --output <dir>", skill_name);
    
    Ok(())
}

/// Run a skill with given inputs
pub async fn run_skill(skill_name: &str, input: Option<&str>, output: Option<&str>) -> anyhow::Result<()> {
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    
    static REGISTRY: Lazy<Mutex<SkillRegistry>> = Lazy::new(|| {
        Mutex::new(SkillRegistry::new(get_skills_dir()))
    });
    
    let mut registry = REGISTRY.lock().unwrap();
    registry.ensure_loaded()?;
    
    let skill = registry.get(skill_name)
        .ok_or_else(|| anyhow::anyhow!("Skill '{}' not found", skill_name))?
        .clone();
    
    let executor = SkillExecutor::new(std::env::temp_dir());
    
    // Validate dependencies first
    let missing = executor.validate_dependencies(&skill)?;
    if !missing.is_empty() {
        println!("⚠️  Missing dependencies:");
        for dep in &missing {
            println!("   • {}", dep);
        }
        println!("\nInstall with: pip install <package>");
    }
    
    let input_path = input.map(|p| std::path::PathBuf::from(p));
    let output_path = output.map(|p| std::path::PathBuf::from(p));
    
    println!("🚀 Running skill: {} v{}", skill.name, skill.version);
    
    let result = executor.execute(&skill, input_path.as_deref(), output_path.as_deref())?;
    
    if result.success {
        println!("\n✅ Success ({}ms)", result.duration_ms);
        if !result.output.is_empty() {
            println!("\n{}", result.output);
        }
        if !result.output_files.is_empty() {
            println!("\n📄 Output files:");
            for file in &result.output_files {
                println!("   • {}", file.display());
            }
        }
    } else {
        eprintln!("\n❌ Execution failed");
        if let Some(error) = result.error {
            eprintln!("\nError: {}", error);
        }
        anyhow::bail!("Skill execution failed");
    }
    
    Ok(())
}

/// Install a skill from a path
pub async fn install_skill(skill_path: &str) -> anyhow::Result<()> {
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    
    static REGISTRY: Lazy<Mutex<SkillRegistry>> = Lazy::new(|| {
        Mutex::new(SkillRegistry::new(get_skills_dir()))
    });
    
    let mut source = std::path::PathBuf::from(skill_path);
    
    // Support bundled skill paths like "labclaw/bio/pyhealth"
    if !source.exists() && skill_path.starts_with("labclaw/") {
        source = std::path::PathBuf::from("skills").join(skill_path);
    }
    
    if !source.exists() {
        anyhow::bail!("Skill path '{}' does not exist", skill_path);
    }
    
    let mut registry = REGISTRY.lock().unwrap();
    let skill_name = registry.install(&source)?;
    
    println!("✅ Installed skill '{}' to {}", skill_name, get_skills_dir().join(&skill_name).display());
    
    Ok(())
}

/// Search for skills matching a query
pub async fn search_skills(query: &str) -> anyhow::Result<()> {
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    
    static REGISTRY: Lazy<Mutex<SkillRegistry>> = Lazy::new(|| {
        Mutex::new(SkillRegistry::new(get_skills_dir()))
    });
    
    let mut registry = REGISTRY.lock().unwrap();
    registry.ensure_loaded()?;
    
    let results = registry.search(query);
    
    if results.is_empty() {
        println!("No skills found matching '{}'", query);
        return Ok(());
    }
    
    println!("\n🔍 Search results for '{}':\n", query);
    
    for result in results {
        println!("  {} (score: {:.1})", result.skill.name, result.score);
        println!("    {}", result.skill.description);
        if !result.matched_on.is_empty() {
            println!("    Matched: {}", result.matched_on.join(", "));
        }
        println!();
    }
    
    Ok(())
}

/// Enhanced natural language query with intelligent routing
pub async fn query_with_natural_language(query: &str, input: Option<&str>, output: Option<&str>) -> anyhow::Result<()> {
    use std::sync::Mutex;
    use once_cell::sync::Lazy;
    
    static ORCHESTRATOR: Lazy<Mutex<EnhancedOrchestrator>> = Lazy::new(|| {
        Mutex::new(EnhancedOrchestrator::new())
    });
    
    static REGISTRY: Lazy<Mutex<SkillRegistry>> = Lazy::new(|| {
        Mutex::new(SkillRegistry::new(get_skills_dir()))
    });
    
    let mut orchestrator = ORCHESTRATOR.lock().unwrap();
    let plan = orchestrator.route(query).await;
    
    println!("\n🧬 OpenLife Analysis");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if plan.is_multi_step() {
        println!("📋 Multi-step workflow detected:");
        for (i, skill) in plan.all_skills().iter().enumerate() {
            println!("   {}. {}", i + 1, skill);
        }
        println!();
        
        if let Some(ref msg) = plan.confirmation_message {
            println!("ℹ️  {}", msg);
        }
    } else {
        println!("🎯 Intent: {}", plan.primary_intent.display_name());
        println!("📊 Confidence: {:.0}%", plan.confidence * 100.0);
        println!("🔧 Method: {:?}", plan.method);
    }
    
    if plan.primary_intent == BioIntent::Unknown {
        println!();
        println!("❓ I couldn't determine the analysis type.");
        println!("   Try being more specific:");
        println!("   • \"Analyze my VCF for drug metabolism genes\"");
        println!("   • \"Find papers on CRISPR gene editing\"");
        println!("   • \"What drugs should I avoid with CYP2D6?\"");
        println!("   • \"Calculate diversity metrics from my genetic data\"");
        
        // Search for matching skills
        let mut registry = REGISTRY.lock().unwrap();
        registry.ensure_loaded()?;
        let matches = registry.search(query);
        
        if !matches.is_empty() {
            println!("\n   Related skills:");
            for m in matches.iter().take(3) {
                println!("   • {} - {}", m.skill.name, m.skill.description);
            }
        }
        
        return Ok(());
    }
    
    // Check registry for skills matching this intent
    let mut registry = REGISTRY.lock().unwrap();
    registry.ensure_loaded()?;
    
    // First try to find a skill matching the intent
    let skill = if let Some(s) = registry.get(plan.primary_intent.skill_name()) {
        Some(s.clone())
    } else {
        // Fall back to searching by intent
        registry.by_intent(plan.primary_intent.clone()).into_iter().next().cloned()
    };
    
    let skill = match skill {
        Some(s) => s,
        None => {
            println!("\n⚠️  No skill available for intent: {}", plan.primary_intent.display_name());
            println!("   Install a skill with: openlife bio install <path>");
            return Ok(());
        }
    };
    
    println!("\n📦 Routing to skill: {} v{}", skill.name, skill.version);
    
    // Check if input is required
    if input.is_none() && skill.requires_input() {
        let formats = skill.supported_formats();
        println!("\n📄 This analysis requires an input file.");
        println!("\n   Supported formats: {}", formats.join(", "));
        println!("\n   Usage:");
        println!("     openlife bio run {} --input <file> --output <dir>", skill.name);
        println!("\n   Demo data:");
        println!("     • skills/pharmgx-reporter/demo_patient.txt");
        println!("     • skills/nutrigx-advisor/synthetic_patient.csv");
        println!("     • examples/demo_populations.vcf");
        return Ok(());
    }
    
    // Execute the skill
    let executor = SkillExecutor::new(std::env::temp_dir());
    let input_path = input.map(|p| std::path::PathBuf::from(p));
    let output_path = output.map(|p| std::path::PathBuf::from(p));
    
    println!("\n🚀 Executing...");
    
    let result = executor.execute(&skill, input_path.as_deref(), output_path.as_deref())?;
    
    if result.success {
        println!("\n✅ Success ({}ms)", result.duration_ms);
        if !result.output.is_empty() {
            println!("\n{}", result.output);
        }
        if !result.output_files.is_empty() {
            println!("\n📄 Output files:");
            for file in &result.output_files {
                println!("   • {}", file.display());
            }
        }
    } else {
        eprintln!("\n❌ Execution failed");
        if let Some(error) = result.error {
            eprintln!("\nError: {}", error);
        }
    }
    
    Ok(())
}

/// List available predefined chains
pub fn list_chains() {
    use crate::bio::orchestrator::chain::ChainBuilder;
    
    let builder = ChainBuilder::new();
    let chains = builder.list_chains();
    
    println!("\n📋 Available Analysis Chains:\n");
    
    for chain in chains {
        println!("  {} - {}", chain.name, chain.description);
        println!("    Steps: {}", chain.steps.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(" → "));
        if let Some(time) = chain.estimated_time {
            println!("    Estimated time: {}s", time);
        }
        println!();
    }
}