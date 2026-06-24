use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TermExplanation {
    pub term: String,
    pub term_freq: usize,
    pub document_frequency: usize,
    pub document_length: usize,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreExplanation {
    pub query_mode: String,
    pub excluded_terms: Vec<String>,
    pub ranker: String,
    pub base_score: f64,
    pub title_boost_multiplier: f64,
    pub final_score: f64,
    pub term_explanations: Vec<TermExplanation>,
}

impl ScoreExplanation {
    pub fn title_boost_applied(&self) -> bool {
        self.title_boost_multiplier > 1.0
    }
}
