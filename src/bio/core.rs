pub mod repro;

use crate::config::BioConfig;
use std::path::PathBuf;

pub struct BioCore {
    pub config: BioConfig,
    #[allow(dead_code)]
    memory: Option<MemoryStore>,
    #[allow(dead_code)]
    sandbox: Option<Sandbox>,
}

pub struct MemoryStore {
    pub enabled: bool,
}

pub struct Sandbox {
    pub enabled: bool,
}

impl BioCore {
    pub fn new() -> anyhow::Result<Self> {
        let config = BioConfig::load()?;

        let memory = if config.memory_enabled {
            Some(MemoryStore { enabled: true })
        } else {
            None
        };

        let sandbox = if config.sandbox_enabled {
            Some(Sandbox { enabled: true })
        } else {
            None
        };

        tracing::info!(
            "BioCore initialized with config: skills_dir={:?}",
            config.skills_dir
        );

        Ok(Self {
            config,
            memory,
            sandbox,
        })
    }

    pub fn get_skills_dir(&self) -> PathBuf {
        self.config.skills_dir.clone()
    }

    pub fn init_skills_dir(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.config.skills_dir)?;

        let bundled_skills = std::path::Path::new("skills");
        if bundled_skills.exists() {
            tracing::info!("Bundled skills found, copying to user skills directory");
        }

        Ok(())
    }
}

impl Default for BioCore {
    fn default() -> Self {
        Self::new().expect("Failed to initialize BioCore")
    }
}
