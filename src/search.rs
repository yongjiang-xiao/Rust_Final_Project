use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    error::Result,
    explain::{ScoreExplanation, TermExplanation},
    filter::PathFilter,
    index::SearchIndex,
    query::parse_query,
    ranker::{Ranker, ScoreInput},
    snippet::make_snippet,
    tokenizer::Tokenizer,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub doc_id: usize,
    pub path: String,
    pub title: String,
    pub score: f64,
    pub ranker: String,
    pub matched_terms: Vec<String>,
    pub title_matches: Vec<String>,
    pub explanation: ScoreExplanation,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub top: usize,
    pub path_filter: PathFilter,
    pub title_boost: f64,
}

impl SearchOptions {
    pub fn new(top: usize, path_filter: PathFilter, title_boost: f64) -> Self {
        Self {
            top,
            path_filter,
            title_boost: title_boost.max(1.0),
        }
    }
}

#[derive(Debug, Default)]
struct SearchAccumulator {
    // 同一篇文档可能命中多个查询词，先在 accumulator 中累加分数和解释信息。
    score: f64,
    matched_terms: BTreeSet<String>,
    term_explanations: BTreeMap<String, TermExplanation>,
}

pub fn search_index<T, R>(
    index: &SearchIndex,
    query: &str,
    tokenizer: &T,
    ranker: &R,
    options: &SearchOptions,
) -> Result<Vec<SearchResult>>
where
    T: Tokenizer,
    R: Ranker,
{
    let parsed_query = parse_query(query, tokenizer)?;
    let mut accumulators: BTreeMap<usize, SearchAccumulator> = BTreeMap::new();
    let average_document_length = index.average_document_length();
    // 排除词先转换成文档集合，后续遍历 postings 时可以快速跳过。
    let excluded_doc_ids = excluded_doc_ids(index, &parsed_query.excluded_terms);

    // 倒排索引只遍历命中查询词的 postings，避免每次搜索重新扫描所有文档。
    for term in &parsed_query.positive_terms {
        let Some(postings) = index.inverted_index.get(term) else {
            continue;
        };
        let document_frequency = postings.len();

        for posting in postings {
            if excluded_doc_ids.contains(&posting.doc_id) {
                continue;
            }

            let Some(document) = index.document_by_id(posting.doc_id) else {
                continue;
            };
            if !options.path_filter.matches(&document.path) {
                continue;
            }

            // Ranker trait 屏蔽 TF-IDF/BM25 差异，搜索流程只依赖统一打分接口。
            let score = ranker.score(ScoreInput {
                total_docs: index.total_docs,
                document_frequency,
                term_frequency: posting.term_freq,
                document_length: document.token_count,
                average_document_length,
            });
            let accumulator = accumulators.entry(posting.doc_id).or_default();
            // 多关键词查询时，每个命中词单独打分，再累加成该文档的基础相关度。
            accumulator.score += score;
            accumulator.matched_terms.insert(term.clone());
            accumulator.term_explanations.insert(
                term.clone(),
                TermExplanation {
                    term: term.clone(),
                    term_freq: posting.term_freq,
                    document_frequency,
                    document_length: document.token_count,
                    score,
                },
            );
        }
    }

    let mut results = Vec::with_capacity(accumulators.len());
    // 第二阶段把候选文档转换成可展示的 SearchResult，包括标题命中、摘要和解释信息。
    for (doc_id, accumulator) in accumulators {
        // AND 查询要求候选文档命中所有正向查询词；默认 OR 查询不做该限制。
        if parsed_query.requires_all_terms()
            && !has_all_required_terms(&accumulator.matched_terms, &parsed_query.positive_terms)
        {
            continue;
        }

        if let Some(document) = index.document_by_id(doc_id) {
            let matched_terms: Vec<String> = accumulator.matched_terms.into_iter().collect();
            let title_matches = title_matches(&document.title, &matched_terms, tokenizer);
            // 标题命中通常说明文档主题更相关，因此在基础分上乘以标题权重。
            let title_boost_multiplier =
                title_boost_multiplier(title_matches.len(), options.title_boost);
            let score = accumulator.score * title_boost_multiplier;
            let snippet = make_snippet(&document.content, query, &matched_terms, 120);
            let term_explanations = accumulator.term_explanations.into_values().collect();
            // explain 输出复用这里记录的中间分数，避免展示结果和真实排序逻辑不一致。
            let explanation = ScoreExplanation {
                query_mode: parsed_query.mode.label().to_string(),
                excluded_terms: parsed_query.excluded_terms.clone(),
                ranker: ranker.name().to_string(),
                base_score: accumulator.score,
                title_boost_multiplier,
                final_score: score,
                term_explanations,
            };
            results.push(SearchResult {
                doc_id,
                path: document.path.clone(),
                title: document.title.clone(),
                score,
                ranker: ranker.name().to_string(),
                matched_terms,
                title_matches,
                explanation,
                snippet,
            });
        }
    }

    results.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
    });
    // top 参数只影响最终展示数量，不影响索引构建和候选文档打分。
    results.truncate(options.top);

    Ok(results)
}

