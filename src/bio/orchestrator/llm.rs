//! LLM-assisted intent classification

use crate::bio::orchestrator::{BioIntent, MatchMethod};
use crate::bio::orchestrator::router::{IntentMatch, IntentResult};
use crate::bio::orchestrator::context::QueryContext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: None,
            base_url: None,
            temperature: 0.3,
            max_tokens: 500,
        }
    }
}

impl LlmConfig {
    pub fn openai(api_key: String) -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: Some(api_key),
            base_url: None,
            temperature: 0.3,
            max_tokens: 500,
        }
    }
    
    pub fn anthropic(api_key: String) -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-3-haiku-20240307".to_string(),
            api_key: Some(api_key),
            base_url: None,
            temperature: 0.3,
            max_tokens: 500,
        }
    }
    
    pub fn openrouter(api_key: String) -> Self {
        Self {
            provider: "openrouter".to_string(),
            model: "anthropic/claude-3-haiku".to_string(),
            api_key: Some(api_key),
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            temperature: 0.3,
            max_tokens: 500,
        }
    }
    
    pub fn ollama(model: String) -> Self {
        Self {
            provider: "ollama".to_string(),
            model,
            api_key: None,
            base_url: Some("http://localhost:11434/v1".to_string()),
            temperature: 0.3,
            max_tokens: 500,
        }
    }
}

pub struct LlmClassifier {
    config: LlmConfig,
    client: Option<reqwest::Client>,
}

impl LlmClassifier {
    pub fn new(config: LlmConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .ok();
        
        Self { config, client }
    }
    
    pub async fn classify(&self, query: &str, context: &QueryContext) -> anyhow::Result<IntentResult> {
        let Some(client) = &self.client else {
            return Ok(IntentResult::single(BioIntent::Unknown, 0.0, MatchMethod::LLM));
        };
        
        let prompt = self.build_classification_prompt(query, context);
        
        let response = self.call_llm(client, &prompt).await?;
        
        self.parse_llm_response(&response)
    }
    
    fn build_classification_prompt(&self, query: &str, context: &QueryContext) -> String {
        let context_info = if !context.mentioned_files.is_empty() {
            format!("\nMentioned files: {:?}", context.mentioned_files)
        } else {
            String::new()
        };
        
        let gene_info = if !context.mentioned_genes.is_empty() {
            format!("\nMentioned genes: {:?}", context.mentioned_genes)
        } else {
            String::new()
        };
        
        format!(
            r#"Classify the following bioinformatics query. Return a JSON object with:
- "intent": one of [Pharmacogenomics, Ancestry, Diversity, Nutrition, VariantAnnotation, Literature, SingleCell, ProteinStructure, Reproducibility, SequenceAnalysis, Metagenomics, Cheminformatics, Clinical, Vision, DataScience, LabAutomation, Unknown]
- "confidence": float 0.0-1.0
- "alternatives": array of additional intents if multi-intent detected
- "reasoning": brief explanation

Query: "{}"
{}
{}

Return ONLY valid JSON, no other text."#,
            query, context_info, gene_info
        )
    }
    
    async fn call_llm(&self, client: &reqwest::Client, prompt: &str) -> anyhow::Result<String> {
        let api_key = if let Some(ref key) = self.config.api_key {
            key.clone()
        } else if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            key
        } else {
            anyhow::bail!("No API key configured");
        };
        
        let base_url = self.config.base_url.as_deref()
            .unwrap_or("https://api.openai.com/v1");
        
        let url = format!("{}/chat/completions", base_url);
        
        let body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });
        
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error = response.text().await?;
            anyhow::bail!("LLM API error: {}", error);
        }
        
        let json: serde_json::Value = response.json().await?;
        
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid LLM response format"))?;
        
        Ok(content.to_string())
    }
    
    fn parse_llm_response(&self, response: &str) -> anyhow::Result<IntentResult> {
        let parsed: LlmClassification = serde_json::from_str(response)
            .unwrap_or_else(|_| LlmClassification {
                intent: "Unknown".to_string(),
                confidence: 0.0,
                alternatives: vec![],
                reasoning: None,
            });
        
        let intent = Self::parse_intent(&parsed.intent);
        let confidence = parsed.confidence.clamp(0.0, 1.0);
        
        let alternatives: Vec<IntentMatch> = parsed.alternatives.iter()
            .map(|s| IntentMatch {
                intent: Self::parse_intent(s),
                confidence: confidence * 0.8,
                method: MatchMethod::LLM,
                clarification: None,
                matched_keywords: vec![],
            })
            .collect();
        
        let primary = IntentMatch {
            intent: intent.clone(),
            confidence,
            method: MatchMethod::LLM,
            clarification: parsed.reasoning,
            matched_keywords: vec![],
        };
        
        if alternatives.is_empty() {
            Ok(IntentResult::single(intent.clone(), confidence, MatchMethod::LLM))
        } else {
            Ok(IntentResult::multi(primary, alternatives, false))
        }
    }
    
    fn parse_intent(s: &str) -> BioIntent {
        match s.to_lowercase().as_str() {
            "pharmacogenomics" => BioIntent::Pharmacogenomics,
            "ancestry" => BioIntent::Ancestry,
            "diversity" => BioIntent::Diversity,
            "nutrition" => BioIntent::Nutrition,
            "variantannotation" | "variant_annotation" | "variant-annotation" => BioIntent::VariantAnnotation,
            "literature" => BioIntent::Literature,
            "singlecell" | "single_cell" | "single-cell" => BioIntent::SingleCell,
            "proteinstructure" | "protein_structure" | "protein-structure" => BioIntent::ProteinStructure,
            "reproducibility" => BioIntent::Reproducibility,
            "sequenceanalysis" | "sequence_analysis" | "sequence-analysis" => BioIntent::SequenceAnalysis,
            "metagenomics" => BioIntent::Metagenomics,
            "cheminformatics" => BioIntent::Cheminformatics,
            "clinical" => BioIntent::Clinical,
            "vision" => BioIntent::Vision,
            "datascience" | "data_science" | "data-science" => BioIntent::DataScience,
            "labautomation" | "lab_automation" | "lab-automation" => BioIntent::LabAutomation,
            _ => BioIntent::Unknown,
        }
    }
}

#[derive(Debug, Deserialize)]
struct LlmClassification {
    intent: String,
    confidence: f32,
    alternatives: Vec<String>,
    #[serde(default)]
    reasoning: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_llm_config_defaults() {
        let config = LlmConfig::default();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.temperature, 0.3);
    }
    
    #[test]
    fn test_intent_parsing() {
        assert_eq!(LlmClassifier::parse_intent("Pharmacogenomics"), BioIntent::Pharmacogenomics);
        assert_eq!(LlmClassifier::parse_intent("variant_annotation"), BioIntent::VariantAnnotation);
        assert_eq!(LlmClassifier::parse_intent("unknown_thing"), BioIntent::Unknown);
    }
}