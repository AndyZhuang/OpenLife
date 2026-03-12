//! Skill Loader - Parses and loads bioinformatics skill manifests

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Skill manifest loaded from SKILL.md or SKILL.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub tags: Vec<String>,
    pub inputs: Vec<SkillInput>,
    pub outputs: Vec<SkillOutput>,
    pub path: PathBuf,
    pub script_path: Option<PathBuf>,
    pub min_python: Option<String>,
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Skill input definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    pub name: String,
    #[serde(rename = "type")]
    pub input_type: String,
    #[serde(default)]
    pub format: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub required: bool,
}

/// Skill output definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOutput {
    pub name: String,
    #[serde(rename = "type")]
    pub output_type: String,
    #[serde(default)]
    pub format: Vec<String>,
    pub description: String,
}

impl SkillManifest {
    /// Check if this skill matches a given intent
    pub fn matches_intent(&self, intent: &str) -> bool {
        let intent_lower = intent.to_lowercase();

        // Check name
        if self.name.to_lowercase().contains(&intent_lower) {
            return true;
        }

        // Check tags
        for tag in &self.tags {
            if tag.to_lowercase().contains(&intent_lower) {
                return true;
            }
        }

        // Check description
        if self.description.to_lowercase().contains(&intent_lower) {
            return true;
        }

        false
    }

    /// Get the primary script to execute
    pub fn get_script(&self) -> Option<&Path> {
        self.script_path.as_deref()
    }

    /// Check if input is required
    pub fn requires_input(&self) -> bool {
        !self.inputs.is_empty() && self.inputs.iter().any(|i| i.required)
    }

    /// Get supported input formats
    pub fn supported_formats(&self) -> Vec<&str> {
        self.inputs
            .iter()
            .flat_map(|i| i.format.iter().map(|s| s.as_str()))
            .collect()
    }
}

/// Skill loader - parses SKILL.md and SKILL.toml files
pub struct SkillLoader {
    skills_dir: PathBuf,
}