pub fn normalized_query_terms<T: Tokenizer>(query: &str, tokenizer: &T) -> Result<Vec<String>> {
    let parsed_query = parse_query(query, tokenizer)?;
    Ok(parsed_query.positive_terms)
}

pub fn title_matches<T: Tokenizer>(
    title: &str,
    matched_terms: &[String],
    tokenizer: &T,
) -> Vec<String> {
    let title_terms: BTreeSet<String> = tokenizer.tokenize(title).into_iter().collect();
    matched_terms
        .iter()
        .filter(|term| title_terms.contains(*term))
        .cloned()
        .collect()
}

pub fn apply_title_boost(score: f64, title_match_count: usize, title_boost: f64) -> f64 {
    score * title_boost_multiplier(title_match_count, title_boost)
}

pub fn title_boost_multiplier(title_match_count: usize, title_boost: f64) -> f64 {
    if title_match_count == 0 {
        1.0
    } else {
        1.0 + (title_boost.max(1.0) - 1.0) * title_match_count as f64
    }
}

pub fn document_has_excluded_terms(
    index: &SearchIndex,
    doc_id: usize,
    excluded_terms: &[String],
) -> bool {
    excluded_terms.iter().any(|term| {
        index
            .inverted_index
            .get(term)
            .map(|postings| postings.iter().any(|posting| posting.doc_id == doc_id))
            .unwrap_or(false)
    })
}

fn excluded_doc_ids(index: &SearchIndex, excluded_terms: &[String]) -> BTreeSet<usize> {
    let mut doc_ids = BTreeSet::new();
    for term in excluded_terms {
        if let Some(postings) = index.inverted_index.get(term) {
            doc_ids.extend(postings.iter().map(|posting| posting.doc_id));
        }
    }
    doc_ids
}

fn has_all_required_terms(matched_terms: &BTreeSet<String>, positive_terms: &[String]) -> bool {
    positive_terms
        .iter()
        .all(|term| matched_terms.contains(term))
}

#[cfg(test)]
mod tests {
    use crate::{
        document::Document,
        explain::ScoreExplanation,
        filter::PathFilter,
        index::build_index,
        ranker::TfIdfRanker,
        search::{SearchOptions, search_index},
        tokenizer::{SimpleTokenizer, Tokenizer},
    };

    #[test]
    fn multi_term_query_ranks_more_specific_document_first() {
        let tokenizer = SimpleTokenizer;
        let documents = vec![
            document(0, "a.md", "Rust", "rust rust"),
            document(1, "b.md", "Ownership", "rust ownership ownership"),
        ];
        let index = build_index(documents, &tokenizer);
        let options = SearchOptions::new(10, PathFilter::default(), 1.5);
        let results = search_index(&index, "rust ownership", &tokenizer, &TfIdfRanker, &options)
            .expect("search should succeed");

        assert_eq!(results[0].path, "b.md");
    }

    #[test]
    fn and_query_requires_all_terms() {
        let tokenizer = SimpleTokenizer;
        let documents = vec![
            document(0, "a.md", "Rust", "rust rust"),
            document(1, "b.md", "Ownership", "rust ownership"),
        ];
        let index = build_index(documents, &tokenizer);
        let options = SearchOptions::new(10, PathFilter::default(), 1.5);

        let results = search_index(
            &index,
            "rust AND ownership",
            &tokenizer,
            &TfIdfRanker,
            &options,
        )
        .expect("search should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "b.md");
        assert_eq!(results[0].explanation.query_mode, "AND");
    }

    #[test]
    fn excluded_terms_remove_documents() {
        let tokenizer = SimpleTokenizer;
        let documents = vec![
            document(0, "rust.md", "Rust", "rust ownership"),
            document(1, "database.md", "Database", "rust ownership database"),
        ];
        let index = build_index(documents, &tokenizer);
        let options = SearchOptions::new(10, PathFilter::default(), 1.5);

        let results = search_index(
            &index,
            "rust ownership -database",
            &tokenizer,
            &TfIdfRanker,
            &options,
        )
        .expect("search should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "rust.md");
        assert_eq!(results[0].explanation.excluded_terms, vec!["database"]);
    }

    #[test]
    fn explanation_records_term_frequencies() {
        let tokenizer = SimpleTokenizer;
        let documents = vec![document(0, "a.md", "Ownership", "ownership ownership")];
        let index = build_index(documents, &tokenizer);
        let options = SearchOptions::new(10, PathFilter::default(), 1.5);

        let results =
            search_index(&index, "ownership", &tokenizer, &TfIdfRanker, &options).expect("search");
        let explanation: &ScoreExplanation = &results[0].explanation;

        assert_eq!(explanation.term_explanations[0].term, "ownership");
        assert_eq!(explanation.term_explanations[0].term_freq, 2);
        assert!(explanation.title_boost_applied());
    }

    fn document(id: usize, path: &str, title: &str, content: &str) -> Document {
        let tokenizer = SimpleTokenizer;
        Document {
            id,
            path: path.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            token_count: tokenizer.tokenize(content).len(),
        }
    }
}
