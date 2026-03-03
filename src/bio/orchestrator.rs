use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    Unknown,
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
            BioIntent::Unknown => "",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentMatch {
    pub intent: BioIntent,
    pub confidence: f32,
    pub method: MatchMethod,
    pub clarification: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchMethod {
    Keyword,
    LLM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    pub primary: IntentMatch,
    pub alternatives: Vec<IntentMatch>,
    pub needs_clarification: bool,
}

impl IntentResult {
    pub fn single(intent: BioIntent, confidence: f32, method: MatchMethod) -> Self {
        Self {
            primary: IntentMatch {
                intent,
                confidence,
                method,
                clarification: None,
            },
            alternatives: vec![],
            needs_clarification: false,
        }
    }
}

pub struct IntentRouter {
    keyword_map: HashMap<String, BioIntent>,
    llm_enabled: bool,
}

impl IntentRouter {
    pub fn new() -> Self {
        let mut keyword_map = HashMap::new();

        for kw in [
            "drug",
            "pharmacogen",
            "cyp2d6",
            "cyp2c19",
            "cpic",
            "metabolizer",
            "warfarin",
            "medication",
        ] {
            keyword_map.insert(kw.to_string(), BioIntent::Pharmacogenomics);
        }
        for kw in [
            "ancestry",
            "pca",
            "population",
            "admixture",
            "sgdp",
            "genetic ancestry",
        ] {
            keyword_map.insert(kw.to_string(), BioIntent::Ancestry);
        }
        for kw in [
            "diversity",
            "equity",
            "heim",
            "representation",
            "fst",
            "heterozygosity",
        ] {
            keyword_map.insert(kw.to_string(), BioIntent::Diversity);
        }
        for kw in [
            "nutrition",
            "nutrigen",
            "diet",
            "mthfr",
            "folate",
            "vitamin",
            "omega",
            "lactose",
        ] {
            keyword_map.insert(kw.to_string(), BioIntent::Nutrition);
        }
        for kw in ["variant", "vcf", "vep", "clinvar", "gnomad", "annotation"] {
            keyword_map.insert(kw.to_string(), BioIntent::VariantAnnotation);
        }
        for kw in ["literature", "pubmed", "paper", "citation", "biorxiv"] {
            keyword_map.insert(kw.to_string(), BioIntent::Literature);
        }
        for kw in [
            "single-cell",
            "scrna",
            "scanpy",
            "clustering",
            "marker genes",
        ] {
            keyword_map.insert(kw.to_string(), BioIntent::SingleCell);
        }
        for kw in ["protein", "structure", "alphafold", "boltz", "pdb"] {
            keyword_map.insert(kw.to_string(), BioIntent::ProteinStructure);
        }
        for kw in [
            "repro",
            "nextflow",
            "singularity",
            "conda",
            "reproducibility",
        ] {
            keyword_map.insert(kw.to_string(), BioIntent::Reproducibility);
        }
        for kw in ["sequence", "fastq", "bam", "alignment", "trim", "qc"] {
            keyword_map.insert(kw.to_string(), BioIntent::SequenceAnalysis);
        }
        for kw in ["metagenomics", "16s", "shotgun", "microbiome"] {
            keyword_map.insert(kw.to_string(), BioIntent::Metagenomics);
        }
        for kw in ["semantic", "research gap", "neglect", "ntd"] {
            keyword_map.insert(kw.to_string(), BioIntent::SemanticAnalysis);
        }

        Self {
            keyword_map,
            llm_enabled: false,
        }
    }

    pub fn with_llm(mut self, enabled: bool) -> Self {
        self.llm_enabled = enabled;
        self
    }

    pub fn route(&self, query: &str) -> IntentResult {
        let query_lower = query.to_lowercase();

        let mut matches: Vec<(BioIntent, i32)> = vec![];

        for (keyword, intent) in &self.keyword_map {
            if query_lower.contains(&keyword.to_lowercase()) {
                let score = keyword.len() as i32;
                matches.push((*intent, score));
            }
        }

        matches.sort_by(|a, b| b.1.cmp(&a.1));

        if matches.is_empty() {
            return IntentResult::single(BioIntent::Unknown, 0.0, MatchMethod::Keyword);
        }

        let best_intent = matches[0].0;
        let confidence = (matches[0].1 as f32 / 10.0).min(1.0);

        let has_multiple = matches.len() > 1 && matches[0].1 == matches[1].1;

        if has_multiple {
            IntentResult {
                primary: IntentMatch {
                    intent: best_intent,
                    confidence,
                    method: MatchMethod::Keyword,
                    clarification: Some("Multiple intents detected".to_string()),
                },
                alternatives: vec![],
                needs_clarification: true,
            }
        } else {
            IntentResult::single(best_intent, confidence, MatchMethod::Keyword)
        }
    }
}

pub trait LlmProvider: Send + Sync {
    fn complete(&self, prompt: &str) -> Result<String, anyhow::Error>;
}

impl Default for IntentRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl BioIntent {
    pub fn from_query(query: &str) -> Self {
        IntentRouter::new().route(query).primary.intent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pharmacogenomics() {
        let router = IntentRouter::new();
        let result = router.route("What drugs should I avoid based on my CYP2D6?");
        assert_eq!(result.primary.intent, BioIntent::Pharmacogenomics);
    }

    #[test]
    fn test_nutrition() {
        let router = IntentRouter::new();
        let result = router.route("How does MTHFR affect folate?");
        assert_eq!(result.primary.intent, BioIntent::Nutrition);
    }

    #[test]
    fn test_unknown() {
        let router = IntentRouter::new();
        let result = router.route("hello world");
        assert_eq!(result.primary.intent, BioIntent::Unknown);
    }
}
