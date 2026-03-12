//! Skill Executor - Runs bioinformatics skills with proper I/O handling

use super::loader::SkillManifest;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Result of skill execution
#[derive(Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub output_files: Vec<PathBuf>,
}

/// Skill executor - runs skills with proper input/output handling
pub struct SkillExecutor {
    /// Working directory for execution
    work_dir: PathBuf,
    /// Python executable path
    python_path: String,
}

impl SkillExecutor {
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            work_dir,
            python_path: "python3".to_string(),
        }
    }

    /// Set custom Python path
    pub fn with_python(mut self, python_path: &str) -> Self {
        self.python_path = python_path.to_string();
        self
    }

    /// Execute a skill with given inputs
    pub fn execute(
        &self,
        skill: &SkillManifest,
        input: Option<&Path>,
        output: Option<&Path>,
    ) -> anyhow::Result<ExecutionResult> {
        let script = skill
            .get_script()
            .ok_or_else(|| anyhow::anyhow!("No script found for skill: {}", skill.name))?;

        let start = Instant::now();

        let mut cmd = Command::new(&self.python_path);
        cmd.arg(script);

        // Add input argument if provided
        if let Some(input_path) = input {
            cmd.arg("--input").arg(input_path);
        }

        // Add output argument if provided
        if let Some(output_path) = output {
            cmd.arg("--output").arg(output_path);
        }

        // Set working directory to skill directory
        cmd.current_dir(&skill.path);

        // Execute
        let cmd_output = cmd.output()?;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Collect output files
        let output_files = if let Some(output_dir) = output {
            self.collect_output_files(output_dir)
        } else {
            vec![]
        };

        Ok(ExecutionResult {
            success: cmd_output.status.success(),
            output: String::from_utf8_lossy(&cmd_output.stdout).to_string(),
            error: if cmd_output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&cmd_output.stderr).to_string())
            },
            duration_ms,
            output_files,
        })
    }

    /// Execute a skill with JSON arguments
    pub fn execute_with_args(
        &self,
        skill: &SkillManifest,
        args: &serde_json::Value,
    ) -> anyhow::Result<ExecutionResult> {
        let script = skill
            .get_script()
            .ok_or_else(|| anyhow::anyhow!("No script found for skill: {}", skill.name))?;

        let start = Instant::now();

        let mut cmd = Command::new(&self.python_path);
        cmd.arg(script);

        // Convert JSON args to command line arguments
        if let Some(obj) = args.as_object() {
            for (key, value) in obj {
                cmd.arg(format!("--{}", key));
                match value {
                    serde_json::Value::String(s) => cmd.arg(s),
                    serde_json::Value::Number(n) => cmd.arg(n.to_string()),
                    serde_json::Value::Bool(b) => cmd.arg(b.to_string()),
                    _ => cmd.arg(value.to_string()),
                };
            }
        }

        cmd.current_dir(&skill.path);

        let output = cmd.output()?;
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ExecutionResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            duration_ms,
            output_files: vec![],
        })
    }

    /// Execute a skill chain
    pub fn execute_chain(
        &self,
        skills: &[SkillManifest],
        initial_input: Option<&Path>,
        final_output: Option<&Path>,
    ) -> anyhow::Result<Vec<ExecutionResult>> {
        let mut results = Vec::new();
        let mut current_input = initial_input.map(|p| p.to_path_buf());

        for (i, skill) in skills.iter().enumerate() {
            let is_last = i == skills.len() - 1;

            // Determine output for this step
            let step_output = if is_last {
                final_output.map(|p| p.to_path_buf())
            } else {
                // Create temp directory for intermediate output
                let temp_dir = self.work_dir.join(format!("step_{}", i));
                std::fs::create_dir_all(&temp_dir)?;
                Some(temp_dir)
            };

            let result = self.execute(skill, current_input.as_deref(), step_output.as_deref())?;

            // Update input for next step
            if let Some(ref output_dir) = step_output {
                current_input = self.find_output_file(output_dir);
            }

            results.push(result);

            // Stop chain on failure
            if !results.last().map(|r| r.success).unwrap_or(false) {
                break;
            }
        }

        Ok(results)
    }

    /// Collect all output files from a directory
    fn collect_output_files(&self, output_dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if output_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(output_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        files.push(path);
                    }
                }
            }
        }

        files
    }

    /// Find the primary output file from a directory
    fn find_output_file(&self, output_dir: &Path) -> Option<PathBuf> {
        if !output_dir.exists() {
            return None;
        }

        let entries: Vec<_> = std::fs::read_dir(output_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .collect();

        // Prefer markdown reports
        for entry in &entries {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                return Some(path);
            }
        }

        // Then any file
        for entry in &entries {
            let path = entry.path();
            if path.is_file() {
                return Some(path);
            }
        }

        None
    }

    /// Validate skill dependencies
    pub fn validate_dependencies(&self, skill: &SkillManifest) -> anyhow::Result<Vec<String>> {
        let mut missing = Vec::new();

        // Check Python version if specified
        if let Some(ref min_python) = skill.min_python {
            let output = Command::new(&self.python_path).arg("--version").output()?;

            let version_str = String::from_utf8_lossy(&output.stdout);
            if !version_str.contains(min_python) {
                // Simple version check - could be more sophisticated
                missing.push(format!(
                    "Python {} required, found: {}",
                    min_python,
                    version_str.trim()
                ));
            }
        }

        // Check dependencies
        for dep in &skill.dependencies {
            let check = Command::new(&self.python_path)
                .args(["-c", &format!("import {}", dep)])
                .output();

            if check.is_err() || !check.unwrap().status.success() {
                missing.push(format!("Python package '{}' not installed", dep));
            }
        }

        Ok(missing)
    }
}

impl Default for SkillExecutor {
    fn default() -> Self {
        Self::new(std::env::temp_dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = SkillExecutor::new(PathBuf::from("/tmp"));
        assert_eq!(executor.python_path, "python3");
    }

    #[test]
    fn test_executor_with_python() {
        let executor = SkillExecutor::new(PathBuf::from("/tmp")).with_python("/usr/bin/python3.11");
        assert_eq!(executor.python_path, "/usr/bin/python3.11");
    }
}
