use crate::trie::{DagEdge, Trie};

pub struct PinyinSegmentor;

impl PinyinSegmentor {
    /// Build a DAG from pinyin input using the trie for efficient prefix matching.
    pub fn build_dag(input: &str, trie: &Trie) -> Vec<DagEdge> {
        let mut edges = Vec::new();
        let n = input.len();

        for i in 0..n {
            let matched_edges = trie.collect_prefixes(input, i);
            let has_match = !matched_edges.is_empty();
            edges.extend(matched_edges);

            // Fallback: unmatched single byte as a raw character (weight 0)
            if !has_match {
                edges.push(DagEdge {
                    from: i,
                    to: i + 1,
                    words: vec![(input[i..i + 1].to_string(), 0)],
                });
            }
        }

        edges
    }
}
