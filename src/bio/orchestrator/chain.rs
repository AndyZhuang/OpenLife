//! Skill Chain - Multi-step workflow composition

use crate::bio::orchestrator::{BioIntent, MatchMethod};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStep {
    pub skill: String,
    pub name: String,
    pub description: String,
    pub input_from: Option<String>,
    pub transform: Option<Transform>,
    pub optional: bool,
}

impl ChainStep {
    pub fn new(skill: &str, name: &str, description: &str) -> Self {
        Self {
            skill: skill.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            input_from: None,
            transform: None,
            optional: false,
        }
    }

    pub fn with_input_from(mut self, step_name: &str) -> Self {
        self.input_from = Some(step_name.to_string());
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transform {
    ExtractField(String),
    RenameFields(HashMap<String, String>),
    Filter(String),
    Custom(String),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillChain {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<ChainStep>,
    pub parallelizable: bool,
    pub estimated_time: Option<u32>,
}

impl SkillChain {
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            steps: vec![],
            parallelizable: false,
            estimated_time: None,
        }
    }

    pub fn add_step(mut self, step: ChainStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn parallelizable(mut self) -> Self {
        self.parallelizable = true;
        self
    }

    pub fn with_estimated_time(mut self, seconds: u32) -> Self {
        self.estimated_time = Some(seconds);
        self
    }

    pub fn skill_names(&self) -> Vec<&str> {
        self.steps.iter().map(|s| s.skill.as_str()).collect()
    }

    pub fn is_valid(&self) -> bool {
        !self.steps.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPlan {
    pub primary_intent: BioIntent,
    pub intents: Vec<BioIntent>,
    pub chain: Option<SkillChain>,
    pub method: MatchMethod,
    pub confidence: f32,
    pub needs_confirmation: bool,
    pub confirmation_message: Option<String>,
}

impl RoutingPlan {
    pub fn single(intent: BioIntent, confidence: f32, method: MatchMethod) -> Self {
        let skill_name = intent.skill_name();
        let display_name = intent.display_name();
        let description = intent.description();
        Self {
            primary_intent: intent.clone(),
            intents: vec![intent],
            chain: if !skill_name.is_empty() {
                Some(
                    SkillChain::new(skill_name, display_name, description)
                        .add_step(ChainStep::new(
                            skill_name,
                            display_name,
                            description,
                        )),
                )
            } else {
                None
            },
            method,
            confidence,
            needs_confirmation: false,
            confirmation_message: None,
        }
    }

    pub fn chain(chain: SkillChain, method: MatchMethod) -> Self {
        let intents: Vec<BioIntent> = chain
            .steps
            .iter()
            .map(|s| BioIntent::from_skill_name(&s.skill))
            .collect();

        Self {
            primary_intent: intents.first().cloned().unwrap_or(BioIntent::Unknown),
            intents,
            chain: Some(chain.clone()),
            method,
            confidence: 0.85,
            needs_confirmation: true,
            confirmation_message: Some(format!(
                "I'll run {} steps: {}. Proceed?",
                chain.steps.len(),
                chain
                    .steps
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join(" → ")
            )),
        }
    }

    pub fn unknown() -> Self {
        Self {
            primary_intent: BioIntent::Unknown,
            intents: vec![],
            chain: None,
            method: MatchMethod::Keyword,
            confidence: 0.0,
            needs_confirmation: false,
            confirmation_message: Some(
                "I couldn't determine what analysis to perform. Could you be more specific?"
                    .to_string(),
            ),
        }
    }

    pub fn primary_skill(&self) -> Option<&str> {
        self.chain
            .as_ref()
            .and_then(|c| c.steps.first().map(|s| s.skill.as_str()))
    }

    pub fn all_skills(&self) -> Vec<&str> {
        self.chain
            .as_ref()
            .map(|c| c.skill_names())
            .unwrap_or_default()
    }

    pub fn is_multi_step(&self) -> bool {
        self.chain
            .as_ref()
            .map(|c| c.steps.len() > 1)
            .unwrap_or(false)
    }
}

pub struct ChainBuilder {
    predefined_chains: HashMap<String, SkillChain>,
    intent_chains: HashMap<Vec<BioIntent>, SkillChain>,
}

impl ChainBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            predefined_chains: HashMap::new(),
            intent_chains: HashMap::new(),
        };

        builder.register_predefined_chains();
        builder
    }

    fn register_predefined_chains(&mut self) {
        let vcf_analysis = SkillChain::new(
            "vcf-analysis-pipeline",
            "VCF Analysis Pipeline",
            "Annotate variants, extract pharmacogenomics, and generate report",
        )
        .add_step(ChainStep::new(
            "vcf-annotator",
            "Variant Annotation",
            "Annotate VCF with VEP, ClinVar, gnomAD",
        ))
        .add_step(
            ChainStep::new(
                "pharmgx-reporter",
                "Pharmacogenomics",
                "Extract drug response variants",
            )
            .with_input_from("Variant Annotation"),
        )
        .add_step(ChainStep::new(
            "bio-orchestrator",
            "Report Generation",
            "Generate comprehensive report",
        ))
        .with_estimated_time(120);

        self.predefined_chains
            .insert("vcf-analysis-pipeline".to_string(), vcf_analysis);

        let diversity_analysis = SkillChain::new(
            "diversity-analysis",
            "Diversity Analysis Pipeline",
            "Calculate population diversity metrics from genetic data",
        )
        .add_step(ChainStep::new(
            "ancestry-pca",
            "Ancestry Analysis",
            "Determine population structure",
        ))
        .add_step(
            ChainStep::new(
                "equity-scorer",
                "Diversity Scoring",
                "Calculate HEIM and FST metrics",
            )
            .with_input_from("Ancestry Analysis"),
        )
        .with_estimated_time(60);

        self.predefined_chains
            .insert("diversity-analysis".to_string(), diversity_analysis);

        let pharmgx_pipeline = SkillChain::new(
            "pharmgx-pipeline",
            "Pharmacogenomics Pipeline",
            "Generate pharmacogenomic report with drug recommendations",
        )
        .add_step(ChainStep::new(
            "pharmgx-reporter",
            "PharmGx Analysis",
            "Analyze drug metabolism genes",
        ))
        .add_step(ChainStep::new(
            "bio-orchestrator",
            "Report",
            "Generate clinical report",
        ))
        .with_estimated_time(30);

        self.predefined_chains
            .insert("pharmgx-pipeline".to_string(), pharmgx_pipeline);

        let scrna_pipeline = SkillChain::new(
            "scrna-pipeline",
            "Single-Cell Analysis Pipeline",
            "Complete single-cell RNA-seq analysis",
        )
        .add_step(ChainStep::new(
            "scrna-orchestrator",
            "QC & Preprocessing",
            "Quality control and normalization",
        ))
        .add_step(
            ChainStep::new(
                "scrna-orchestrator",
                "Clustering",
                "Identify cell populations",
            )
            .with_input_from("QC & Preprocessing"),
        )
        .add_step(
            ChainStep::new(
                "scrna-orchestrator",
                "Marker Genes",
                "Find marker genes per cluster",
            )
            .with_input_from("Clustering"),
        )
        .with_estimated_time(300);

        self.predefined_chains
            .insert("scrna-pipeline".to_string(), scrna_pipeline);

        let lit_pipeline = SkillChain::new(
            "literature-review",
            "Literature Review Pipeline",
            "Search and synthesize literature on a topic",
        )
        .add_step(ChainStep::new(
            "lit-synthesizer",
            "Search",
            "Search PubMed and bioRxiv",
        ))
        .add_step(
            ChainStep::new(
                "lit-synthesizer",
                "Synthesis",
                "Generate literature summary",
            )
            .with_input_from("Search"),
        )
        .with_estimated_time(45);

        self.predefined_chains
            .insert("literature-review".to_string(), lit_pipeline);

        self.register_intent_combination(
            vec![BioIntent::VariantAnnotation, BioIntent::Pharmacogenomics],
            "vcf-analysis-pipeline",
        );

        self.register_intent_combination(
            vec![BioIntent::Ancestry, BioIntent::Diversity],
            "diversity-analysis",
        );

        self.register_intent_combination(vec![BioIntent::Pharmacogenomics], "pharmgx-pipeline");

        self.register_intent_combination(vec![BioIntent::SingleCell], "scrna-pipeline");

        self.register_intent_combination(vec![BioIntent::Literature], "literature-review");
    }

    fn register_intent_combination(&mut self, intents: Vec<BioIntent>, chain_id: &str) {
        if let Some(chain) = self.predefined_chains.get(chain_id).cloned() {
            self.intent_chains.insert(intents, chain);
        }
    }

    pub fn get_chain_by_id(&self, id: &str) -> Option<&SkillChain> {
        self.predefined_chains.get(id)
    }

    pub fn get_chain(&self, intents: &[BioIntent]) -> Option<&SkillChain> {
        if let Some(chain) = self.intent_chains.get(intents) {
            return Some(chain);
        }

        for (chain_intents, chain) in &self.intent_chains {
            if intents.iter().all(|i| chain_intents.contains(i)) {
                return Some(chain);
            }
        }

        None
    }

    pub fn get_context_chain(
        &self,
        intent: &BioIntent,
        _context: &super::context::QueryContext,
    ) -> Option<SkillChain> {
        match intent {
            BioIntent::VariantAnnotation => {
                self.predefined_chains.get("vcf-analysis-pipeline").cloned()
            }
            BioIntent::Diversity => self.predefined_chains.get("diversity-analysis").cloned(),
            _ => None,
        }
    }

    pub fn build_sequential_chain(
        &self,
        intents: &[BioIntent],
        method: MatchMethod,
    ) -> RoutingPlan {
        let chain_id = intents
            .iter()
            .map(|i| i.skill_name())
            .collect::<Vec<_>>()
            .join("-");

        let name = intents
            .iter()
            .map(|i| i.display_name())
            .collect::<Vec<_>>()
            .join(" → ");

        let mut chain = SkillChain::new(&chain_id, &name, "Multi-step analysis");

        for intent in intents {
            if intent != &BioIntent::Unknown {
                chain = chain.add_step(ChainStep::new(
                    intent.skill_name(),
                    intent.display_name(),
                    intent.description(),
                ));
            }
        }

        RoutingPlan::chain(chain, method)
    }

    pub fn add_chain(&mut self, chain: SkillChain) {
        self.predefined_chains.insert(chain.id.clone(), chain);
    }

    pub fn list_chains(&self) -> Vec<&SkillChain> {
        self.predefined_chains.values().collect()
    }
}

impl Default for ChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BioIntent {
    fn from_skill_name(skill_name: &str) -> Self {
        match skill_name {
            "pharmgx-reporter" => BioIntent::Pharmacogenomics,
            "ancestry-pca" => BioIntent::Ancestry,
            "equity-scorer" => BioIntent::Diversity,
            "nutrigx-advisor" => BioIntent::Nutrition,
            "vcf-annotator" => BioIntent::VariantAnnotation,
            "lit-synthesizer" => BioIntent::Literature,
            "scrna-orchestrator" => BioIntent::SingleCell,
            "struct-predictor" => BioIntent::ProteinStructure,
            "repro-enforcer" => BioIntent::Reproducibility,
            "seq-wrangler" => BioIntent::SequenceAnalysis,
            "bio-orchestrator" => BioIntent::DatabaseQuery,
            "semantic-sim" => BioIntent::SemanticAnalysis,
            "metagenomics" => BioIntent::Metagenomics,
            "labclaw/pharma" => BioIntent::Cheminformatics,
            "labclaw/med" => BioIntent::Clinical,
            "labclaw/vision" => BioIntent::Vision,
            "labclaw/general" => BioIntent::DataScience,
            "labclaw/bio" => BioIntent::LabAutomation,
            _ => BioIntent::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_step_creation() {
        let step = ChainStep::new("pharmgx-reporter", "PharmGx", "Drug metabolism analysis");
        assert_eq!(step.skill, "pharmgx-reporter");
        assert_eq!(step.optional, false);
    }

    #[test]
    fn test_skill_chain_building() {
        let chain = SkillChain::new("test", "Test Chain", "Description")
            .add_step(ChainStep::new("skill1", "Step 1", "First step"))
            .add_step(ChainStep::new("skill2", "Step 2", "Second step"));

        assert_eq!(chain.steps.len(), 2);
        assert!(chain.is_valid());
    }

    #[test]
    fn test_chain_builder() {
        let builder = ChainBuilder::new();
        assert!(builder.get_chain_by_id("vcf-analysis-pipeline").is_some());
        assert!(builder.get_chain_by_id("diversity-analysis").is_some());
    }

    #[test]
    fn test_intent_chain_lookup() {
        let builder = ChainBuilder::new();

        let chain = builder.get_chain(&[BioIntent::VariantAnnotation, BioIntent::Pharmacogenomics]);
        assert!(chain.is_some());
        assert_eq!(chain.unwrap().steps.len(), 3);
    }

    #[test]
    fn test_routing_plan_single() {
        let plan = RoutingPlan::single(BioIntent::Pharmacogenomics, 0.9, MatchMethod::Keyword);

        assert_eq!(plan.primary_intent, BioIntent::Pharmacogenomics);
        assert!(!plan.is_multi_step());
    }

    #[test]
    fn test_sequential_chain_building() {
        let builder = ChainBuilder::new();
        let plan = builder.build_sequential_chain(
            &[BioIntent::VariantAnnotation, BioIntent::Diversity],
            MatchMethod::Keyword,
        );

        assert!(plan.is_multi_step());
        assert_eq!(plan.intents.len(), 2);
    }
}
