use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    // 统一错误类型让各模块可以用 ? 传播错误，程序入口只需要处理一种结果类型。
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("directory walk error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("query is empty")]
    EmptyQuery,

    #[error("index not found at {0}; please run: rust-note-search index <DIR>")]
    IndexNotFound(String),

    #[error("no supported .md or .txt files found in {0}")]
    NoSupportedFiles(String),
}
