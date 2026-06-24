use std::collections::BTreeSet;

pub fn make_snippet(
    content: &str,
    query: &str,
    matched_terms: &[String],
    max_chars: usize,
) -> String {
    let display_terms = display_terms(query, matched_terms);
    // 优先围绕最早出现的命中词生成片段，让用户快速看到为什么这篇文档被返回。
    let hit_position = display_terms
        .iter()
        .filter_map(|term| find_case_insensitive(content, term))
        .min()
        .unwrap_or(0);

    let snippet = window_around(content, hit_position, max_chars);
    highlight_terms(&snippet, &display_terms)
}

pub fn display_terms(query: &str, matched_terms: &[String]) -> Vec<String> {
    let matched: BTreeSet<String> = matched_terms.iter().cloned().collect();
    let mut terms = BTreeSet::new();
    let mut word = String::new();
    let mut cjk_run = String::new();

    for ch in query.chars() {
        // 英文按单词展示；中文尽量保留用户输入的连续文本，避免片段里只高亮双字切分结果。
        if ch.is_ascii_alphanumeric() {
            flush_cjk_display(&mut cjk_run, &mut terms, &matched);
            word.push(ch.to_ascii_lowercase());
        } else if is_cjk(ch) {
            flush_word_display(&mut word, &mut terms, &matched);
            cjk_run.push(ch);
        } else {
            flush_word_display(&mut word, &mut terms, &matched);
            flush_cjk_display(&mut cjk_run, &mut terms, &matched);
        }
    }

    flush_word_display(&mut word, &mut terms, &matched);
    flush_cjk_display(&mut cjk_run, &mut terms, &matched);

    if terms.is_empty() {
        terms.extend(matched.iter().cloned());
    }

    let mut sorted: Vec<String> = terms.into_iter().collect();
    sorted.sort_by_key(|term| std::cmp::Reverse(term.chars().count()));
    sorted
}

fn flush_word_display(word: &mut String, terms: &mut BTreeSet<String>, matched: &BTreeSet<String>) {
    if !word.is_empty() {
        if matched.contains(word) {
            terms.insert(std::mem::take(word));
        } else {
            word.clear();
        }
    }
}

fn flush_cjk_display(run: &mut String, terms: &mut BTreeSet<String>, matched: &BTreeSet<String>) {
    if run.is_empty() {
        return;
    }

    if cjk_run_matches(run, matched) {
        terms.insert(std::mem::take(run));
    } else {
        run.clear();
    }
}

fn cjk_run_matches(run: &str, matched: &BTreeSet<String>) -> bool {
    let chars: Vec<char> = run.chars().collect();
    if chars.len() == 1 {
        return matched.contains(run);
    }

    chars
        .windows(2)
        .map(|pair| pair.iter().collect::<String>())
        .any(|term| matched.contains(&term))
}

fn window_around(content: &str, byte_position: usize, max_chars: usize) -> String {
    let chars: Vec<char> = content.chars().collect();
    if chars.len() <= max_chars {
        return content.to_string();
    }

    // 用字符数量截断而不是字节数量，避免中文内容被截断到非法 UTF-8 边界。
    let hit_char = content[..byte_position.min(content.len())].chars().count();
    let start = hit_char.saturating_sub(max_chars / 3);
    let end = (start + max_chars).min(chars.len());

    let mut snippet: String = chars[start..end].iter().collect();
    if start > 0 {
        snippet.insert_str(0, "...");
    }
    if end < chars.len() {
        snippet.push_str("...");
    }
    snippet
}

fn highlight_terms(text: &str, terms: &[String]) -> String {
    // 输出中使用 Markdown 加粗标记突出命中词，命令行中可以直接展示。
    terms
        .iter()
        .filter(|term| !term.is_empty())
        .fold(text.to_string(), |current, term| {
            highlight_one_term(&current, term)
        })
}

fn highlight_one_term(text: &str, term: &str) -> String {
    let lower_text = text.to_lowercase();
    let lower_term = term.to_lowercase();
    let mut result = String::new();
    let mut search_from = 0;

    while let Some(relative_start) = lower_text[search_from..].find(&lower_term) {
        let start = search_from + relative_start;
        let end = start + lower_term.len();
        result.push_str(&text[search_from..start]);
        result.push_str("**");
        result.push_str(&text[start..end]);
        result.push_str("**");
        search_from = end;
    }

    result.push_str(&text[search_from..]);
    result
}

fn find_case_insensitive(text: &str, term: &str) -> Option<usize> {
    text.to_lowercase().find(&term.to_lowercase())
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x4E00..=0x9FFF
            | 0x3400..=0x4DBF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B820..=0x2CEAF
            | 0xF900..=0xFAFF
    )
}

#[cfg(test)]
mod tests {
    use super::make_snippet;

    #[test]
    fn highlights_english_keyword() {
        let snippet = make_snippet(
            "Rust ownership means each value has one owner.",
            "ownership",
            &["ownership".to_string()],
            80,
        );
        assert!(snippet.contains("**ownership**"));
    }
}
