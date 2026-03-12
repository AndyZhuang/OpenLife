//! Intent Router - Keyword and LLM-based routing
//!
//! This module handles the detection of bioinformatics intents from natural language queries.

use crate::bio::orchestrator::BioIntent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Method used to determine the intent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchMethod {
    /// Fast keyword matching
    Keyword,
    /// LLM-assisted classification
    LLM,
    /// User-specified explicitly
    Explicit,
    /// Context-inferred from conversation
    ContextInferred,
}

/// A single intent match with confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentMatch {
    /// The matched intent
    pub intent: BioIntent,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Method used for matching
    pub method: MatchMethod,
    /// Optional clarification message
    pub clarification: Option<String>,
    /// Keywords that matched (for keyword routing)
    pub matched_keywords: Vec<String>,
}

/// Result of intent routing with primary and alternative matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    /// Primary (best) intent match
    pub primary: IntentMatch,
    /// Alternative intents that were also detected
    pub alternatives: Vec<IntentMatch>,
    /// Whether clarification is needed from user
    pub needs_clarification: bool,
    /// Suggested clarification question
    pub clarification_question: Option<String>,
}

impl IntentResult {
    /// Create a single-intent result
    pub fn single(intent: BioIntent, confidence: f32, method: MatchMethod) -> Self {
        Self {
            primary: IntentMatch {
                intent,
                confidence,
                method,
                clarification: None,
                matched_keywords: vec![],
            },
            alternatives: vec![],
            needs_clarification: false,
            clarification_question: None,
        }
    }

    /// Create a multi-intent result
    pub fn multi(
        primary: IntentMatch,
        alternatives: Vec<IntentMatch>,
        needs_clarification: bool,
    ) -> Self {
        let clarification_question = if needs_clarification {
            Some(Self::generate_clarification_question(
                &primary,
                &alternatives,
            ))
        } else {
            None
        };

        Self {
            primary,
            alternatives,
            needs_clarification,
            clarification_question,
        }
    }

    fn generate_clarification_question(
        primary: &IntentMatch,
        alternatives: &[IntentMatch],
    ) -> String {
        let all_intents: Vec<&BioIntent> = std::iter::once(&primary.intent)
            .chain(alternatives.iter().map(|m| &m.intent))
            .collect();

        format!(
            "I detected multiple analysis types: {}. Would you like me to run all of them sequentially, or did you mean one specifically?",
            all_intents.iter()
                .map(|i| i.display_name())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    /// Check if this is a multi-intent result
    pub fn is_multi(&self) -> bool {
        !self.alternatives.is_empty()
    }

    /// Get all detected intents
    pub fn all_intents(&self) -> Vec<BioIntent> {
        std::iter::once(self.primary.intent.clone())
            .chain(self.alternatives.iter().map(|m| m.intent.clone()))
            .filter(|i| *i != BioIntent::Unknown)
            .collect()
    }
}

/// Keyword-based intent router
pub struct IntentRouter {
    /// Keyword to intent mapping
    keyword_map: HashMap<String, BioIntent>,
    /// Intent to keywords reverse mapping (for debugging)
    intent_keywords: HashMap<BioIntent, Vec<String>>,
    /// Phrase patterns for complex matching
    phrase_patterns: Vec<(String, BioIntent, f32)>,
}

impl IntentRouter {
    /// Create a new intent router with all keyword mappings
    pub fn new() -> Self {
        let mut keyword_map = HashMap::new();
        let mut intent_keywords: HashMap<BioIntent, Vec<String>> = HashMap::new();

        let add_keywords = |map: &mut HashMap<String, BioIntent>,
                            ikw: &mut HashMap<BioIntent, Vec<String>>,
                            intent: BioIntent,
                            keywords: &[&str]| {
            for kw in keywords {
                map.insert(kw.to_lowercase(), intent.clone());
                ikw.entry(intent.clone()).or_default().push(kw.to_string());
            }
        };

        // Pharmacogenomics keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::Pharmacogenomics,
            &[
                "drug",
                "pharmacogen",
                "cyp2d6",
                "cyp2c19",
                "cyp2c9",
                "cpic",
                "metabolizer",
                "warfarin",
                "medication",
                "pharmgx",
                "pgx",
                "clopidogrel",
                "codeine",
                "tamoxifen",
                "pharmacogenomic",
                "star allele",
                "diplotype",
                "drug response",
                "drug metabolism",
                "adverse drug",
                "drug interaction",
                "dosage",
                "dose adjustment",
            ],
        );

        // Ancestry keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::Ancestry,
            &[
                "ancestry",
                "pca",
                "population",
                "admixture",
                "sgdp",
                "genetic ancestry",
                "ethnicity",
                "continental",
                "ancestral",
                "population structure",
                "principal component",
                "genetic origin",
                "heritage",
                "lineage",
                "genealogy",
            ],
        );

        // Diversity/Equity keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::Diversity,
            &[
                "diversity",
                "equity",
                "heim",
                "representation",
                "fst",
                "heterozygosity",
                "inclusion",
                "bias",
                "fair",
                "hef",
                "population diversity",
                "genetic diversity",
                "allele frequency",
                "minority",
                "underrepresented",
            ],
        );

