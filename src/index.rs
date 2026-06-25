use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{document::Document, tokenizer::Tokenizer};

// 倒排项（Posting）表示某个词在一篇文档中的索引信息，是倒排索引的基本单元。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Posting {
    pub doc_id: usize,
    pub term_freq: usize,
    pub positions: Vec<usize>,
}

// 倒排索引结构：由“词项”快速定位到包含该词的文档列表。
pub type InvertedIndex = BTreeMap<String, Vec<Posting>>;

// 搜索索引（SearchIndex）是持久化到 .rns/index.json 的核心数据结构。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchIndex {
    pub documents: Vec<Document>,
    pub inverted_index: InvertedIndex,
    pub total_docs: usize,
    pub total_tokens: usize,
}

pub fn build_index<T: Tokenizer>(documents: Vec<Document>, tokenizer: &T) -> SearchIndex {
    // 构建阶段先用哈希表收集“词项 -> 文档 -> 位置列表”，便于累计同一文档内的词频。
    let mut builders: BTreeMap<String, HashMap<usize, Vec<usize>>> = BTreeMap::new();

    for document in &documents {
        for (position, token) in tokenizer
            .tokenize(&document.content)
            .into_iter()
            .enumerate()
        {
            builders
                .entry(token)
                .or_default()
                .entry(document.id)
                .or_default()
                .push(position);
        }
    }

    let inverted_index = builders
        .into_iter()
        .map(|(term, doc_positions)| {
            // 将临时结构压缩成倒排项列表，词频可直接由位置数量得到。
            let mut postings: Vec<Posting> = doc_positions
                .into_iter()
                .map(|(doc_id, positions)| Posting {
                    doc_id,
                    term_freq: positions.len(),
                    positions,
                })
                .collect();
            // 按文档编号排序后，索引文件和测试结果更稳定。
            postings.sort_by_key(|posting| posting.doc_id);
            (term, postings)
        })
        .collect();

    let total_docs = documents.len();
    let total_tokens = documents.iter().map(|document| document.token_count).sum();

    SearchIndex {
        documents,
        inverted_index,
        total_docs,
        total_tokens,
    }
}

impl SearchIndex {
    pub fn term_count(&self) -> usize {
        self.inverted_index.len()
    }

    pub fn average_document_length(&self) -> f64 {
        // 平均文档长度主要供 BM25 使用，空索引时返回 0 避免除零。
        if self.total_docs == 0 {
            0.0
        } else {
            self.total_tokens as f64 / self.total_docs as f64
        }
    }

    pub fn document_by_id(&self, doc_id: usize) -> Option<&Document> {
        self.documents.iter().find(|document| document.id == doc_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        document::Document,
        index::build_index,
        tokenizer::{SimpleTokenizer, Tokenizer},
    };

    #[test]
    fn records_term_frequency_and_positions() {
        let tokenizer = SimpleTokenizer;
        let document = Document {
            id: 0,
            path: "rust/ownership.md".to_string(),
            title: "Rust Ownership".to_string(),
            content: "ownership and ownership".to_string(),
            token_count: tokenizer.tokenize("ownership and ownership").len(),
        };

        let index = build_index(vec![document], &tokenizer);
        let posting = &index.inverted_index["ownership"][0];

        assert_eq!(posting.term_freq, 2);
        assert_eq!(posting.positions, vec![0, 2]);
    }
}
