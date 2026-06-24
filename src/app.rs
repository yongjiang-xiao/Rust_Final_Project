use std::{path::Path, time::Instant};

use crate::{
    cli::{Cli, Commands, join_query},
    compare::{CompareReport, compare_plans},
    document::scan_documents,
    error::{AppError, Result},
    filter::PathFilter,
    history::{HistoryEntry, append_history, load_history},
    index::build_index,
    ranker::{Bm25Ranker, RankerKind, TfIdfRanker},
    search::{SearchResult, search_index},
    storage::{load_index, save_index},
    tokenizer::SimpleTokenizer,
};

pub fn run(cli: Cli) -> Result<()> {
    let tokenizer = SimpleTokenizer;
    // CLI 入口只负责分发命令，具体逻辑分别放到 run_index、run_search 等函数中。
    match cli.command {
        Commands::Index { dir } => run_index(&dir, &tokenizer),
        Commands::Search {
            query,
            path,
            top,
            ranker,
            filter,
            title_boost,
            no_history,
            explain,
        } => run_search(
            &path,
            &join_query(&query),
            top,
            ranker,
            filter,
            title_boost,
            !no_history,
            explain,
            &tokenizer,
        ),
        Commands::Compare {
            query,
            path,
            top,
            filter,
            title_boost,
        } => run_compare(
            &path,
            &join_query(&query),
            top,
            filter,
            title_boost,
            &tokenizer,
        ),
        Commands::History { path, limit } => run_history(&path, limit),
    }
}

fn run_index(root: &Path, tokenizer: &SimpleTokenizer) -> Result<()> {
    let documents = scan_documents(root, tokenizer)?;
    if documents.is_empty() {
        return Err(AppError::NoSupportedFiles(root.display().to_string()));
    }

    let index = build_index(documents, tokenizer);
    let index_path = save_index(root, &index)?;

    println!("Index built successfully.");
    println!("Documents: {}", index.total_docs);
    println!("Terms: {}", index.term_count());
    println!("Tokens: {}", index.total_tokens);
    println!("Saved: {}", index_path.display());

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_search(
    root: &Path,
    query: &str,
    top: usize,
    ranker: RankerKind,
    filter: Option<String>,
    title_boost: f64,
    record_history: bool,
    explain: bool,
    tokenizer: &SimpleTokenizer,
) -> Result<()> {
    let start = Instant::now();
    let index = load_index(root)?;
    let path_filter = PathFilter::new(filter);
    let options = crate::search::SearchOptions::new(top, path_filter.clone(), title_boost);
    // 根据命令行参数选择排序器，二者都通过 Ranker trait 进入同一套搜索流程。
    let results = match ranker {
        RankerKind::Tfidf => search_index(&index, query, tokenizer, &TfIdfRanker, &options)?,
        RankerKind::Bm25 => {
            search_index(&index, query, tokenizer, &Bm25Ranker::default(), &options)?
        }
    };
    let elapsed = start.elapsed();

    print_search_results(&results, explain);

    if record_history {
        // 默认记录搜索历史；用户传入 --no-history 时会跳过这里。
        let history_path = append_history(
            root,
            HistoryEntry::new(
                query.to_string(),
                ranker.label().to_string(),
                path_filter.label(),
                top,
                results.len(),
                elapsed.as_micros(),
            ),
        )?;
        println!("History: {}", history_path.display());
    }

    Ok(())
}

fn run_compare(
    root: &Path,
    query: &str,
    top: usize,
    filter: Option<String>,
    title_boost: f64,
    tokenizer: &SimpleTokenizer,
) -> Result<()> {
    let report = compare_plans(
        root,
        query,
        tokenizer,
        top,
        PathFilter::new(filter),
        title_boost,
    )?;
    print_compare_report(&report);
    Ok(())
}

fn run_history(root: &Path, limit: usize) -> Result<()> {
    let entries = load_history(root)?;
    if entries.is_empty() {
        println!("No search history found.");
        return Ok(());
    }

    for entry in entries.iter().rev().take(limit) {
        println!("Time: {}", entry.timestamp_epoch_secs);
        println!("Query: {}", entry.query);
        println!("Ranker: {}", entry.ranker);
        println!("Filter: {}", entry.filter);
        println!("Top: {}", entry.top);
        println!("Results: {}", entry.result_count);
        println!("Elapsed: {} us", entry.elapsed_micros);
        println!();
    }

    Ok(())
}

fn print_search_results(results: &[SearchResult], explain: bool) {
    if results.is_empty() {
        println!("No results found.");
        return;
    }

    for (index, result) in results.iter().enumerate() {
        println!("[{}] {}", index + 1, result.title);
        println!("Path: {}", result.path);
        println!("Score: {:.3}", result.score);
        println!("Ranker: {}", result.ranker);
        println!("Matched: {}", result.matched_terms.join(", "));
        if !result.title_matches.is_empty() {
            println!("Title matches: {}", result.title_matches.join(", "));
        }
        println!("Snippet: {}", result.snippet);
        if explain {
            print_result_explanation(result);
        }
        println!();
    }
}

fn print_result_explanation(result: &SearchResult) {
    println!("Explain:");
    println!("  Query mode: {}", result.explanation.query_mode);
    println!("  Ranker: {}", result.explanation.ranker);
    println!("  Base score: {:.3}", result.explanation.base_score);
    println!(
        "  Title boost multiplier: {:.3}",
        result.explanation.title_boost_multiplier
    );
    println!("  Final score: {:.3}", result.explanation.final_score);
    if !result.explanation.excluded_terms.is_empty() {
        println!(
            "  Excluded terms: {}",
            result.explanation.excluded_terms.join(", ")
        );
    }
    for term in &result.explanation.term_explanations {
        println!(
            "  Term `{}`: tf={}, df={}, doc_len={}, score={:.3}",
            term.term, term.term_freq, term.document_frequency, term.document_length, term.score
        );
    }
}

fn print_compare_report(report: &CompareReport) {
    println!("Query: {}", report.query);
    println!("Filter: {}", report.filter);
    println!("Title boost: {:.2}", report.title_boost);
    println!();

    for summary in &report.summaries {
        println!("{}", summary.name);
        println!(
            "Need index: {}",
            if summary.need_index { "yes" } else { "no" }
        );
        println!("Ranking method: {}", summary.ranking_method);
        println!("Matched documents: {}", summary.matched_documents);
        println!("Elapsed: {} us", summary.elapsed_micros);
        if let Some(top_result) = &summary.top_result {
            println!("Top result: {top_result}");
        }
        if let Some(top_score) = summary.top_score {
            println!("Top score: {top_score:.3}");
        }
        println!();
    }
}
