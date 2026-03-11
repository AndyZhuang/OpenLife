use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLifeConfig {
    pub version: String,
    pub llm: LlmConfig,
    pub bio: BioConfig,
    pub gateway: GatewayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioConfig {
    pub skills_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub auto_update: bool,
    pub local_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub host: String,
    pub port: u16,
    pub require_pairing: bool,
}

impl Default for OpenLifeConfig {
    fn default() -> Self {
        let dirs = ProjectDirs::from("ai", "openlife", "openlife")
            .expect("Failed to get project directories");

        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            llm: LlmConfig {
                provider: "openai".to_string(),
                model: "gpt-4o-mini".to_string(),
                api_key: std::env::var("OPENAI_API_KEY").ok(),
                base_url: None,
                temperature: 0.7,
            },
            bio: BioConfig {
                skills_dir: dirs.data_dir().join("skills"),
                cache_dir: dirs.cache_dir().to_path_buf(),
                auto_update: true,
                local_only: true,
            },
            gateway: GatewayConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                require_pairing: true,
            },
        }
    }
}

impl OpenLifeConfig {
    pub fn config_dir() -> PathBuf {
        let dirs = ProjectDirs::from("ai", "openlife", "openlife")
            .expect("Failed to get project directories");
        dirs.config_dir().to_path_buf()
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: OpenLifeConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(Self::config_dir())?;
        let config_str = toml::to_string_pretty(self)?;
        std::fs::write(Self::config_path(), config_str)?;
        Ok(())
    }

    pub fn set_api_key(&mut self, provider: &str, api_key: &str) {
        self.llm.provider = provider.to_string();
        self.llm.api_key = Some(api_key.to_string());

        match provider {
            "openai" => {
                self.llm.model = "gpt-4o-mini".to_string();
                self.llm.base_url = None;
            }
            "anthropic" => {
                self.llm.model = "claude-sonnet-4-20250514".to_string();
                self.llm.base_url = None;
            }
            "openrouter" => {
                self.llm.model = "anthropic/claude-sonnet-4".to_string();
                self.llm.base_url = Some("https://openrouter.ai/api/v1".to_string());
            }
            "ollama" => {
                self.llm.model = "llama3.2".to_string();
                self.llm.base_url = Some("http://localhost:11434/v1".to_string());
            }
            _ => {}
        }
    }
}
