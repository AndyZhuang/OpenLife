pub mod orchestrator;
pub mod core;

pub use orchestrator::BioIntent;

pub const OPENLIFE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_skills_dir() -> std::path::PathBuf {
    let dirs = directories::ProjectDirs::from("ai", "openlife", "openlife")
        .expect("Failed to get project directories");
    dirs.data_dir().join("skills")
}

pub async fn list_skills() -> anyhow::Result<()> {
    let skills_dir = get_skills_dir();
    
    if !skills_dir.exists() {
        println!("No skills installed. Use `openlife bio install <path>` to install a skill.");
        return Ok(());
    }

    println!("Available Bio-Skills:\n");
    
    for entry in std::fs::read_dir(&skills_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let skill_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            let info = load_skill_info(&path);
            println!("  {} - {}", skill_name, info.description);
        }
    }
    
    Ok(())
}

struct SkillInfo {
    name: String,
    version: String,
    description: String,
    author: Option<String>,
    tags: Vec<String>,
}

fn load_skill_info(skill_dir: &std::path::PathBuf) -> SkillInfo {
    let toml_path = skill_dir.join("SKILL.toml");
    
    if toml_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&toml_path) {
            if let Ok(info) = toml::from_str::<toml::Value>(&content) {
                return SkillInfo {
                    name: info.get("skill").and_then(|s| s.get("name")).and_then(|n| n.as_str()).unwrap_or("unknown").to_string(),
                    version: info.get("skill").and_then(|s| s.get("version")).and_then(|v| v.as_str()).unwrap_or("0.1.0").to_string(),
                    description: info.get("skill").and_then(|s| s.get("description")).and_then(|d| d.as_str()).unwrap_or("").to_string(),
                    author: info.get("skill").and_then(|s| s.get("author")).and_then(|a| a.as_str()).map(|s| s.to_string()),
                    tags: info.get("skill").and_then(|s| s.get("tags")).and_then(|t| t.as_array()).map(|arr| {
                        arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect()
                    }).unwrap_or_default(),
                };
            }
        }
    }
    
    let md_path = skill_dir.join("SKILL.md");
    if md_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&md_path) {
            if content.starts_with("---") {
                let after_first = &content[3..];
                if let Some(end) = after_first.find("---") {
                    let frontmatter = after_first[..end].trim();
                    if let Ok(info) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
                        let get_str = |key: &str| -> String {
                            info.get(key)
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_default()
                        };
                        let get_array = |key: &str| -> Vec<String> {
                            info.get(key)
                                .and_then(|v| v.as_sequence())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .collect()
                                })
                                .unwrap_or_default()
                        };
                        return SkillInfo {
                            name: get_str("name"),
                            version: get_str("version"),
                            description: get_str("description"),
                            author: Some(get_str("author")).filter(|s| !s.is_empty()),
                            tags: get_array("tags"),
                        };
                    }
                }
            }
            let lines: Vec<&str> = content.lines().collect();
            let mut desc = String::new();
            let mut found_desc = false;
            for line in lines {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                desc = trimmed.to_string();
                found_desc = true;
                break;
            }
            if found_desc {
                return SkillInfo {
                    name: skill_dir.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    version: "0.1.0".to_string(),
                    description: desc,
                    author: None,
                    tags: vec![],
                };
            }
        }
    }
    
    SkillInfo {
        name: skill_dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        version: "0.1.0".to_string(),
        description: "No description available".to_string(),
        author: None,
        tags: vec![],
    }
}

pub async fn show_skill_info(skill_name: &str) -> anyhow::Result<()> {
    let skills_dir = get_skills_dir();
    let skill_path = skills_dir.join(skill_name);
    
    if !skill_path.exists() {
        anyhow::bail!("Skill '{}' not found. Use `openlife bio list` to see available skills.", skill_name);
    }
    
    let info = load_skill_info(&skill_path);
    
    println!("\n=== {} ===", info.name);
    println!("Version: {}", info.version);
    println!("Description: {}", info.description);
    
    if let Some(author) = info.author {
        println!("Author: {}", author);
    }
    
    if !info.tags.is_empty() {
        println!("Tags: {:?}", info.tags);
    }
    
    Ok(())
}

