use std::fs;

use rust_note_search::{
    document::{Document, scan_documents},
    index::build_index,
    tokenizer::{SimpleTokenizer, Tokenizer},
};
use tempfile::tempdir;

#[test]
fn builds_document_statistics() {
    let tokenizer = SimpleTokenizer;
    let docs = vec![
        document(0, "a.md", "A", "rust ownership"),
        document(1, "b.md", "B", "rust borrowing"),
    ];
    let index = build_index(docs, &tokenizer);

    assert_eq!(index.total_docs, 2);
    assert_eq!(index.total_tokens, 4);
}

#[test]
fn records_which_documents_contain_terms() {
    let tokenizer = SimpleTokenizer;
    let docs = vec![
        document(0, "a.md", "A", "rust ownership"),
        document(1, "b.md", "B", "rust borrowing"),
    ];
    let index = build_index(docs, &tokenizer);

    let postings = &index.inverted_index["rust"];
    assert_eq!(postings.len(), 2);
}

#[test]
fn records_term_frequency() {
    let tokenizer = SimpleTokenizer;
    let docs = vec![document(0, "a.md", "A", "rust rust ownership")];
    let index = build_index(docs, &tokenizer);

    assert_eq!(index.inverted_index["rust"][0].term_freq, 2);
}

#[test]
fn scanner_ignores_unsupported_files() {
    let temp = tempdir().expect("tempdir");
    fs::write(temp.path().join("note.md"), "# Note\nrust").expect("write note");
    fs::write(temp.path().join("image.png"), "ignored").expect("write png");

    let docs = scan_documents(temp.path(), &SimpleTokenizer).expect("scan");

    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].path, "note.md");
}

#[test]
fn scanner_handles_empty_directory() {
    let temp = tempdir().expect("tempdir");
    let docs = scan_documents(temp.path(), &SimpleTokenizer).expect("scan");

    assert!(docs.is_empty());
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
