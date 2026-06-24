use std::{path::Path, time::Instant};

use crate::{
    direct_search::direct_search,
    document::scan_documents,
    error::Result,
    filter::PathFilter,
    index::SearchIndex,
    ranker::{Bm25Ranker, TfIdfRanker},
    search::{SearchOptions, SearchResult, search_index},
    storage::load_index,
    tokenizer::SimpleTokenizer,
};

#[derive(Debug, Clone)]
pub struct PlanSummary {
    pub name: String,
    pub need_index: bool,
    pub ranking_method: String,
    pub matched_documents: usize,
    pub elapsed_micros: u128,
    pub top_result: Option<String>,
    pub top_score: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct CompareReport {
    pub query: String,
    pub filter: String,
    pub title_boost: f64,
    pub summaries: Vec<PlanSummary>,
}

pub fn compare_plans(
    root: &Path,
    query: &str,
    tokenizer: &SimpleTokenizer,
    top: usize,
    path_filter: PathFilter,
    title_boost: f64,
) -> Result<CompareReport> {
    // 三种方案使用同一组查询参数，保证耗时、命中数量和首条结果具有可比性。
    let options = SearchOptions::new(top, path_filter.clone(), title_boost);
    let direct = run_direct_scan(root, query, tokenizer, &options)?;
    let index = load_index(root)?;
    let tfidf = run_tfidf(&index, query, tokenizer, &options)?;
    let bm25 = run_bm25(&index, query, tokenizer, &options)?;

    Ok(CompareReport {
        query: query.to_string(),
        filter: path_filter.label(),
        title_boost,
        summaries: vec![direct, tfidf, bm25],
    })
}

fn run_direct_scan(
    root: &Path,
    query: &str,
    tokenizer: &SimpleTokenizer,
    options: &SearchOptions,
) -> Result<PlanSummary> {
    let start = Instant::now();
    // 方案 A 每次都重新扫描文件，体现“无需索引但效率较低”的特点。
    let documents = scan_documents(root, tokenizer)?;
    let results = direct_search(&documents, query, tokenizer, options)?;
    let elapsed = start.elapsed();

    Ok(PlanSummary {
        name: "Plan A: Direct Scan".to_string(),
        need_index: false,
        ranking_method: "keyword frequency + title boost".to_string(),
        matched_documents: results.len(),
        elapsed_micros: elapsed.as_micros(),
        top_result: results.first().map(|result| result.path.clone()),
        top_score: results.first().map(|result| result.score),
    })
}

fn run_tfidf(
    index: &SearchIndex,
    query: &str,
    tokenizer: &SimpleTokenizer,
    options: &SearchOptions,
) -> Result<PlanSummary> {
    let start = Instant::now();
    // 方案 B 复用倒排索引，再用 TF-IDF 根据词频和文档频率计算相关性。
    let results = search_index(index, query, tokenizer, &TfIdfRanker, options)?;
    let elapsed = start.elapsed();
    Ok(index_summary(
        "Plan B: Inverted Index + TF-IDF",
        "TF-IDF score + title boost",
        results,
        elapsed.as_micros(),
    ))
}

fn run_bm25(
    index: &SearchIndex,
    query: &str,
    tokenizer: &SimpleTokenizer,
    options: &SearchOptions,
) -> Result<PlanSummary> {
    let start = Instant::now();
    // 方案 C 在倒排索引上使用 BM25，额外考虑文档长度对得分的影响。
    let results = search_index(index, query, tokenizer, &Bm25Ranker::default(), options)?;
    let elapsed = start.elapsed();
    Ok(index_summary(
        "Plan C: Inverted Index + BM25",
        "BM25 score + document length normalization + title boost",
        results,
        elapsed.as_micros(),
    ))
}

fn index_summary(
    name: &str,
    ranking_method: &str,
    results: Vec<SearchResult>,
    elapsed_micros: u128,
) -> PlanSummary {
    PlanSummary {
        name: name.to_string(),
        need_index: true,
        ranking_method: ranking_method.to_string(),
        matched_documents: results.len(),
        elapsed_micros,
        top_result: results.first().map(|result| result.path.clone()),
        top_score: results.first().map(|result| result.score),
    }
}
