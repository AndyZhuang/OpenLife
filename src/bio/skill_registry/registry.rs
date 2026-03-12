//! Skill Registry - Centralized skill management with caching and fast lookups

use super::loader::{SkillLoader, SkillManifest};
use crate::bio::orchestrator::BioIntent;
use std::collections::HashMap;
use std::path::PathBuf;

/// Skill search result with relevance score
#[derive(Debug, Clone)]
pub struct SkillSearchResult {
    pub skill: SkillManifest,
    pub score: f32,
    pub matched_on: Vec<String>,
}

/// Centralized skill registry with caching
pub struct SkillRegistry {
    /// Loaded skills indexed by name
    skills: HashMap<String, SkillManifest>,
    /// Skills indexed by tags for fast tag-based search
    by_tag: HashMap<String, Vec<String>>,
    /// Skills indexed by intent for routing
    by_intent: HashMap<BioIntent, Vec<String>>,
    /// Skill loader
    loader: SkillLoader,
    /// Whether the registry has been loaded
    loaded: bool,
}

impl SkillRegistry {
    /// Create a new skill registry
    pub fn new(skills_dir: PathBuf) -> Self {
        Self {
            skills: HashMap::new(),
            by_tag: HashMap::new(),
            by_intent: HashMap::new(),
            loader: SkillLoader::new(skills_dir),
            loaded: false,
        }
    }

    /// Load all skills from the skills directory
    pub fn load(&mut self) -> anyhow::Result<()> {
        if self.loaded {
            return Ok(());
        }

        let skills = self.loader.load_all();

        for skill in skills {
            self.index_skill(skill);
        }

        self.loaded = true;
        tracing::info!("Loaded {} skills into registry", self.skills.len());

        Ok(())
    }

    /// Ensure the registry is loaded
    pub fn ensure_loaded(&mut self) -> anyhow::Result<()> {
        self.load()
    }

    /// Index a skill in all lookup tables
    fn index_skill(&mut self, skill: SkillManifest) {
        let name = skill.name.clone();

        // Index by tag
        for tag in &skill.tags {
            self.by_tag
                .entry(tag.to_lowercase())
                .or_default()
                .push(name.clone());
        }

        // Index by intent (based on skill name patterns)
        if let Some(intent) = self.infer_intent(&skill) {
            self.by_intent.entry(intent).or_default().push(name.clone());
        }

        // Store skill
        self.skills.insert(name, skill);
    }

    /// Infer intent from skill name and tags
    fn infer_intent(&self, skill: &SkillManifest) -> Option<BioIntent> {
        let name_lower = skill.name.to_lowercase();
        let tags_lower: Vec<String> = skill
            .tags
            .iter()
            .map(|t| t.to_lowercase())
            .collect();

        // Map skill names/tags to intents
        // Map skill names/tags to intents
        match () {
            _ if name_lower.contains("pharmgx")
                || name_lower.contains("pharmacogen")
                || tags_lower.iter().any(|t| t.contains("pharmacogenomic")) =>
            {
                Some(BioIntent::Pharmacogenomics)
            }
            _ if name_lower.contains("ancestry") || name_lower.contains("population") => {
                Some(BioIntent::Ancestry)
            }
            _ if name_lower.contains("diversity")
                || name_lower.contains("equity")
                || name_lower.contains("fst") =>
            {
                Some(BioIntent::Diversity)
            }
            _ if name_lower.contains("nutri")
                || tags_lower.iter().any(|t| t.contains("nutrition")) =>
            {
                Some(BioIntent::Nutrition)
            }
            _ if name_lower.contains("vcf")
                || name_lower.contains("variant")
                || name_lower.contains("annot") =>
            {
                Some(BioIntent::VariantAnnotation)
            }
            _ if name_lower.contains("literature")
                || name_lower.contains("pubmed")
                || name_lower.contains("paper") =>
            {
                Some(BioIntent::Literature)
            }
            _ if name_lower.contains("scrna")
                || name_lower.contains("single-cell")
                || name_lower.contains("singlecell") =>
            {
                Some(BioIntent::SingleCell)
            }
            _ if name_lower.contains("protein")
                || name_lower.contains("structure")
                || name_lower.contains("alphafold") =>
            {
                Some(BioIntent::ProteinStructure)
            }
            _ if name_lower.contains("repro")
                || name_lower.contains("nextflow")
                || name_lower.contains("workflow") =>
            {
                Some(BioIntent::Reproducibility)
            }
            _ if name_lower.contains("seq")
                || name_lower.contains("alignment")
                || name_lower.contains("bam")
                || name_lower.contains("fastq") =>
            {
                Some(BioIntent::SequenceAnalysis)
            }
            _ if name_lower.contains("metagenom") || name_lower.contains("microbiome") => {
                Some(BioIntent::Metagenomics)
            }
            _ if name_lower.contains("chem")
                || name_lower.contains("mol")
                || name_lower.contains("rdkit")
                || name_lower.contains("drug") =>
            {
                Some(BioIntent::Cheminformatics)
            }
            _ if name_lower.contains("clinical") || name_lower.contains("trial") => {
                Some(BioIntent::Clinical)
            }
            _ if name_lower.contains("vision")
                || name_lower.contains("image")
                || name_lower.contains("segment") =>
            {
                Some(BioIntent::Vision)
            }
            _ if name_lower.contains("stat")
                || name_lower.contains("data")
                || name_lower.contains("ml")
                || name_lower.contains("machine") =>
            {
                Some(BioIntent::DataScience)
            }
            _ if name_lower.contains("lab") || name_lower.contains("automat") => {
                Some(BioIntent::LabAutomation)
            }
            _ if name_lower.contains("gwas")
                || name_lower.contains("database")
                || name_lower.contains("api") =>
            {
                Some(BioIntent::DatabaseQuery)
            }
            _ => None,
        }
    }

