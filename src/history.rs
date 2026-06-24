use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    pub timestamp_epoch_secs: u64,
    pub query: String,
    pub ranker: String,
    pub filter: String,
    pub top: usize,
    pub result_count: usize,
    pub elapsed_micros: u128,
}

impl HistoryEntry {
    pub fn new(
        query: String,
        ranker: String,
        filter: String,
        top: usize,
        result_count: usize,
        elapsed_micros: u128,
    ) -> Self {
        Self {
            timestamp_epoch_secs: current_epoch_seconds(),
            query,
            ranker,
            filter,
            top,
            result_count,
            elapsed_micros,
        }
    }
}

pub fn history_file_path(root: &Path) -> PathBuf {
    // 历史记录和索引一样放在 .rns 中，属于工具生成的辅助数据。
    root.join(".rns").join("history.json")
}

pub fn append_history(root: &Path, entry: HistoryEntry) -> Result<PathBuf> {
    // 先读取旧记录再追加，保持 history.json 是一个完整的 JSON 数组。
    let mut entries = load_history(root)?;
    entries.push(entry);

    let history_path = history_file_path(root);
    if let Some(parent) = history_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&history_path, serde_json::to_string_pretty(&entries)?)?;
    Ok(history_path)
}

pub fn load_history(root: &Path) -> Result<Vec<HistoryEntry>> {
    let history_path = history_file_path(root);
    if !history_path.exists() {
        return Ok(Vec::new());
    }

    let json = fs::read_to_string(history_path)?;
    Ok(serde_json::from_str(&json)?)
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{HistoryEntry, append_history, load_history};
    use tempfile::tempdir;

    #[test]
    fn appends_and_loads_history() {
        let temp = tempdir().expect("tempdir");
        append_history(
            temp.path(),
            HistoryEntry::new(
                "ownership".to_string(),
                "tfidf".to_string(),
                "none".to_string(),
                5,
                2,
                100,
            ),
        )
        .expect("append");

        let entries = load_history(temp.path()).expect("load");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].query, "ownership");
    }
}
