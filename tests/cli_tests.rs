use std::fs;

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

#[test]
fn cli_indexes_and_searches() {
    let temp = tempdir().expect("tempdir");
    fs::write(
        temp.path().join("ownership.md"),
        "# Ownership\nRust ownership and borrowing.",
    )
    .expect("write note");

    Command::cargo_bin("rust-note-search")
        .expect("binary")
        .args(["index", temp.path().to_str().expect("utf8 path")])
        .assert()
        .success()
        .stdout(contains("Index built successfully"));

    Command::cargo_bin("rust-note-search")
        .expect("binary")
        .args([
            "search",
            "ownership",
            "--path",
            temp.path().to_str().expect("utf8 path"),
        ])
        .assert()
        .success()
        .stdout(contains("ownership.md"));
}

#[test]
fn cli_search_supports_bm25_filter_and_history() {
    let temp = tempdir().expect("tempdir");
    let rust_dir = temp.path().join("rust");
    let db_dir = temp.path().join("database");
    fs::create_dir_all(&rust_dir).expect("rust dir");
    fs::create_dir_all(&db_dir).expect("db dir");
    fs::write(
        rust_dir.join("ownership.md"),
        "# Ownership\nRust [ownership](https://example.com) and borrowing.",
    )
    .expect("write rust note");
    fs::write(db_dir.join("transaction.txt"), "ownership in database").expect("write db note");

    Command::cargo_bin("rust-note-search")
        .expect("binary")
        .args(["index", temp.path().to_str().expect("utf8 path")])
        .assert()
        .success();

    Command::cargo_bin("rust-note-search")
        .expect("binary")
        .args([
            "search",
            "ownership",
            "--path",
            temp.path().to_str().expect("utf8 path"),
            "--ranker",
            "bm25",
            "--filter",
            "rust",
        ])
        .assert()
        .success()
        .stdout(contains("Ranker: bm25"))
        .stdout(contains("rust/ownership.md"));

    Command::cargo_bin("rust-note-search")
        .expect("binary")
        .args([
            "history",
            "--path",
            temp.path().to_str().expect("utf8 path"),
        ])
        .assert()
        .success()
        .stdout(contains("Query: ownership"))
        .stdout(contains("Ranker: bm25"));
}

#[test]
fn cli_compare_prints_three_plans() {
    let temp = tempdir().expect("tempdir");
    fs::write(temp.path().join("rust.md"), "# Rust\nRust ownership.").expect("write note");

    Command::cargo_bin("rust-note-search")
        .expect("binary")
        .args(["index", temp.path().to_str().expect("utf8 path")])
        .assert()
        .success();

    Command::cargo_bin("rust-note-search")
        .expect("binary")
        .args([
            "compare",
            "ownership",
            "--path",
            temp.path().to_str().expect("utf8 path"),
        ])
        .assert()
        .success()
        .stdout(contains("Plan A: Direct Scan"))
        .stdout(contains("Plan B: Inverted Index + TF-IDF"))
        .stdout(contains("Plan C: Inverted Index + BM25"));
}

#[test]
fn cli_search_supports_query_syntax_and_explain() {
    let temp = tempdir().expect("tempdir");
    fs::write(
        temp.path().join("rust.md"),
        "# Rust Ownership\nRust ownership borrowing.",
    )
    .expect("write rust note");
    fs::write(
        temp.path().join("database.md"),
        "# Database Ownership\nRust ownership database.",
    )
    .expect("write database note");

    Command::cargo_bin("rust-note-search")
        .expect("binary")
        .args(["index", temp.path().to_str().expect("utf8 path")])
        .assert()
        .success();

    Command::cargo_bin("rust-note-search")
        .expect("binary")
        .args([
            "search",
            "--path",
            temp.path().to_str().expect("utf8 path"),
            "--explain",
            "--no-history",
            "--",
            "rust",
            "AND",
            "ownership",
            "-database",
        ])
        .assert()
        .success()
        .stdout(contains("rust.md"))
        .stdout(contains("Explain:"))
        .stdout(contains("Query mode: AND"))
        .stdout(contains("Excluded terms: database"));
}
