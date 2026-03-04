pub mod repro;

#[allow(dead_code)]
pub struct BioCore;

#[allow(dead_code)]
impl BioCore {
    pub fn new() -> anyhow::Result<Self> {
        tracing::info!("BioCore placeholder - using skills from ~/.local/share/openlife/skills");
        Ok(Self)
    }
}
