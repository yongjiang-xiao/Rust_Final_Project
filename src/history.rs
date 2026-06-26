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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HistoryFilter {
    query_keyword: Option<String>,
    ranker: Option<String>,
}

impl HistoryFilter {
    pub fn new(query_keyword: Option<String>, ranker: Option<String>) -> Self {
        Self {
            query_keyword: normalize_filter_text(query_keyword),
            ranker: normalize_filter_text(ranker),
        }
    }

    pub fn is_active(&self) -> bool {
        self.query_keyword.is_some() || self.ranker.is_some()
    }

    pub fn matches(&self, entry: &HistoryEntry) -> bool {
        let query_matches = self
            .query_keyword
            .as_ref()
            .is_none_or(|keyword| entry.query.to_lowercase().contains(keyword));
        let ranker_matches = self
            .ranker
            .as_ref()
            .is_none_or(|ranker| entry.ranker.eq_ignore_ascii_case(ranker));

        query_matches && ranker_matches
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

pub fn filter_history<'a>(
    entries: &'a [HistoryEntry],
    filter: &HistoryFilter,
) -> Vec<&'a HistoryEntry> {
    entries
        .iter()
        .filter(|entry| filter.matches(entry))
        .collect()
}

pub fn clear_history(root: &Path) -> Result<Option<PathBuf>> {
    let history_path = history_file_path(root);
    if !history_path.exists() {
        return Ok(None);
    }

    fs::remove_file(&history_path)?;
    Ok(Some(history_path))
}

fn normalize_filter_text(value: Option<String>) -> Option<String> {
    let text = value?.trim().to_lowercase();
    if text.is_empty() { None } else { Some(text) }
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{
        HistoryEntry, HistoryFilter, append_history, clear_history, filter_history,
        history_file_path, load_history,
    };
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

    #[test]
    fn filters_history_by_query_keyword() {
        let entries = sample_entries();
        let filter = HistoryFilter::new(Some("Data".to_string()), None);

        let filtered = filter_history(&entries, &filter);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|entry| entry.query.contains("data")));
    }

    #[test]
    fn filters_history_by_ranker() {
        let entries = sample_entries();
        let filter = HistoryFilter::new(None, Some("bm25".to_string()));

        let filtered = filter_history(&entries, &filter);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|entry| entry.ranker == "bm25"));
    }

    #[test]
    fn combines_query_and_ranker_filters() {
        let entries = sample_entries();
        let filter = HistoryFilter::new(Some("data".to_string()), Some("tfidf".to_string()));

        let filtered = filter_history(&entries, &filter);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].query, "data structure");
        assert_eq!(filtered[0].ranker, "tfidf");
    }

    #[test]
    fn empty_history_filter_is_inactive() {
        let filter = HistoryFilter::new(Some("   ".to_string()), None);

        assert!(!filter.is_active());
    }

    #[test]
    fn clear_history_removes_existing_file() {
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

        let removed = clear_history(temp.path()).expect("clear");

        assert_eq!(removed, Some(history_file_path(temp.path())));
        assert!(!history_file_path(temp.path()).exists());
        assert!(load_history(temp.path()).expect("load").is_empty());
    }

    #[test]
    fn clear_history_reports_missing_file() {
        let temp = tempdir().expect("tempdir");

        let removed = clear_history(temp.path()).expect("clear");

        assert_eq!(removed, None);
    }

    fn sample_entries() -> Vec<HistoryEntry> {
        vec![
            HistoryEntry::new(
                "data structure".to_string(),
                "tfidf".to_string(),
                "none".to_string(),
                10,
                3,
                120,
            ),
            HistoryEntry::new(
                "rust ownership".to_string(),
                "bm25".to_string(),
                "rust".to_string(),
                10,
                2,
                90,
            ),
            HistoryEntry::new(
                "database data".to_string(),
                "bm25".to_string(),
                "database".to_string(),
                5,
                1,
                80,
            ),
        ]
    }
}
