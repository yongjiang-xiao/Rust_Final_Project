use rust_note_search::tokenizer::{SimpleTokenizer, Tokenizer};

#[test]
fn tokenizes_english_and_lowercase() {
    let tokenizer = SimpleTokenizer;
    assert_eq!(
        tokenizer.tokenize("Rust Ownership!"),
        vec!["rust", "ownership"]
    );
}

#[test]
fn tokenizes_multiple_english_words() {
    let tokenizer = SimpleTokenizer;
    assert_eq!(
        tokenizer.tokenize("borrowed values"),
        vec!["borrowed", "values"]
    );
}

#[test]
fn tokenizes_chinese_ownership_bigram() {
    let tokenizer = SimpleTokenizer;
    assert_eq!(tokenizer.tokenize("所有权"), vec!["所有", "有权"]);
}

#[test]
fn tokenizes_chinese_dynamic_programming_bigram() {
    let tokenizer = SimpleTokenizer;
    assert_eq!(tokenizer.tokenize("动态规划"), vec!["动态", "态规", "规划"]);
}

#[test]
fn empty_input_returns_no_tokens() {
    let tokenizer = SimpleTokenizer;
    assert!(tokenizer.tokenize("   ").is_empty());
}