impl SkillLoader {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self { skills_dir }
    }

    pub fn skills_dir(&self) -> &Path {
        &self.skills_dir
    }

    /// Load all skills from the skills directory
    pub fn load_all(&self) -> Vec<SkillManifest> {
        let mut skills = Vec::new();

        if !self.skills_dir.exists() {
            return skills;
        }

        if let Ok(entries) = std::fs::read_dir(&self.skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(skill) = self.load_skill(&path) {
                        skills.push(skill);
                    }
                }
            }
        }

        skills
    }

    /// Load a single skill from a directory
    pub fn load_skill(&self, skill_dir: &Path) -> Option<SkillManifest> {
        // Try SKILL.toml first
        let toml_path = skill_dir.join("SKILL.toml");
        if toml_path.exists() {
            if let Some(manifest) = self.parse_toml(&toml_path, skill_dir) {
                return Some(manifest);
            }
        }

        // Fall back to SKILL.md
        let md_path = skill_dir.join("SKILL.md");
        if md_path.exists() {
            if let Some(manifest) = self.parse_markdown(&md_path, skill_dir) {
                return Some(manifest);
            }
        }

        // No manifest found, create a minimal one from directory name
        self.create_minimal_manifest(skill_dir)
    }

    fn parse_toml(&self, path: &Path, skill_dir: &Path) -> Option<SkillManifest> {
        let content = std::fs::read_to_string(path).ok()?;
        let value: toml::Value = toml::from_str(&content).ok()?;

        let skill = value.get("skill")?;

        let name = skill
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();

        let version = skill
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string();

        let description = skill
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        let author = skill
            .get("author")
            .and_then(|a| a.as_str())
            .map(|s| s.to_string());

        let tags = skill
            .get("tags")
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let inputs = self.parse_toml_inputs(skill);
        let outputs = self.parse_toml_outputs(skill);

        Some(SkillManifest {
            name,
            version,
            description,
            author,
            license: None,
            tags,
            inputs,
            outputs,
            path: skill_dir.to_path_buf(),
            script_path: self.find_script(skill_dir),
            min_python: None,
            dependencies: vec![],
            metadata: HashMap::new(),
        })
    }

    fn parse_toml_inputs(&self, skill: &toml::Value) -> Vec<SkillInput> {
        let mut inputs = Vec::new();

        if let Some(inputs_section) = skill.get("inputs") {
            if let Some(params) = inputs_section.get("param").and_then(|p| p.as_array()) {
                for param in params {
                    if let Some(p) = param.as_table() {
                        inputs.push(SkillInput {
                            name: p
                                .get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("input")
                                .to_string(),
                            input_type: p
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("file")
                                .to_string(),
                            format: p
                                .get("format")
                                .and_then(|f| f.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .collect()
                                })
                                .unwrap_or_default(),
                            description: p
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                                .to_string(),
                            required: p.get("required").and_then(|r| r.as_bool()).unwrap_or(true),
                        });
                    }
                }
            }
        }

        inputs
    }

    fn parse_toml_outputs(&self, skill: &toml::Value) -> Vec<SkillOutput> {
        let mut outputs = Vec::new();

        if let Some(outputs_section) = skill.get("outputs") {
            if let Some(params) = outputs_section.get("param").and_then(|p| p.as_array()) {
                for param in params {
                    if let Some(p) = param.as_table() {
                        outputs.push(SkillOutput {
                            name: p
                                .get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("output")
                                .to_string(),
                            output_type: p
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("file")
                                .to_string(),
                            format: p
                                .get("format")
                                .and_then(|f| f.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .collect()
                                })
                                .unwrap_or_default(),
                            description: p
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                                .to_string(),
                        });
                    }
                }
            }
        }

        outputs
    }

    fn parse_markdown(&self, path: &Path, skill_dir: &Path) -> Option<SkillManifest> {
        let content = std::fs::read_to_string(path).ok()?;

        // Parse YAML frontmatter
        if !content.starts_with("---") {
            return None;
        }

        let after_first = &content[3..];
        let end = after_first.find("---")?;
        let frontmatter = after_first[..end].trim();

        let value: serde_yaml::Value = serde_yaml::from_str(frontmatter).ok()?;

        let get_str = |key: &str| -> String {
            value
                .get(key)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default()
        };

        let get_array = |key: &str| -> Vec<String> {
            value
                .get(key)
                .and_then(|v| v.as_sequence())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default()
        };

        let name = get_str("name");
        let version = get_str("version");
        let description = get_str("description");
        let author = Some(get_str("author")).filter(|s| !s.is_empty());
        let license = Some(get_str("license")).filter(|s| !s.is_empty());
        let tags = get_array("tags");

        // Parse inputs/outputs from frontmatter
        let inputs = self.parse_yaml_inputs(&value);
        let outputs = self.parse_yaml_outputs(&value);

        // Parse metadata
        let metadata = value
            .get("metadata")
            .and_then(|m| serde_json::to_value(m).ok())
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Some(SkillManifest {
            name,
            version,
            description,
            author,
            license,
            tags,
            inputs,
            outputs,
            path: skill_dir.to_path_buf(),
            script_path: self.find_script(skill_dir),
            min_python: Some(get_str("min_python")).filter(|s| !s.is_empty()),
            dependencies: get_array("dependencies"),
            metadata,
        })
    }

    fn parse_yaml_inputs(&self, value: &serde_yaml::Value) -> Vec<SkillInput> {
        let mut inputs = Vec::new();

        if let Some(inputs_list) = value.get("inputs").and_then(|i| i.as_sequence()) {
            for input in inputs_list {
                if let Some(obj) = input.as_mapping() {
                    let get_str = |key: &str| -> String {
                        obj.get(&serde_yaml::Value::String(key.to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_default()
                    };

                    let get_array = |key: &str| -> Vec<String> {
                        obj.get(&serde_yaml::Value::String(key.to_string()))
                            .and_then(|v| v.as_sequence())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_default()
                    };

                    inputs.push(SkillInput {
                        name: get_str("name"),
                        input_type: get_str("type"),
                        format: get_array("format"),
                        description: get_str("description"),
                        required: true,
                    });
                }
            }
        }

        inputs
    }

    fn parse_yaml_outputs(&self, value: &serde_yaml::Value) -> Vec<SkillOutput> {
        let mut outputs = Vec::new();

        if let Some(outputs_list) = value.get("outputs").and_then(|o| o.as_sequence()) {
            for output in outputs_list {
                if let Some(obj) = output.as_mapping() {
                    let get_str = |key: &str| -> String {
                        obj.get(&serde_yaml::Value::String(key.to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_default()
                    };

                    let get_array = |key: &str| -> Vec<String> {
                        obj.get(&serde_yaml::Value::String(key.to_string()))
                            .and_then(|v| v.as_sequence())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect()
                            })
                            .unwrap_or_default()
                    };

                    outputs.push(SkillOutput {
                        name: get_str("name"),
                        output_type: get_str("type"),
                        format: get_array("format"),
                        description: get_str("description"),
                    });
                }
            }
        }

        outputs
    }

    fn find_script(&self, skill_dir: &Path) -> Option<PathBuf> {
        let skill_name = skill_dir.file_name()?.to_str()?;

        // Common script name patterns
        let script_names = vec![
            format!("{}.py", skill_name.replace("-", "_")),
            format!("{}.py", skill_name),
            "main.py".to_string(),
            "run.py".to_string(),
            "skill.py".to_string(),
        ];

        for name in script_names {
            let path = skill_dir.join(&name);
            if path.exists() {
                return Some(path);
            }
        }

        // Find any Python script in the directory
        if let Ok(entries) = std::fs::read_dir(skill_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "py").unwrap_or(false) {
                    return Some(path);
                }
            }
        }

        None
    }

    fn create_minimal_manifest(&self, skill_dir: &Path) -> Option<SkillManifest> {
        let name = skill_dir.file_name()?.to_str()?.to_string();

        Some(SkillManifest {
            name: name.clone(),
            version: "0.1.0".to_string(),
            description: format!("Skill: {}", name),
            author: None,
            license: None,
            tags: vec![],
            inputs: vec![],
            outputs: vec![],
            path: skill_dir.to_path_buf(),
            script_path: self.find_script(skill_dir),
            min_python: None,
            dependencies: vec![],
            metadata: HashMap::new(),
        })
    }

    /// Install a skill from a source directory
    pub fn install_skill(&self, source_path: &Path) -> anyhow::Result<String> {
        if !source_path.exists() {
            anyhow::bail!("Source path does not exist: {}", source_path.display());
        }

        let skill_name = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid skill path"))?
            .to_string();

        let target = self.skills_dir.join(&skill_name);

        if target.exists() {
            anyhow::bail!("Skill '{}' is already installed", skill_name);
        }

        std::fs::create_dir_all(&self.skills_dir)?;
        copy_dir_all(source_path, &target)?;

        Ok(skill_name)
    }

    /// Uninstall a skill
    pub fn uninstall_skill(&self, skill_name: &str) -> anyhow::Result<()> {
        let skill_path = self.skills_dir.join(skill_name);

        if !skill_path.exists() {
            anyhow::bail!("Skill '{}' is not installed", skill_name);
        }

        std::fs::remove_dir_all(&skill_path)?;

        Ok(())
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_manifest_intent_matching() {
        let manifest = SkillManifest {
            name: "pharmgx-reporter".to_string(),
            version: "0.1.0".to_string(),
            description: "Pharmacogenomic analysis".to_string(),
            author: None,
            license: None,
            tags: vec!["pharmacogenomics".to_string(), "cpic".to_string()],
            inputs: vec![],
            outputs: vec![],
            path: PathBuf::from("/test"),
            script_path: None,
            min_python: None,
            dependencies: vec![],
            metadata: HashMap::new(),
        };

        assert!(manifest.matches_intent("pharmacogenomics"));
        assert!(manifest.matches_intent("pharmgx"));
        assert!(manifest.matches_intent("cpic"));
        assert!(!manifest.matches_intent("metagenomics"));
        assert!(!manifest.matches_intent("drug"));
    }

    #[test]
    fn test_skill_requires_input() {
        let manifest_with_input = SkillManifest {
            name: "test".to_string(),
            version: "0.1.0".to_string(),
            description: "Test".to_string(),
            author: None,
            license: None,
            tags: vec![],
            inputs: vec![SkillInput {
                name: "input".to_string(),
                input_type: "file".to_string(),
                format: vec!["vcf".to_string()],
                description: "Input file".to_string(),
                required: true,
            }],
            outputs: vec![],
            path: PathBuf::from("/test"),
            script_path: None,
            min_python: None,
            dependencies: vec![],
            metadata: HashMap::new(),
        };

        assert!(manifest_with_input.requires_input());
    }
}