        // Nutrigenomics keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::Nutrition,
            &[
                "nutrition",
                "nutrigen",
                "diet",
                "mthfr",
                "folate",
                "vitamin",
                "omega",
                "lactose",
                "nutrigenomics",
                "nutrient",
                "supplement",
                "food",
                "eating",
                "metabolism nutrition",
                "caffeine",
                "alcohol",
                "sugar",
                "carbohydrate",
            ],
        );

        // Variant Annotation keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::VariantAnnotation,
            &[
                "variant",
                "vcf",
                "vep",
                "clinvar",
                "gnomad",
                "annotation",
                "snv",
                "indel",
                "mutation",
                "pathogenic",
                "benign",
                "allele frequency",
                "consequence",
                "impact",
                "snp",
                "rsid",
                "rs number",
                "genomic variant",
                "exome",
                "genome",
                "interpret variant",
                "annotate",
                "functional annotation",
            ],
        );

        // Literature keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::Literature,
            &[
                "literature",
                "pubmed",
                "paper",
                "citation",
                "biorxiv",
                "article",
                "publication",
                "research paper",
                "scientific literature",
                "medline",
                "pmid",
                "doi",
                "abstract",
                "review",
                "synthesis",
                "summarize paper",
                "find papers",
                "search papers",
            ],
        );

        // Single-cell keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::SingleCell,
            &[
                "single-cell",
                "scrna",
                "scanpy",
                "clustering",
                "marker genes",
                "cell type",
                "scRNA-seq",
                "single cell",
                "cell atlas",
                "umi",
                "gene expression",
                "h5ad",
                "anndata",
                "seurat",
                "cell population",
                "cell cluster",
                "differential expression",
            ],
        );

        // Protein Structure keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::ProteinStructure,
            &[
                "protein",
                "structure",
                "alphafold",
                "boltz",
                "pdb",
                "fold",
                "folding",
                "3d structure",
                "secondary structure",
                "tertiary",
                "domain",
                "binding site",
                "active site",
                "homology modeling",
                "structure prediction",
                "esmfold",
            ],
        );

        // Reproducibility keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::Reproducibility,
            &[
                "repro",
                "nextflow",
                "singularity",
                "conda",
                "reproducibility",
                "workflow",
                "pipeline",
                "docker",
                "container",
                "snakemake",
                "cwl",
                "wdl",
                "environment",
                "export analysis",
            ],
        );

        // Sequence Analysis keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::SequenceAnalysis,
            &[
                "sequence",
                "fastq",
                "bam",
                "alignment",
                "trim",
                "qc",
                "quality control",
                "read",
                "nucleotide",
                "genome sequencing",
                "bwa",
                "bowtie",
                "minimap",
                "samtools",
                "fastp",
                "adapter",
                "base calling",
                "coverage",
                "depth",
            ],
        );

        // Metagenomics keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::Metagenomics,
            &[
                "metagenomics",
                "16s",
                "shotgun",
                "microbiome",
                "microbiota",
                "kraken",
                "bracken",
                "taxonomic",
                "taxa",
                "species abundance",
                "gut bacteria",
                "microbial community",
                "amplicon",
            ],
        );

        // Cheminformatics keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::Cheminformatics,
            &[
                "rdkit",
                "molecular",
                "cheminformatics",
                "docking",
                "compound",
                "molecule",
                "smiles",
                "maccs",
                "fingerprint",
                "descriptors",
                "qsar",
                "admet",
                "drug discovery",
                "ligand",
                "receptor",
                "inhibitor",
                "agonist",
                "antagonist",
                "pharmacophore",
            ],
        );

        // Clinical keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::Clinical,
            &[
                "clinical",
                "trial",
                "patient",
                "diagnosis",
                "treatment",
                "oncology",
                "cancer",
                "precision medicine",
                "therapy",
                "prognosis",
                "biomarker",
                "companion diagnostic",
                "ehr",
                "icd",
                "cpt",
                "clinical trial",
                "phase",
                "fda approved",
            ],
        );

        // Vision keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::Vision,
            &[
                "vision",
                "image",
                "tracking",
                "pose",
                "segmentation",
                "camera",
                "video",
                "microscopy",
                "cell tracking",
                "histology",
                "pathology image",
                "medical imaging",
                "ct",
                "mri",
            ],
        );

        // Data Science keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::DataScience,
            &[
                "statistics",
                "machine learning",
                "ml",
                "regression",
                "classification",
                "visualization",
                "plot",
                "data analysis",
                "statistical test",
                "p-value",
                "significance",
                "correlation",
                "pca",
                "clustering",
                "dimensionality reduction",
                "feature selection",
                "model training",
            ],
        );

        // Lab Automation keywords
        add_keywords(
            &mut keyword_map,
            &mut intent_keywords,
            BioIntent::LabAutomation,
            &[
                "automation",
                "robot",
                "lab",
                "protocol",
                "benchling",
                "opentrons",
                "lims",
                "laboratory",
                "wet lab",
                "pipetting",
                "sample tracking",
                "inventory",
                "plate",
                "well",
            ],
        );

        // Build phrase patterns for multi-word matching
        let phrase_patterns = vec![
            // Multi-intent patterns
            (
                "drug metabolism and ancestry".to_string(),
                BioIntent::Pharmacogenomics,
                0.9,
            ),
            (
                "pharmacogenomics and diversity".to_string(),
                BioIntent::Pharmacogenomics,
                0.85,
            ),
            (
                "variant annotation and diversity".to_string(),
                BioIntent::VariantAnnotation,
                0.9,
            ),
            (
                "vcf annotation and equity".to_string(),
                BioIntent::VariantAnnotation,
                0.9,
            ),
            (
                "annotate my variants".to_string(),
                BioIntent::VariantAnnotation,
                0.95,
            ),
            (
                "analyze my genetic data".to_string(),
                BioIntent::Pharmacogenomics,
                0.8,
            ),
            (
                "what drugs should i avoid".to_string(),
                BioIntent::Pharmacogenomics,
                0.95,
            ),
            (
                "check my medications".to_string(),
                BioIntent::Pharmacogenomics,
                0.9,
            ),
            ("find papers on".to_string(), BioIntent::Literature, 0.9),
            ("search pubmed for".to_string(), BioIntent::Literature, 0.95),
            (
                "analyze my microbiome".to_string(),
                BioIntent::Metagenomics,
                0.95,
            ),
            (
                "profile my gut bacteria".to_string(),
                BioIntent::Metagenomics,
                0.9,
            ),
        ];

        Self {
            keyword_map,
            intent_keywords,
            phrase_patterns,
        }
    }

    /// Route a query to determine the intent
    pub fn route(&self, query: &str) -> IntentResult {
        let query_lower = query.to_lowercase();

        // Step 1: Check phrase patterns first (higher precision)
        for (pattern, intent, confidence) in &self.phrase_patterns {
            if query_lower.contains(pattern) {
                return IntentResult::single(intent.clone(), *confidence, MatchMethod::Keyword);
            }
        }

        // Step 2: Keyword matching
        let mut matches: Vec<(BioIntent, i32, String)> = vec![];

        for (keyword, intent) in &self.keyword_map {
            if query_lower.contains(keyword) {
                // Score based on keyword length (longer = more specific)
                let score = keyword.len() as i32;
                matches.push((intent.clone(), score, keyword.clone()));
            }
        }

        // Sort by score descending
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        if matches.is_empty() {
            return IntentResult::single(BioIntent::Unknown, 0.0, MatchMethod::Keyword);
        }

        // Collect top matches
        let best_score = matches[0].1;
        let top_matches: Vec<_> = matches
            .iter()
            .filter(|(_, score, _)| *score >= best_score * 8 / 10) // Within 80% of best
            .take(3) // Max 3 alternatives
            .collect();

        // Build primary match
        let primary = IntentMatch {
            intent: top_matches[0].0.clone(),
            confidence: Self::calculate_confidence(top_matches[0].1, top_matches.len()),
            method: MatchMethod::Keyword,
            clarification: None,
            matched_keywords: top_matches.iter().map(|(_, _, kw)| kw.clone()).collect(),
        };

        // Build alternatives if multiple intents detected
        if top_matches.len() > 1 {
            let alternatives: Vec<IntentMatch> = top_matches[1..]
                .iter()
                .map(|(intent, score, kw)| IntentMatch {
                    intent: intent.clone(),
                    confidence: Self::calculate_confidence(*score, top_matches.len()),
                    method: MatchMethod::Keyword,
                    clarification: None,
                    matched_keywords: vec![kw.clone()],
                })
                .collect();

            // Check if alternatives are different intents (not duplicates)
            let unique_intents: std::collections::HashSet<_> = std::iter::once(primary.intent.clone())
                .chain(alternatives.iter().map(|m| m.intent.clone()))
                .collect();

            if unique_intents.len() > 1 {
                return IntentResult::multi(primary, alternatives, true);
            }
        }

        IntentResult::single(primary.intent.clone(), primary.confidence, MatchMethod::Keyword)
    }

    fn calculate_confidence(score: i32, match_count: usize) -> f32 {
        // Base confidence from keyword specificity
        let base = (score as f32 / 15.0).min(1.0);

        // Reduce confidence if many matches (ambiguous)
        let ambiguity_penalty = if match_count > 1 {
            0.1 * (match_count - 1) as f32
        } else {
            0.0
        };

        (base - ambiguity_penalty).max(0.0).min(1.0)
    }

    /// Get all keywords for a specific intent
    pub fn get_keywords_for_intent(&self, intent: BioIntent) -> Option<&Vec<String>> {
        self.intent_keywords.get(&intent)
    }

    /// Add a custom keyword mapping
    pub fn add_keyword(&mut self, keyword: &str, intent: BioIntent) {
        self.keyword_map.insert(keyword.to_lowercase(), intent.clone());
        self.intent_keywords
            .entry(intent)
            .or_default()
            .push(keyword.to_string());
    }
}

