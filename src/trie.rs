use std::collections::HashMap;

#[derive(Clone)]
pub struct DagEdge {
    pub from: usize,
    pub to: usize,
    pub words: Vec<(String, u32)>,
}

struct TrieNode<V> {
    children: HashMap<u8, TrieNode<V>>,
    entries: Vec<V>,
}

pub struct Trie<V> {
    root: TrieNode<V>,
}

impl<V> TrieNode<V> {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            entries: Vec::new(),
        }
    }
}

impl<V> Trie<V> {
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(),
        }
    }

    pub fn insert(&mut self, key: &str, entry: V) {
        let mut node = &mut self.root;
        for byte in key.bytes() {
            node = node.children.entry(byte).or_insert_with(TrieNode::new);
        }
        node.entries.push(entry);
    }

    /// Walk the trie from `text[start]`, calling `to_edge` at each terminal node.
    /// Returns collected edges.
    pub fn collect_prefixes<F>(&self, text: &str, start: usize, to_edge: F) -> Vec<DagEdge>
    where
        F: Fn(usize, usize, &[V]) -> Option<DagEdge>,
    {
        let bytes = text.as_bytes();
        let n = bytes.len();
        let mut edges = Vec::new();
        let mut node = &self.root;

        for i in start..n {
            match node.children.get(&bytes[i]) {
                Some(child) => {
                    node = child;
                    if !node.entries.is_empty() {
                        if let Some(edge) = to_edge(start, i + 1, &node.entries) {
                            edges.push(edge);
                        }
                    }
                }
                None => break,
            }
        }

        edges
    }

    /// Collect all entries whose key starts with the given prefix.
    /// Returns (exact_matches, prefix_matches).
    pub fn prefix_search(&self, prefix: &str) -> (Vec<&V>, Vec<&V>) {
        let mut node = &self.root;
        for byte in prefix.bytes() {
            match node.children.get(&byte) {
                Some(child) => node = child,
                None => return (Vec::new(), Vec::new()),
            }
        }
        let exact: Vec<&V> = node.entries.iter().collect();
        let mut prefix_matches = Vec::new();
        Self::collect_all(node, &mut prefix_matches);
        (exact, prefix_matches)
    }

    fn collect_all<'a>(node: &'a TrieNode<V>, result: &mut Vec<&'a V>) {
        for child in node.children.values() {
            for entry in &child.entries {
                result.push(entry);
            }
            Self::collect_all(child, result);
        }
    }
}
