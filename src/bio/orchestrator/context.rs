//! Conversation Context - Memory and state management

use crate::bio::orchestrator::{BioIntent, IntentResult};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum number of interactions to remember
const MAX_HISTORY: usize = 10;

/// A single interaction in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// User's query
    pub query: String,
    /// Detected intent
    pub intent: BioIntent,
    /// Whether a skill was executed
    pub executed: bool,
    /// Skill that was run (if any)
    pub skill: Option<String>,
    /// Result summary
    pub result_summary: Option<String>,
}

/// Conversation context for multi-turn interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    /// Interaction history
    history: VecDeque<Interaction>,
    /// Current session ID
    session_id: String,
    /// Detected entities from conversation
    entities: EntityCache,
    /// User preferences learned from conversation
    preferences: UserPreferences,
}

impl ConversationContext {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(MAX_HISTORY),
            session_id: uuid::Uuid::new_v4().to_string(),
            entities: EntityCache::new(),
            preferences: UserPreferences::new(),
        }
    }

    pub fn add_interaction(&mut self, query: &str, result: &IntentResult) {
        let interaction = Interaction {
            query: query.to_string(),
            intent: result.primary.intent.clone(),
            executed: false,
            skill: Some(result.primary.intent.skill_name().to_string()),
            result_summary: None,
        };

        // Extract entities from query
        self.entities.extract_from_query(query);

        // Add to history, removing oldest if at capacity
        if self.history.len() >= MAX_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(interaction);
    }

    pub fn build_query_context(&self, query: &str) -> QueryContext {
        QueryContext {
            current_query: query.to_string(),
            previous_intents: self.history.iter().map(|i| i.intent.clone()).collect(),
            mentioned_files: self.entities.files.clone(),
            mentioned_genes: self.entities.genes.clone(),
            mentioned_drugs: self.entities.drugs.clone(),
            preferences: self.preferences.clone(),
        }
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.entities.clear();
        self.session_id = uuid::Uuid::new_v4().to_string();
    }

    pub fn history(&self) -> &VecDeque<Interaction> {
        &self.history
    }

    pub fn last_intent(&self) -> Option<BioIntent> {
        self.history.back().map(|i| i.intent.clone())
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for a single query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    pub current_query: String,
    pub previous_intents: Vec<BioIntent>,
    pub mentioned_files: Vec<String>,
    pub mentioned_genes: Vec<String>,
    pub mentioned_drugs: Vec<String>,
    pub preferences: UserPreferences,
}

impl QueryContext {
    pub fn new(query: &str) -> Self {
        Self {
            current_query: query.to_string(),
            previous_intents: vec![],
            mentioned_files: vec![],
            mentioned_genes: vec![],
            mentioned_drugs: vec![],
            preferences: UserPreferences::new(),
        }
    }

    pub fn has_file_reference(&self) -> bool {
        !self.mentioned_files.is_empty()
    }

    pub fn has_gene_reference(&self) -> bool {
        !self.mentioned_genes.is_empty()
    }

    pub fn get_file_extension(&self) -> Option<&str> {
        self.mentioned_files
            .first()
            .and_then(|f| f.rsplit('.').next())
    }

    pub fn suggest_intent_from_file(&self) -> Option<BioIntent> {
        let ext = self.get_file_extension()?;
        Some(match ext.to_lowercase().as_str() {
            "vcf" | "vcf.gz" => BioIntent::VariantAnnotation,
            "fastq" | "fq" | "fastq.gz" | "fq.gz" => BioIntent::SequenceAnalysis,
            "bam" | "cram" => BioIntent::SequenceAnalysis,
            "h5ad" | "rds" => BioIntent::SingleCell,
            "pdb" | "cif" => BioIntent::ProteinStructure,
            "txt" | "tsv" | "csv" => BioIntent::Pharmacogenomics,
            _ => return None,
        })
    }
}

/// Cache of detected entities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityCache {
    pub files: Vec<String>,
    pub genes: Vec<String>,
    pub drugs: Vec<String>,
    pub diseases: Vec<String>,
}

