use std::collections::HashMap;

use crate::dict::DictCandidate;

pub struct DagEdge {
    pub from: usize,
    pub to: usize,
    pub words: Vec<(String, u32)>,
}

struct TrieNode {
    children: HashMap<u8, TrieNode>,
    entries: Vec<DictCandidate>,
}

pub struct Trie {
    root: TrieNode,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            entries: Vec::new(),
        }
    }
}

impl Trie {
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(),
        }
    }

    pub fn insert(&mut self, key: &str, entry: DictCandidate) {
        let mut node = &mut self.root;
        for byte in key.bytes() {
            node = node.children.entry(byte).or_insert_with(TrieNode::new);
        }
        node.entries.push(entry);
    }

    /// Build DAG edges starting at position `start` in `text`.
    /// Walks the trie from root, collecting edges at every terminal node.
    pub fn collect_prefixes(&self, text: &str, start: usize) -> Vec<DagEdge> {
        let bytes = text.as_bytes();
        let n = bytes.len();
        let mut edges = Vec::new();
        let mut node = &self.root;

        for i in start..n {
            match node.children.get(&bytes[i]) {
                Some(child) => {
                    node = child;
                    if !node.entries.is_empty() {
                        let words: Vec<(String, u32)> = node
                            .entries
                            .iter()
                            .map(|e| (e.text.clone(), e.weight))
                            .collect();
                        edges.push(DagEdge {
                            from: start,
                            to: i + 1,
                            words,
                        });
                    }
                }
                None => break,
            }
        }

        edges
    }

    /// Collect all entries whose key starts with the given prefix.
    /// Returns (exact_matches, prefix_matches).
    pub fn prefix_search(&self, prefix: &str) -> (Vec<&DictCandidate>, Vec<&DictCandidate>) {
        let mut node = &self.root;
        for byte in prefix.bytes() {
            match node.children.get(&byte) {
                Some(child) => node = child,
                None => return (Vec::new(), Vec::new()),
            }
        }
        // node is now at the end of the prefix
        let exact: Vec<&DictCandidate> = node.entries.iter().collect();
        let mut prefix_matches = Vec::new();
        Self::collect_all(node, &mut prefix_matches);
        (exact, prefix_matches)
    }

    fn collect_all<'a>(node: &'a TrieNode, result: &mut Vec<&'a DictCandidate>) {
        for entry in &node.entries {
            result.push(entry);
        }
        for child in node.children.values() {
            Self::collect_all(child, result);
        }
    }
}
