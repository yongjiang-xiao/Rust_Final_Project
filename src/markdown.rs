use std::path::Path;

pub fn clean_document_content(path: &Path, content: &str) -> String {
    if is_markdown(path) {
        clean_markdown(content)
    } else {
        content.to_string()
    }
}

pub fn clean_markdown(content: &str) -> String {
    let mut cleaned = String::new();
    let mut in_fence = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if is_fence_marker(trimmed) {
            // 代码块围栏本身不参与搜索，但围栏中的代码文本仍然保留。
            in_fence = !in_fence;
            continue;
        }

        let line_without_block_syntax = if in_fence {
            line.to_string()
        } else {
            // 标题、引用、列表等符号属于 Markdown 语法，不应影响关键词匹配。
            strip_block_markers(trimmed)
        };
        let without_links = replace_markdown_links(&line_without_block_syntax);
        let without_inline = strip_inline_markers(&without_links);

        cleaned.push_str(without_inline.trim_end());
        cleaned.push('\n');
    }

    cleaned
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

fn is_fence_marker(line: &str) -> bool {
    line.starts_with("```") || line.starts_with("~~~")
}

fn strip_block_markers(line: &str) -> String {
    if let Some(stripped) = strip_heading_marker(line) {
        return stripped;
    }

    if let Some(stripped) = line.strip_prefix("> ") {
        return stripped.to_string();
    }

    for marker in ["- ", "* ", "+ "] {
        if let Some(stripped) = line.strip_prefix(marker) {
            return stripped.to_string();
        }
    }

    line.to_string()
}

fn strip_heading_marker(line: &str) -> Option<String> {
    let marker_len = line.chars().take_while(|ch| *ch == '#').count();
    if marker_len == 0 || marker_len > 6 {
        return None;
    }

    let rest = &line[marker_len..];
    rest.strip_prefix(' ')
        .map(|heading_text| heading_text.trim_start().to_string())
}

fn strip_inline_markers(line: &str) -> String {
    line.replace("**", "")
        .replace("__", "")
        .replace(['`', '*', '_'], "")
}

fn replace_markdown_links(line: &str) -> String {
    let mut output = String::new();
    let mut index = 0;

    while index < line.len() {
        // 链接和图片只保留可见文本，URL 地址不进入索引，减少无关命中。
        if line[index..].starts_with("![")
            && let Some((label, next_index)) = parse_link_like(line, index + 2)
        {
            output.push_str(label);
            index = next_index;
            continue;
        }

        if line[index..].starts_with('[')
            && let Some((label, next_index)) = parse_link_like(line, index + 1)
        {
            output.push_str(label);
            index = next_index;
            continue;
        }

        let ch = line[index..]
            .chars()
            .next()
            .expect("index must be a char boundary");
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

fn parse_link_like(line: &str, label_start: usize) -> Option<(&str, usize)> {
    let label_end = label_start + line[label_start..].find(']')?;
    let after_label = label_end + 1;
    if !line[after_label..].starts_with('(') {
        return None;
    }

    let url_start = after_label + 1;
    let url_end = url_start + line[url_start..].find(')')?;
    Some((&line[label_start..label_end], url_end + 1))
}

#[cfg(test)]
mod tests {
    use super::clean_markdown;

    #[test]
    fn removes_heading_markers_and_inline_symbols() {
        let cleaned = clean_markdown("# **Rust Ownership**");
        assert_eq!(cleaned.trim(), "Rust Ownership");
    }

    #[test]
    fn keeps_link_text_and_removes_url() {
        let cleaned = clean_markdown("Read [ownership](https://example.com) docs.");
        assert_eq!(cleaned.trim(), "Read ownership docs.");
    }

    #[test]
    fn removes_code_fence_markers_but_keeps_code_text() {
        let cleaned = clean_markdown("```rust\nlet owner = value;\n```");
        assert_eq!(cleaned.trim(), "let owner = value;");
    }
}
