use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use crate::{
    error::{AppError, Result},
    markdown::clean_document_content,
    tokenizer::Tokenizer,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub id: usize,
    pub path: String,
    pub title: String,
    pub content: String,
    pub token_count: usize,
}

pub fn scan_documents<T: Tokenizer>(root: &Path, tokenizer: &T) -> Result<Vec<Document>> {
    if !root.exists() {
        return Err(AppError::InvalidPath(format!(
            "{} does not exist",
            root.display()
        )));
    }
    if !root.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "{} is not a directory",
            root.display()
        )));
    }

    // 先收集所有支持的文档路径，再排序，保证索引构建结果稳定可复现。
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(should_keep_entry)
    {
        let entry = entry?;
        if entry.file_type().is_file() && is_supported_file(entry.path()) {
            files.push(entry.path().to_path_buf());
        }
    }

    files.sort_by_key(|path| relative_path(root, path));

    let mut documents = Vec::with_capacity(files.len());
    for path in files {
        let raw_content = fs::read_to_string(&path)?;
        // 标题和正文分别处理：标题用于展示和加权，正文用于清洗后参与索引。
        let title = extract_title(&path, &raw_content);
        let content = clean_document_content(&path, &raw_content);
        let token_count = tokenizer.tokenize(&content).len();
        let id = documents.len();
        documents.push(Document {
            id,
            path: relative_path(root, &path),
            title,
            content,
            token_count,
        });
    }

    Ok(documents)
}

pub fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| matches!(extension.to_ascii_lowercase().as_str(), "md" | "txt"))
        .unwrap_or(false)
}

pub fn extract_title(path: &Path, content: &str) -> String {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if extension == "md"
        && let Some(title) = content.lines().find_map(markdown_heading)
    {
        return title;
    }

    // TXT 或没有一级标题的 Markdown 使用文件名作为标题。
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

fn markdown_heading(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let title = trimmed.strip_prefix("# ")?;
    let title = title.trim();
    if title.is_empty() {
        None
    } else {
        Some(title.to_string())
    }
}

fn should_keep_entry(entry: &DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }

    // 这些目录属于工程或搜索缓存，不应该进入知识库扫描范围。
    let ignored = ["target", ".git", ".rns", "node_modules"];
    entry
        .file_name()
        .to_str()
        .map(|name| !ignored.contains(&name))
        .unwrap_or(true)
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::extract_title;

    #[test]
    fn extracts_first_markdown_heading() {
        let title = extract_title(Path::new("note.md"), "text\n# Real Title\nbody");
        assert_eq!(title, "Real Title");
    }

    #[test]
    fn falls_back_to_file_name() {
        let title = extract_title(Path::new("plain.txt"), "hello");
        assert_eq!(title, "plain");
    }
}
