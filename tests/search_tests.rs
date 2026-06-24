use rust_note_search::{
    document::Document,
    error::AppError,
    filter::PathFilter,
    index::build_index,
    ranker::{Bm25Ranker, TfIdfRanker},
    search::{SearchOptions, search_index},
    tokenizer::{SimpleTokenizer, Tokenizer},
};

#[test]
fn single_keyword_returns_matching_document() {
    let tokenizer = SimpleTokenizer;
    let index = build_index(
        vec![
            document(0, "ownership.md", "Ownership", "ownership borrowing"),
            document(1, "graph.md", "Graph", "graph search"),
        ],
        &tokenizer,
    );

    let results =
        search_index(&index, "ownership", &tokenizer, &TfIdfRanker, &options(10)).expect("search");

    assert_eq!(results[0].path, "ownership.md");
}

#[test]
fn multi_keyword_improves_relevant_ranking() {
    let tokenizer = SimpleTokenizer;
    let index = build_index(
        vec![
            document(0, "rust.md", "Rust", "rust rust"),
            document(1, "ownership.md", "Ownership", "rust ownership ownership"),
        ],
        &tokenizer,
    );

    let results = search_index(
        &index,
        "rust ownership",
        &tokenizer,
        &TfIdfRanker,
        &options(10),
    )
    .expect("search");

    assert_eq!(results[0].path, "ownership.md");
}

#[test]
fn missing_keyword_returns_empty_result() {
    let tokenizer = SimpleTokenizer;
    let index = build_index(vec![document(0, "rust.md", "Rust", "rust")], &tokenizer);

    let results =
        search_index(&index, "database", &tokenizer, &TfIdfRanker, &options(10)).expect("search");

    assert!(results.is_empty());
}

#[test]
fn empty_query_returns_error() {
    let tokenizer = SimpleTokenizer;
    let index = build_index(vec![document(0, "rust.md", "Rust", "rust")], &tokenizer);

    let error = search_index(&index, "   ", &tokenizer, &TfIdfRanker, &options(10))
        .expect_err("empty query should fail");

    assert!(matches!(error, AppError::EmptyQuery));
}

#[test]
fn search_result_contains_snippet_and_matched_terms() {
    let tokenizer = SimpleTokenizer;
    let index = build_index(
        vec![document(
            0,
            "ownership.md",
            "Ownership",
            "Rust ownership means each value has one owner.",
        )],
        &tokenizer,
    );

    let results =
        search_index(&index, "ownership", &tokenizer, &TfIdfRanker, &options(10)).expect("search");

    assert_eq!(results[0].matched_terms, vec!["ownership"]);
    assert!(results[0].snippet.contains("**ownership**"));
}

#[test]
fn path_filter_limits_results() {
    let tokenizer = SimpleTokenizer;
    let index = build_index(
        vec![
            document(0, "rust/ownership.md", "Ownership", "ownership"),
            document(1, "database/transaction.txt", "Transaction", "ownership"),
        ],
        &tokenizer,
    );
    let search_options = SearchOptions::new(10, PathFilter::new(Some("rust".to_string())), 1.5);

    let results = search_index(
        &index,
        "ownership",
        &tokenizer,
        &TfIdfRanker,
        &search_options,
    )
    .expect("search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].path, "rust/ownership.md");
}

#[test]
fn title_boost_improves_title_match() {
    let tokenizer = SimpleTokenizer;
    let index = build_index(
        vec![
            document(0, "body.md", "General Note", "ownership ownership"),
            document(1, "title.md", "Ownership Guide", "ownership"),
        ],
        &tokenizer,
    );
    let search_options = SearchOptions::new(10, PathFilter::default(), 3.0);

    let results = search_index(
        &index,
        "ownership",
        &tokenizer,
        &TfIdfRanker,
        &search_options,
    )
    .expect("search");

    assert_eq!(results[0].path, "title.md");
    assert_eq!(results[0].title_matches, vec!["ownership"]);
}

#[test]
fn bm25_search_returns_ranker_label() {
    let tokenizer = SimpleTokenizer;
    let index = build_index(
        vec![document(0, "rust.md", "Rust", "rust ownership")],
        &tokenizer,
    );

    let results = search_index(
        &index,
        "ownership",
        &tokenizer,
        &Bm25Ranker::default(),
        &options(10),
    )
    .expect("search");

    assert_eq!(results[0].ranker, "bm25");
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

fn options(top: usize) -> SearchOptions {
    SearchOptions::new(top, PathFilter::default(), 1.5)
}
