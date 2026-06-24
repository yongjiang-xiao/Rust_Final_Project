use rust_note_search::{
    document::Document,
    index::build_index,
    storage::{load_index, save_index},
    tokenizer::{SimpleTokenizer, Tokenizer},
};
use tempfile::tempdir;

#[test]
fn saves_and_loads_index() {
    let temp = tempdir().expect("tempdir");
    let tokenizer = SimpleTokenizer;
    let index = build_index(
        vec![Document {
            id: 0,
            path: "note.md".to_string(),
            title: "Note".to_string(),
            content: "rust ownership".to_string(),
            token_count: tokenizer.tokenize("rust ownership").len(),
        }],
        &tokenizer,
    );

    save_index(temp.path(), &index).expect("save");
    let loaded = load_index(temp.path()).expect("load");

    assert_eq!(loaded.total_docs, 1);
    assert!(loaded.inverted_index.contains_key("ownership"));
}
