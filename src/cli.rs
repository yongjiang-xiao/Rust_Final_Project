use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::ranker::RankerKind;

#[derive(Debug, Parser)]
#[command(
    name = "rust-note-search",
    version,
    about = "A local Markdown/TXT knowledge base search tool written in Rust"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Scan a knowledge base directory and build .rns/index.json.
    Index {
        /// Knowledge base directory.
        dir: PathBuf,
    },

    /// Search from an existing index.
    Search {
        /// Query words. Multiple words are joined with spaces.
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,

        /// Knowledge base directory containing .rns/index.json.
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// Maximum number of results to show.
        #[arg(long, default_value_t = 10)]
        top: usize,

        /// Ranking algorithm used by indexed search.
        #[arg(long, value_enum, default_value = "tfidf")]
        ranker: RankerKind,

        /// Only return documents whose relative path matches this path or directory.
        #[arg(long)]
        filter: Option<String>,

        /// Score multiplier for documents whose title contains matched query terms.
        #[arg(long, default_value_t = 1.5)]
        title_boost: f64,

        /// Do not append this query to .rns/history.json.
        #[arg(long)]
        no_history: bool,

        /// Print score details for each result.
        #[arg(long)]
        explain: bool,
    },

    /// Compare direct scan search with inverted-index search.
    Compare {
        /// Query words. Multiple words are joined with spaces.
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,

        /// Knowledge base directory.
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// Maximum number of top results to show.
        #[arg(long, default_value_t = 5)]
        top: usize,

        /// Only compare documents whose relative path matches this path or directory.
        #[arg(long)]
        filter: Option<String>,

        /// Score multiplier for documents whose title contains matched query terms.
        #[arg(long, default_value_t = 1.5)]
        title_boost: f64,
    },

    /// Show recent search history recorded under .rns/history.json.
    History {
        /// Knowledge base directory containing .rns/history.json.
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// Maximum number of history entries to show.
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
}

pub fn join_query(query: &[String]) -> String {
    query.join(" ")
}
