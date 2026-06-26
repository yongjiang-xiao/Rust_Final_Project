use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::ranker::RankerKind;

#[derive(Debug, Parser)]
#[command(
    name = "rust-note-search",
    version,
    about = "使用 Rust 实现的本地 Markdown/TXT 知识库搜索工具"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// 扫描知识库目录并构建 .rns/index.json 索引文件。
    Index {
        /// 知识库目录。
        dir: PathBuf,
    },

    /// 基于已有索引执行搜索。
    Search {
        /// 查询词；多个词会用空格拼接为完整查询表达式。
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,

        /// 包含 .rns/index.json 的知识库目录。
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// 最多显示的搜索结果数量。
        #[arg(long, default_value_t = 10)]
        top: usize,

        /// 索引搜索使用的排序算法。
        #[arg(long, value_enum, default_value = "tfidf")]
        ranker: RankerKind,

        /// 只返回相对路径匹配该路径或目录关键词的文档。
        #[arg(long)]
        filter: Option<String>,

        /// 标题命中查询词时使用的得分加权倍数。
        #[arg(long, default_value_t = 1.5)]
        title_boost: f64,

        /// 本次查询不追加到 .rns/history.json。
        #[arg(long)]
        no_history: bool,

        /// 输出每条结果的评分细节。
        #[arg(long)]
        explain: bool,
    },

    /// 对比直接全文扫描和倒排索引搜索方案。
    Compare {
        /// 查询词；多个词会用空格拼接为完整查询表达式。
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,

        /// 知识库目录。
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// 每种方案最多保留的结果数量。
        #[arg(long, default_value_t = 5)]
        top: usize,

        /// 只对比相对路径匹配该路径或目录关键词的文档。
        #[arg(long)]
        filter: Option<String>,

        /// 标题命中查询词时使用的得分加权倍数。
        #[arg(long, default_value_t = 1.5)]
        title_boost: f64,
    },

    /// 显示 .rns/history.json 中记录的最近搜索历史。
    History {
        /// 包含 .rns/history.json 的知识库目录。
        #[arg(long, default_value = ".")]
        path: PathBuf,

        /// 最多显示的历史记录条数。
        #[arg(long, default_value_t = 10)]
        limit: usize,

        /// 只显示查询内容中包含该关键词的历史记录。
        #[arg(long)]
        query: Option<String>,

        /// 只显示指定排序器产生的历史记录。
        #[arg(long, value_enum)]
        ranker: Option<RankerKind>,

        /// 清空当前知识库目录下的搜索历史。
        #[arg(long)]
        clear: bool,
    },
}

pub fn join_query(query: &[String]) -> String {
    // clap 会把多个查询片段收集到 Vec 中，这里还原为用户输入的完整查询表达式。
    query.join(" ")
}
