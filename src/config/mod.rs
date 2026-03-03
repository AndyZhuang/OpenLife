use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioConfig {
    pub skills_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub llm_provider: LlmProvider,
    pub sandbox_enabled: bool,
    pub memory_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmProvider {
    pub name: String,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
}

impl Default for BioConfig {
    fn default() -> Self {
        let dirs = ProjectDirs::from("ai", "openlife", "openlife")
            .expect("Failed to get project directories");

        Self {
            skills_dir: dirs.data_dir().join("skills"),
            cache_dir: dirs.cache_dir().to_path_buf(),
            config_dir: dirs.config_dir().to_path_buf(),
            data_dir: dirs.data_dir().to_path_buf(),
            llm_provider: LlmProvider {
                name: "openai".to_string(),
                api_key: std::env::var("OPENAI_API_KEY").ok(),
                model: "gpt-4o-mini".to_string(),
                base_url: None,
            },
            sandbox_enabled: true,
            memory_enabled: true,
        }
    }
}

impl BioConfig {
    pub fn load() -> anyhow::Result<Self> {
        let config = Self::default();
        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        let config_path = self.config_dir.join("config.toml");

        let config_str = toml::to_string_pretty(self)?;

        std::fs::write(config_path, config_str)?;
        Ok(())
    }
}
