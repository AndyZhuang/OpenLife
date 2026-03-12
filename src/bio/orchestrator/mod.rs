//! Enhanced Bio Orchestrator for OpenLife

pub mod router;
pub mod chain;
pub mod context;
pub mod llm;

pub use router::{IntentRouter, IntentResult, IntentMatch, MatchMethod};
pub use chain::{SkillChain, ChainStep, ChainBuilder, RoutingPlan};
pub use context::{ConversationContext, QueryContext};
pub use llm::{LlmClassifier, LlmConfig};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BioIntent {
    Pharmacogenomics,
    Ancestry,
    Diversity,
    Nutrition,
    VariantAnnotation,
    Literature,
    SingleCell,
    ProteinStructure,
    Reproducibility,
    SequenceAnalysis,
    DatabaseQuery,
    SemanticAnalysis,
    Metagenomics,
    Cheminformatics,
    Clinical,
    Vision,
    DataScience,
    LabAutomation,
    Unknown,
    Multi(Vec<BioIntent>),
}

impl BioIntent {
    pub fn skill_name(&self) -> &'static str {
        match self {
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
            BioIntent::Cheminformatics => "labclaw/pharma",
            BioIntent::Clinical => "labclaw/med",
            BioIntent::Vision => "labclaw/vision",
            BioIntent::DataScience => "labclaw/general",
            BioIntent::LabAutomation => "labclaw/bio",
            BioIntent::Unknown | BioIntent::Multi(_) => "",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            BioIntent::Pharmacogenomics => "Pharmacogenomics Analysis",
            BioIntent::Ancestry => "Ancestry Analysis",
            BioIntent::Diversity => "Diversity & Equity Scoring",
            BioIntent::Nutrition => "Nutrigenomics",
            BioIntent::VariantAnnotation => "Variant Annotation",
            BioIntent::Literature => "Literature Synthesis",
            BioIntent::SingleCell => "Single-Cell Analysis",
            BioIntent::ProteinStructure => "Protein Structure Prediction",
            BioIntent::Reproducibility => "Reproducibility Export",
            BioIntent::SequenceAnalysis => "Sequence Analysis",
            BioIntent::DatabaseQuery => "Database Query",
            BioIntent::SemanticAnalysis => "Semantic Analysis",
            BioIntent::Metagenomics => "Metagenomics Profiling",
            BioIntent::Cheminformatics => "Cheminformatics",
            BioIntent::Clinical => "Clinical Analysis",
            BioIntent::Vision => "Computer Vision",
            BioIntent::DataScience => "Data Science",
            BioIntent::LabAutomation => "Lab Automation",
            BioIntent::Unknown => "Unknown",
            BioIntent::Multi(_) => "Multi-Intent",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            BioIntent::Pharmacogenomics => "Analyze genetic variants affecting drug metabolism and response",
            BioIntent::Ancestry => "Determine population ancestry and genetic origins",
            BioIntent::Diversity => "Calculate diversity metrics (HEIM, FST, heterozygosity)",
            BioIntent::Nutrition => "Analyze genetic variants affecting nutrition and diet",
            BioIntent::VariantAnnotation => "Annotate VCF variants with VEP, ClinVar, gnomAD",
            BioIntent::Literature => "Search and synthesize biomedical literature",
            BioIntent::SingleCell => "Analyze single-cell RNA sequencing data",
            BioIntent::ProteinStructure => "Predict or analyze protein 3D structures",
            BioIntent::Reproducibility => "Export reproducible analysis packages",
            BioIntent::SequenceAnalysis => "Process FASTQ/BAM files, QC, alignment",
            BioIntent::DatabaseQuery => "Query bioinformatics databases",
            BioIntent::SemanticAnalysis => "Analyze research gaps and semantic similarity",
            BioIntent::Metagenomics => "Profile microbiome from 16S or shotgun data",
            BioIntent::Cheminformatics => "Molecular analysis and drug discovery",
            BioIntent::Clinical => "Clinical trial analysis and precision medicine",
            BioIntent::Vision => "Image and video analysis for biology",
            BioIntent::DataScience => "Statistical analysis and ML",
            BioIntent::LabAutomation => "Laboratory automation and protocols",
            BioIntent::Unknown => "Unable to determine intent",
            BioIntent::Multi(_) => "Multiple analysis types requested",
        }
    }
    
    pub fn requires_input(&self) -> bool {
        matches!(
            self,
            BioIntent::Pharmacogenomics
                | BioIntent::Ancestry
                | BioIntent::Diversity
                | BioIntent::Nutrition
                | BioIntent::VariantAnnotation
                | BioIntent::SingleCell
                | BioIntent::SequenceAnalysis
                | BioIntent::Metagenomics
                | BioIntent::ProteinStructure
        )
    }
    
    pub fn supported_formats(&self) -> Vec<&'static str> {
        match self {
            BioIntent::Pharmacogenomics => vec!["23andme", "ancestrydna", "tsv", "txt"],
            BioIntent::VariantAnnotation => vec!["vcf", "vcf.gz"],
            BioIntent::SingleCell => vec!["h5ad", "rds"],
            BioIntent::SequenceAnalysis => vec!["fastq", "fastq.gz", "fq", "bam", "cram"],
            BioIntent::ProteinStructure => vec!["pdb", "cif", "fasta"],
            BioIntent::Diversity | BioIntent::Ancestry => vec!["vcf", "vcf.gz", "csv", "tsv"],
            BioIntent::Metagenomics => vec!["fastq", "fastq.gz", "fasta"],
            _ => vec![],
        }
    }
}