    /// Get a skill by name
    pub fn get(&self, name: &str) -> Option<&SkillManifest> {
        self.skills.get(name)
    }

    /// Get all skills
    pub fn all(&self) -> impl Iterator<Item = &SkillManifest> {
        self.skills.values()
    }

    /// Get skill count
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Search for skills by query
    pub fn search(&self, query: &str) -> Vec<SkillSearchResult> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<SkillSearchResult> = Vec::new();

        for skill in self.skills.values() {
            let mut score = 0.0f32;
            let mut matched_on = Vec::new();

            // Exact name match
            if skill.name.to_lowercase() == query_lower {
                score += 1.0;
                matched_on.push(format!("name:{}", skill.name));
            }
            // Name contains query
            else if skill.name.to_lowercase().contains(&query_lower) {
                score += 0.8;
                matched_on.push(format!("name:{}", skill.name));
            }

            // Tag match
            for tag in &skill.tags {
                if tag.to_lowercase().contains(&query_lower) {
                    score += 0.5;
                    matched_on.push(format!("tag:{}", tag));
                }
            }

            // Description match
            if skill.description.to_lowercase().contains(&query_lower) {
                score += 0.3;
                matched_on.push("description".to_string());
            }

            if score > 0.0 {
                results.push(SkillSearchResult {
                    skill: skill.clone(),
                    score,
                    matched_on,
                });
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Get skills by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&SkillManifest> {
        self.by_tag
            .get(&tag.to_lowercase())
            .map(|names| names.iter().filter_map(|n| self.skills.get(n)).collect())
            .unwrap_or_default()
    }

    /// Get skills by intent
    pub fn by_intent(&self, intent: BioIntent) -> Vec<&SkillManifest> {
        self.by_intent
            .get(&intent)
            .map(|names| names.iter().filter_map(|n| self.skills.get(n)).collect())
            .unwrap_or_default()
    }

    /// Get skill for a specific intent (primary skill)
    pub fn get_primary_skill(&self, intent: BioIntent) -> Option<&SkillManifest> {
        self.by_intent(intent).into_iter().next()
    }

    /// Install a new skill
    pub fn install(&mut self, source_path: &PathBuf) -> anyhow::Result<String> {
        let skill_name = self.loader.install_skill(source_path)?;

        // Reload the newly installed skill
        let skill_path = self.loader.skills_dir().join(&skill_name);
        if let Some(skill) = self.loader.load_skill(&skill_path) {
            self.index_skill(skill);
        }

        Ok(skill_name)
    }

    /// Uninstall a skill
    pub fn uninstall(&mut self, skill_name: &str) -> anyhow::Result<()> {
        // Remove from indices
        if let Some(skill) = self.skills.remove(skill_name) {
            for tag in &skill.tags {
                if let Some(names) = self.by_tag.get_mut(&tag.to_lowercase()) {
                    names.retain(|n| n != skill_name);
                }
            }
        }

        self.loader.uninstall_skill(skill_name)
    }

    /// Get all available tags
    pub fn available_tags(&self) -> Vec<&String> {
        let mut tags: Vec<_> = self.by_tag.keys().collect();
        tags.sort();
        tags
    }

    /// Get skill names
    pub fn skill_names(&self) -> Vec<&String> {
        self.skills.keys().collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        let skills_dir = crate::bio::get_skills_dir();
        Self::new(skills_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_registry_search() {
        let mut registry = SkillRegistry::new(PathBuf::from("/tmp/test-skills"));

        // Add a mock skill
        let skill = SkillManifest {
            name: "pharmgx-reporter".to_string(),
            version: "0.1.0".to_string(),
            description: "Pharmacogenomic analysis".to_string(),
            author: None,
            license: None,
            tags: vec!["pharmacogenomics".to_string()],
            inputs: vec![],
            outputs: vec![],
            path: PathBuf::from("/test"),
            script_path: None,
            min_python: None,
            dependencies: vec![],
            metadata: HashMap::new(),
        };

        registry.index_skill(skill);

        let results = registry.search("pharmgx");
        assert!(!results.is_empty());
        assert_eq!(results[0].skill.name, "pharmgx-reporter");
    }

    #[test]
    fn test_intent_inference() {
        let registry = SkillRegistry::new(PathBuf::from("/tmp/test"));

        let skill = SkillManifest {
            name: "vcf-annotator".to_string(),
            version: "0.1.0".to_string(),
            description: "Annotate VCF files".to_string(),
            author: None,
            license: None,
            tags: vec!["genomics".to_string()],
            inputs: vec![],
            outputs: vec![],
            path: PathBuf::from("/test"),
            script_path: None,
            min_python: None,
            dependencies: vec![],
            metadata: HashMap::new(),
        };

        let intent = registry.infer_intent(&skill);
        assert_eq!(intent, Some(BioIntent::VariantAnnotation));
    }
}