impl EntityCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extract_from_query(&mut self, query: &str) {
        self.extract_files(query);
        self.extract_genes(query);
        self.extract_drugs(query);
    }

    fn extract_files(&mut self, query: &str) {
        let extensions = [
            ".vcf", ".fastq", ".bam", ".h5ad", ".pdb", ".csv", ".tsv", ".txt", ".fq", ".cram",
            ".cif",
        ];

        for word in query.split_whitespace() {
            for ext in extensions {
                if word.to_lowercase().ends_with(ext) {
                    let file = word.trim_matches(|c: char| {
                        !c.is_alphanumeric() && c != '.' && c != '/' && c != '_'
                    });
                    if !file.is_empty() && !self.files.contains(&file.to_string()) {
                        self.files.push(file.to_string());
                    }
                }
            }
        }
    }

    fn extract_genes(&mut self, query: &str) {
        let common_genes = [
            "CYP2D6", "CYP2C19", "CYP2C9", "VKORC1", "SLCO1B1", "DPYD", "TPMT", "UGT1A1", "CYP3A5",
            "CYP2B6", "NUDT15", "CYP1A2", "BRCA1", "BRCA2", "TP53", "EGFR", "KRAS", "BRAF",
            "PIK3CA", "MTHFR", "F5", "F2", "HBB", "HFE",
        ];

        for gene in common_genes {
            if query.to_uppercase().contains(gene) {
                if !self.genes.contains(&gene.to_string()) {
                    self.genes.push(gene.to_string());
                }
            }
        }
    }

    fn extract_drugs(&mut self, query: &str) {
        let common_drugs = [
            "warfarin",
            "clopidogrel",
            "codeine",
            "tramadol",
            "tamoxifen",
            "simvastatin",
            "atorvastatin",
            "omeprazole",
            "pantoprazole",
            "fluorouracil",
            "capecitabine",
            "irinotecan",
            "azathioprine",
            "mercaptopurine",
            "thioguanine",
            "tacrolimus",
            "efavirenz",
        ];

        let query_lower = query.to_lowercase();
        for drug in common_drugs {
            if query_lower.contains(drug) {
                if !self.drugs.contains(&drug.to_string()) {
                    self.drugs.push(drug.to_string());
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.files.clear();
        self.genes.clear();
        self.drugs.clear();
        self.diseases.clear();
    }
}

/// User preferences learned from conversation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_output_format: Option<String>,
    pub include_reproducibility: bool,
    pub language: Option<String>,
    pub detail_level: DetailLevel,
}

impl UserPreferences {
    pub fn new() -> Self {
        Self {
            preferred_output_format: None,
            include_reproducibility: true,
            language: None,
            detail_level: DetailLevel::Standard,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum DetailLevel {
    Brief,
    #[default]
    Standard,
    Detailed,
}

mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
        pub fn to_string(&self) -> String {
            format!(
                "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
                rand_u32(),
                rand_u16(),
                rand_u16(),
                rand_u16(),
                rand_u64()
            )
        }
    }
    fn rand_u32() -> u32 {
        std::time::SystemTime::now().elapsed().unwrap().as_nanos() as u32
    }
    fn rand_u16() -> u16 {
        rand_u32() as u16
    }
    fn rand_u64() -> u64 {
        std::time::SystemTime::now().elapsed().unwrap().as_nanos() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ConversationContext::new();
        assert!(ctx.history.is_empty());
    }

    #[test]
    fn test_entity_extraction() {
        let mut cache = EntityCache::new();
        cache.extract_from_query("Analyze my sample.vcf file for CYP2D6 and warfarin interactions");

        assert!(cache.files.contains(&"sample.vcf".to_string()));
        assert!(cache.genes.contains(&"CYP2D6".to_string()));
        assert!(cache.drugs.contains(&"warfarin".to_string()));
    }

    #[test]
    fn test_file_extension_detection() {
        let ctx = QueryContext {
            current_query: "test".to_string(),
            previous_intents: vec![],
            mentioned_files: vec!["sample.vcf".to_string()],
            mentioned_genes: vec![],
            mentioned_drugs: vec![],
            preferences: UserPreferences::new(),
        };

        assert_eq!(ctx.get_file_extension(), Some("vcf"));
        assert_eq!(
            ctx.suggest_intent_from_file(),
            Some(BioIntent::VariantAnnotation)
        );
    }
}
