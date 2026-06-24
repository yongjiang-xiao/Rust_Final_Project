use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathFilter {
    pattern: Option<String>,
}

impl PathFilter {
    pub fn new(pattern: Option<String>) -> Self {
        Self {
            pattern: pattern.and_then(|value| {
                let normalized = normalize_path_filter(&value);
                if normalized.is_empty() {
                    None
                } else {
                    Some(normalized)
                }
            }),
        }
    }

    pub fn matches(&self, path: &str) -> bool {
        let Some(pattern) = &self.pattern else {
            return true;
        };

        let path = normalize_path_filter(path);
        // 带 / 的过滤条件按路径前缀匹配；普通单词既可匹配目录段，也可匹配文件名片段。
        if pattern.contains('/') {
            path == *pattern || path.starts_with(&format!("{pattern}/"))
        } else {
            path.split('/').any(|segment| segment == pattern) || path.contains(pattern)
        }
    }

    pub fn label(&self) -> String {
        self.pattern.clone().unwrap_or_else(|| "none".to_string())
    }
}

fn normalize_path_filter(value: &str) -> String {
    // 将 Windows 反斜杠转换为 /，使同一条过滤规则在不同系统上表现一致。
    value
        .trim()
        .trim_matches('/')
        .trim_matches('\\')
        .replace('\\', "/")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::PathFilter;

    #[test]
    fn empty_filter_matches_everything() {
        assert!(PathFilter::new(None).matches("rust/ownership.md"));
    }

    #[test]
    fn segment_filter_matches_directory() {
        let filter = PathFilter::new(Some("rust".to_string()));
        assert!(filter.matches("rust/ownership.md"));
        assert!(!filter.matches("algorithm/graph.md"));
    }

    #[test]
    fn path_filter_matches_prefix() {
        let filter = PathFilter::new(Some("algorithm".to_string()));
        assert!(filter.matches("algorithm/dynamic_programming.md"));
    }
}
