use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RankerKind {
    Tfidf,
    Bm25,
}

impl RankerKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tfidf => "tfidf",
            Self::Bm25 => "bm25",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScoreInput {
    // 排序器只接收打分所需的统计量，不直接依赖搜索索引，便于测试和替换算法。
    pub total_docs: usize,
    pub document_frequency: usize,
    pub term_frequency: usize,
    pub document_length: usize,
    pub average_document_length: f64,
}

pub trait Ranker {
    // 排序器接口把“搜索流程”和“具体排序算法”解耦。
    fn name(&self) -> &'static str;
    fn score(&self, input: ScoreInput) -> f64;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TfIdfRanker;

impl Ranker for TfIdfRanker {
    fn name(&self) -> &'static str {
        "tfidf"
    }

    fn score(&self, input: ScoreInput) -> f64 {
        let total_docs = input.total_docs as f64;
        let document_frequency = input.document_frequency as f64;
        let term_frequency = input.term_frequency as f64;
        // 平滑后的 IDF：避免除零，同时让低频词获得更高权重。
        let idf = ((total_docs + 1.0) / (document_frequency + 1.0)).ln() + 1.0;
        term_frequency * idf
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bm25Ranker {
    pub k1: f64,
    pub b: f64,
}

impl Default for Bm25Ranker {
    fn default() -> Self {
        Self { k1: 1.2, b: 0.75 }
    }
}

impl Ranker for Bm25Ranker {
    fn name(&self) -> &'static str {
        "bm25"
    }

    fn score(&self, input: ScoreInput) -> f64 {
        if input.term_frequency == 0 || input.total_docs == 0 {
            return 0.0;
        }

        let total_docs = input.total_docs as f64;
        let document_frequency = input.document_frequency as f64;
        let term_frequency = input.term_frequency as f64;
        let document_length = input.document_length as f64;
        let average_document_length = input.average_document_length.max(1.0);
        // BM25 在词频和逆文档频率的基础上增加词频饱和和文档长度归一化。
        let idf = (1.0 + (total_docs - document_frequency + 0.5) / (document_frequency + 0.5)).ln();
        let length_factor = 1.0 - self.b + self.b * (document_length / average_document_length);
        // k1 控制词频增长的饱和速度，b 控制文档长度对得分的影响程度。
        let numerator = term_frequency * (self.k1 + 1.0);
        let denominator = term_frequency + self.k1 * length_factor;

        idf * numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::{Bm25Ranker, Ranker, ScoreInput, TfIdfRanker};

    #[test]
    fn rarer_terms_get_higher_score() {
        let ranker = TfIdfRanker;
        let rare = ranker.score(input(10, 1, 1, 10, 10.0));
        let common = ranker.score(input(10, 9, 1, 10, 10.0));

        assert!(rare > common);
    }

    #[test]
    fn bm25_penalizes_long_documents() {
        let ranker = Bm25Ranker::default();
        let short_doc = ranker.score(input(10, 2, 2, 20, 100.0));
        let long_doc = ranker.score(input(10, 2, 2, 500, 100.0));

        assert!(short_doc > long_doc);
    }

    fn input(
        total_docs: usize,
        document_frequency: usize,
        term_frequency: usize,
        document_length: usize,
        average_document_length: f64,
    ) -> ScoreInput {
        ScoreInput {
            total_docs,
            document_frequency,
            term_frequency,
            document_length,
            average_document_length,
        }
    }
}
