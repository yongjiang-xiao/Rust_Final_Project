use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    document::Document,
    error::Result,
    query::{QueryMode, parse_query},
    search::{SearchOptions, apply_title_boost, title_matches},
    snippet::make_snippet,
    tokenizer::Tokenizer,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectSearchResult {
    pub doc_id: usize,
    pub path: String,
    pub title: String,
    pub score: f64,
    pub matched_terms: Vec<String>,
    pub title_matches: Vec<String>,
    pub snippet: String,
}

pub fn direct_search<T: Tokenizer>(
    documents: &[Document],
    query: &str,
    tokenizer: &T,
    options: &SearchOptions,
) -> Result<Vec<DirectSearchResult>> {
    // 方案 A 不依赖倒排索引，每次搜索都遍历全部文档，作为性能和排序效果的对照基线。
    let parsed_query = parse_query(query, tokenizer)?;
    let query_set: BTreeSet<String> = parsed_query.positive_terms.iter().cloned().collect();
    let excluded_set: BTreeSet<String> = parsed_query.excluded_terms.iter().cloned().collect();
    let mut results = Vec::new();

    for document in documents {
        if !options.path_filter.matches(&document.path) {
            continue;
        }

        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut has_excluded_term = false;
        // 直接扫描需要现场分词并统计词频，因此文档越多耗时越明显。
        for token in tokenizer.tokenize(&document.content) {
            if excluded_set.contains(&token) {
                has_excluded_term = true;
            }
            if query_set.contains(&token) {
                *counts.entry(token).or_default() += 1;
            }
        }

        if has_excluded_term || counts.is_empty() {
            continue;
        }
        if parsed_query.mode == QueryMode::All
            && !parsed_query
                .positive_terms
                .iter()
                .all(|term| counts.contains_key(term))
        {
            // 全部匹配模式对应用户输入的逻辑与查询，要求所有正向关键词都命中。
            continue;
        }

        let score = counts.values().sum::<usize>() as f64;
        let matched_terms: Vec<String> = counts.keys().cloned().collect();
        let title_matches = title_matches(&document.title, &matched_terms, tokenizer);
        let score = apply_title_boost(score, title_matches.len(), options.title_boost);
        let snippet = make_snippet(&document.content, query, &matched_terms, 120);
        results.push(DirectSearchResult {
            doc_id: document.id,
            path: document.path.clone(),
            title: document.title.clone(),
            score,
            matched_terms,
            title_matches,
            snippet,
        });
    }

    results.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.path.cmp(&right.path))
    });
    // 只保留用户需要展示的前若干条结果，减少命令行输出噪声。
    results.truncate(options.top);

    Ok(results)
}
