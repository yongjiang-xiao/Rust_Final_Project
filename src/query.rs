use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, Result},
    tokenizer::Tokenizer,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryMode {
    // Any 对应 OR 语义：命中任意一个正向关键词即可进入候选结果。
    Any,
    // All 对应 AND 语义：必须命中全部正向关键词。
    All,
}

impl QueryMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Any => "OR",
            Self::All => "AND",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParsedQuery {
    pub original: String,
    // 查询模式和关键词分开存储，后续搜索流程不需要再解析原始字符串。
    pub mode: QueryMode,
    pub positive_terms: Vec<String>,
    pub excluded_terms: Vec<String>,
}

impl ParsedQuery {
    pub fn requires_all_terms(&self) -> bool {
        self.mode == QueryMode::All
    }
}

pub fn parse_query<T: Tokenizer>(query: &str, tokenizer: &T) -> Result<ParsedQuery> {
    if query.trim().is_empty() {
        return Err(AppError::EmptyQuery);
    }

    // BTreeSet 用来去重并保持固定顺序，便于测试和结果展示。
    let mut positives = BTreeSet::new();
    let mut excluded = BTreeSet::new();
    let mut saw_and = false;
    let mut saw_or = false;

    for part in query.split_whitespace() {
        // AND/OR 作为查询模式标记，不作为普通关键词参与分词。
        if part.eq_ignore_ascii_case("AND") {
            saw_and = true;
            continue;
        }
        if part.eq_ignore_ascii_case("OR") {
            saw_or = true;
            continue;
        }

        if let Some(excluded_part) = part.strip_prefix('-') {
            // 以 '-' 开头的词进入排除集合，例如 rust -trait。
            add_tokens(&mut excluded, excluded_part, tokenizer);
        } else {
            add_tokens(&mut positives, part, tokenizer);
        }
    }

    if positives.is_empty() {
        return Err(AppError::EmptyQuery);
    }

    let mode = if saw_and && !saw_or {
        QueryMode::All
    } else {
        // 默认使用 OR；如果同时出现 AND 和 OR，也退回 OR，避免规则冲突。
        QueryMode::Any
    };

    Ok(ParsedQuery {
        original: query.to_string(),
        mode,
        positive_terms: positives.into_iter().collect(),
        excluded_terms: excluded.into_iter().collect(),
    })
}

fn add_tokens<T: Tokenizer>(target: &mut BTreeSet<String>, text: &str, tokenizer: &T) {
    // 查询词也走同一套 tokenizer，保证“建索引时怎么切，查询时也怎么切”。
    target.extend(tokenizer.tokenize(text));
}

#[cfg(test)]
mod tests {
    use super::{QueryMode, parse_query};
    use crate::tokenizer::SimpleTokenizer;

    #[test]
    fn parses_default_or_terms() {
        let query = parse_query("rust ownership", &SimpleTokenizer).expect("parse");

        assert_eq!(query.mode, QueryMode::Any);
        assert_eq!(query.positive_terms, vec!["ownership", "rust"]);
        assert!(query.excluded_terms.is_empty());
    }

    #[test]
    fn parses_and_mode() {
        let query = parse_query("rust AND ownership", &SimpleTokenizer).expect("parse");

        assert_eq!(query.mode, QueryMode::All);
        assert_eq!(query.positive_terms, vec!["ownership", "rust"]);
    }

    #[test]
    fn parses_or_mode() {
        let query = parse_query("ownership OR borrowing", &SimpleTokenizer).expect("parse");

        assert_eq!(query.mode, QueryMode::Any);
        assert_eq!(query.positive_terms, vec!["borrowing", "ownership"]);
    }

    #[test]
    fn parses_excluded_terms() {
        let query = parse_query("rust ownership -database", &SimpleTokenizer).expect("parse");

        assert_eq!(query.positive_terms, vec!["ownership", "rust"]);
        assert_eq!(query.excluded_terms, vec!["database"]);
    }

    #[test]
    fn empty_positive_terms_are_rejected() {
        assert!(parse_query("-database", &SimpleTokenizer).is_err());
    }
}