impl std::fmt::Display for BioIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

pub struct EnhancedOrchestrator {
    keyword_router: router::IntentRouter,
    llm_classifier: Option<llm::LlmClassifier>,
    chain_builder: chain::ChainBuilder,
    context: context::ConversationContext,
}

impl EnhancedOrchestrator {
    pub fn new() -> Self {
        Self {
            keyword_router: router::IntentRouter::new(),
            llm_classifier: None,
            chain_builder: chain::ChainBuilder::new(),
            context: context::ConversationContext::new(),
        }
    }
    
    pub fn with_llm(mut self, config: llm::LlmConfig) -> Self {
        self.llm_classifier = Some(llm::LlmClassifier::new(config));
        self
    }
    
    pub async fn route(&mut self, query: &str) -> RoutingPlan {
        let query_context = self.context.build_query_context(query);
        let keyword_result = self.keyword_router.route(query);
        
        if keyword_result.primary.confidence > 0.85 {
            self.context.add_interaction(query, &keyword_result);
            return self.build_plan(&keyword_result, &query_context);
        }
        
        if let Some(ref llm) = self.llm_classifier {
            if let Ok(llm_result) = llm.classify(query, &query_context).await {
                if llm_result.primary.confidence > keyword_result.primary.confidence {
                    self.context.add_interaction(query, &llm_result);
                    return self.build_plan(&llm_result, &query_context);
                }
            }
        }
        
        self.context.add_interaction(query, &keyword_result);
        self.build_plan(&keyword_result, &query_context)
    }
    
    fn build_plan(&self, result: &IntentResult, context: &QueryContext) -> RoutingPlan {
        if !result.alternatives.is_empty() && result.primary.confidence > 0.5 {
            let intents: Vec<BioIntent> = std::iter::once(result.primary.intent.clone())
                .chain(result.alternatives.iter().map(|m| m.intent.clone()))
                .filter(|i| *i != BioIntent::Unknown)
                .collect();
            
            if intents.len() > 1 {
                if let Some(chain) = self.chain_builder.get_chain(&intents) {
                    return RoutingPlan::chain(chain.clone(), result.primary.method);
                }
                return self.chain_builder.build_sequential_chain(&intents, result.primary.method);
            }
        }
        
        let intent = result.primary.intent.clone();
        
        if let Some(chain) = self.chain_builder.get_context_chain(&intent, context) {
            return RoutingPlan::chain(chain, result.primary.method);
        }
        
        RoutingPlan::single(intent, result.primary.confidence, result.primary.method)
    }
    
    pub fn context(&self) -> &ConversationContext {
        &self.context
    }
    
    pub fn clear_context(&mut self) {
        self.context.clear();
    }
}

impl Default for EnhancedOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn detect_intent(query: &str) -> BioIntent {
    router::IntentRouter::new().route(query).primary.intent
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bio_intent_skill_name() {
        assert_eq!(BioIntent::Pharmacogenomics.skill_name(), "pharmgx-reporter");
        assert_eq!(BioIntent::VariantAnnotation.skill_name(), "vcf-annotator");
    }
    
    #[test]
    fn test_bio_intent_requires_input() {
        assert!(BioIntent::Pharmacogenomics.requires_input());
        assert!(!BioIntent::Literature.requires_input());
    }
    
    #[test]
    fn test_enhanced_orchestrator_creation() {
        let orchestrator = EnhancedOrchestrator::new();
        assert!(orchestrator.llm_classifier.is_none());
    }
    
    #[tokio::test]
    async fn test_simple_routing() {
        let mut orchestrator = EnhancedOrchestrator::new();
        let plan = orchestrator.route("Analyze my CYP2D6 gene for drug interactions").await;
        
        assert_eq!(plan.primary_intent, BioIntent::Pharmacogenomics);
    }
}
