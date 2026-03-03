use std::path::PathBuf;

pub struct ReproducibilityBundle {
    pub output_dir: PathBuf,
    pub commands: Vec<String>,
    pub environment: String,
    pub checksums: String,
}

impl ReproducibilityBundle {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            commands: Vec::new(),
            environment: String::new(),
            checksums: String::new(),
        }
    }

    pub fn add_command(&mut self, cmd: &str) {
        self.commands.push(cmd.to_string());
    }

    pub fn set_environment(&mut self, env: &str) {
        self.environment = env.to_string();
    }

    pub fn add_checksum(&mut self, file: &str, checksum: &str) {
        self.checksums
            .push_str(&format!("{}  {}\n", checksum, file));
    }

    pub fn generate(self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.output_dir)?;

        let commands_path = self.output_dir.join("commands.sh");
        let mut content = String::from("#!/bin/bash\nset -e\n\n");
        for cmd in self.commands {
            content.push_str(&format!("{}\n", cmd));
        }
        std::fs::write(commands_path, content)?;

        let env_path = self.output_dir.join("environment.yml");
        std::fs::write(env_path, self.environment)?;

        let checksum_path = self.output_dir.join("checksums.sha256");
        std::fs::write(checksum_path, self.checksums)?;

        tracing::info!("Generated reproducibility bundle in {:?}", self.output_dir);
        Ok(())
    }
}