pub async fn run_skill(skill_name: &str, input: Option<&str>, output: Option<&str>) -> anyhow::Result<()> {
    use std::process::Command;
    
    let skills_dir = get_skills_dir();
    let skill_path = skills_dir.join(skill_name);
    
    if !skill_path.exists() {
        anyhow::bail!("Skill '{}' not found", skill_name);
    }
    
    let script_names = vec![
        format!("{}.py", skill_name.replace("-", "_")),
        format!("{}.py", skill_name),
    ];
    
    let mut script_path = None;
    for name in script_names {
        let path = skill_path.join(name);
        if path.exists() {
            script_path = Some(path);
            break;
        }
    }
    
    let script_path = match script_path {
        Some(p) => p,
        None => {
            if let Ok(entries) = std::fs::read_dir(&skill_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "py").unwrap_or(false) {
                        script_path = Some(path);
                        break;
                    }
                }
            }
            if let Some(p) = script_path {
                p
            } else {
                anyhow::bail!("No Python script found in skill directory");
            }
        }
    };
    
    let mut cmd = Command::new("python3");
    cmd.arg(&script_path);
    
    if let Some(input_file) = input {
        cmd.arg("--input").arg(input_file);
    }
    
    if let Some(output_dir) = output {
        cmd.arg("--output").arg(output_dir);
    }
    
    println!("Running skill: {}", skill_name);
    
    let output = cmd.output()?;
    
    if output.status.success() {
        println!("\n{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Skill execution failed");
    }
    
    Ok(())
}

pub async fn install_skill(skill_path: &str) -> anyhow::Result<()> {
    let source = std::path::PathBuf::from(skill_path);
    let skills_dir = get_skills_dir();
    
    if !source.exists() {
        anyhow::bail!("Skill path '{}' does not exist", skill_path);
    }
    
    let skill_name = source.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid skill path"))?;
    
    let target = skills_dir.join(skill_name);
    
    if target.exists() {
        anyhow::bail!("Skill '{}' is already installed", skill_name);
    }
    
    std::fs::create_dir_all(&skills_dir)?;
    copy_dir_all(&source, &target)?;
    
    println!("Installed skill '{}' to {}", skill_name, target.display());
    
    Ok(())
}

fn copy_dir_all(src: &std::path::PathBuf, dst: &std::path::PathBuf) -> anyhow::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    
    Ok(())
}

pub async fn query_with_natural_language(query: &str, input: Option<&str>, output: Option<&str>) -> anyhow::Result<()> {
    let intent = BioIntent::from_query(query);
    
    println!("Detected intent: {:?}", intent);
    
    let skill_name = match intent {
        BioIntent::Pharmacogenomics => "pharmgx-reporter",
        BioIntent::Ancestry => "ancestry-pca",
        BioIntent::Diversity => "equity-scorer",
        BioIntent::Nutrition => "nutrigx-advisor",
        BioIntent::VariantAnnotation => "vcf-annotator",
        BioIntent::Literature => "lit-synthesizer",
        BioIntent::SingleCell => "scrna-orchestrator",
        BioIntent::ProteinStructure => "struct-predictor",
        BioIntent::Reproducibility => "repro-enforcer",
        BioIntent::SequenceAnalysis => "seq-wrangler",
        BioIntent::DatabaseQuery => "bio-orchestrator",
        BioIntent::SemanticAnalysis => "semantic-sim",
        BioIntent::Metagenomics => "metagenomics",
        BioIntent::Unknown => {
            println!("Could not determine which skill to use. Try being more specific.");
            return Ok(());
        }
    };
    
    println!("Routing to skill: {}", skill_name);
    
    run_skill(skill_name, input, output).await
}
