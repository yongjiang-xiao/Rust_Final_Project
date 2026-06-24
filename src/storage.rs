use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    error::{AppError, Result},
    index::SearchIndex,
};

const INDEX_DIR: &str = ".rns";
const INDEX_FILE: &str = "index.json";

pub fn index_file_path(root: &Path) -> PathBuf {
    // 将索引统一保存到知识库目录下的 .rns/index.json，避免和原始笔记混在一起。
    root.join(INDEX_DIR).join(INDEX_FILE)
}

pub fn save_index(root: &Path, index: &SearchIndex) -> Result<PathBuf> {
    let index_path = index_file_path(root);
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent)?;
    }
    // 使用 pretty JSON 便于调试和截图展示，也方便报告中说明索引文件结构。
    let json = serde_json::to_string_pretty(index)?;
    fs::write(&index_path, json)?;
    Ok(index_path)
}

pub fn load_index(root: &Path) -> Result<SearchIndex> {
    let index_path = index_file_path(root);
    if !index_path.exists() {
        return Err(AppError::IndexNotFound(index_path.display().to_string()));
    }

    let json = fs::read_to_string(&index_path)?;
    Ok(serde_json::from_str(&json)?)
}
