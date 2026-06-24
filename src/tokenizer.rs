use serde::{Deserialize, Serialize};

pub trait Tokenizer {
    fn tokenize(&self, text: &str) -> Vec<String>;
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct SimpleTokenizer;

impl Tokenizer for SimpleTokenizer {
    fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut word = String::new();
        let mut cjk_run = String::new();

        // 英文和数字按连续单词切分；连续中文先收集成一段，再使用 bigram 切分。
        for ch in text.chars() {
            if ch.is_ascii_alphanumeric() {
                flush_cjk(&mut cjk_run, &mut tokens);
                word.push(ch.to_ascii_lowercase());
            } else if is_cjk(ch) {
                flush_word(&mut word, &mut tokens);
                cjk_run.push(ch);
            } else {
                flush_word(&mut word, &mut tokens);
                flush_cjk(&mut cjk_run, &mut tokens);
            }
        }

        flush_word(&mut word, &mut tokens);
        flush_cjk(&mut cjk_run, &mut tokens);
        tokens
    }
}

fn flush_word(word: &mut String, tokens: &mut Vec<String>) {
    if !word.is_empty() {
        tokens.push(std::mem::take(word));
    }
}

fn flush_cjk(run: &mut String, tokens: &mut Vec<String>) {
    if run.is_empty() {
        return;
    }

    let chars: Vec<char> = run.chars().collect();
    match chars.len() {
        0 => {}
        1 => tokens.push(chars[0].to_string()),
        _ => {
            // 中文没有天然空格分隔，使用相邻两个字组成 token，例如“所有权” -> “所有”“有权”。
            for pair in chars.windows(2) {
                tokens.push(pair.iter().collect());
            }
        }
    }
    run.clear();
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
    use super::{SimpleTokenizer, Tokenizer};

    #[test]
    fn tokenizes_english_words() {
        let tokenizer = SimpleTokenizer;
        assert_eq!(
            tokenizer.tokenize("Rust Ownership!"),
            vec!["rust", "ownership"]
        );
    }

    #[test]
    fn tokenizes_chinese_bigram() {
        let tokenizer = SimpleTokenizer;
        assert_eq!(tokenizer.tokenize("所有权"), vec!["所有", "有权"]);
    }
}