impl Default for IntentRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Backward compatible function
impl BioIntent {
    /// Detect intent from a query (simple keyword matching)
    pub fn from_query(query: &str) -> Self {
        IntentRouter::new().route(query).primary.intent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pharmacogenomics_detection() {
        let router = IntentRouter::new();
        let result = router.route("What drugs should I avoid based on my CYP2D6?");
        assert_eq!(result.primary.intent, BioIntent::Pharmacogenomics);
        assert!(result.primary.confidence > 0.5);
    }

    #[test]
    fn test_nutrition_detection() {
        let router = IntentRouter::new();
        let result = router.route("How does MTHFR affect folate metabolism?");
        assert_eq!(result.primary.intent, BioIntent::Nutrition);
    }

    #[test]
    fn test_variant_annotation_detection() {
        let router = IntentRouter::new();
        let result = router.route("Annotate the variants in my VCF file with ClinVar");
        assert_eq!(result.primary.intent, BioIntent::VariantAnnotation);
    }

    #[test]
    fn test_unknown_detection() {
        let router = IntentRouter::new();
        let result = router.route("hello world");
        assert_eq!(result.primary.intent, BioIntent::Unknown);
        assert_eq!(result.primary.confidence, 0.0);
    }

    #[test]
    fn test_multi_intent_detection() {
        let router = IntentRouter::new();
        let result = router
            .route("Analyze my VCF for drug metabolism variants and check population diversity");

        // Should detect both Pharmacogenomics and Diversity
        assert!(result.primary.intent != BioIntent::Unknown);
        // May or may not have alternatives depending on scoring
    }

    #[test]
    fn test_confidence_scoring() {
        let router = IntentRouter::new();

        // Specific query should have high confidence
        let result = router.route("pharmacogenomics");
        assert!(result.primary.confidence > 0.7);

        // Ambiguous query should have lower confidence
        let result = router.route("analysis");
        assert_eq!(result.primary.intent, BioIntent::Unknown);
    }

    #[test]
    fn test_phrase_patterns() {
        let router = IntentRouter::new();

        let result = router.route("What drugs should I avoid?");
        assert_eq!(result.primary.intent, BioIntent::Pharmacogenomics);
        assert!(result.primary.confidence > 0.9);

        let result = router.route("Find papers on CRISPR");
        assert_eq!(result.primary.intent, BioIntent::Literature);
    }
}
