use std::sync::Arc;

use crate::candidate::Candidate;
use crate::dict::{BilingualIndex, DictCandidate, LoadedDictionaries};
use crate::trie::{DagEdge, Trie};
use crate::user_freq::UserFreq;

const BEAM_WIDTH: usize = 24;
const QUALITY_LEN_BONUS: f64 = 50.0;
const UNKNOWN_WORD_PENALTY: f64 = -1e8;
const MAX_CANDIDATES: usize = 50;

struct PathState {
    score: f64,
    raw_score: f64,
    prev_node: usize,
    prev_state_idx: usize,
    word_text: String,
    segment_count: u32,
}

pub struct Pipeline {
    phrase_trie: Arc<Trie<DictCandidate>>,
    char_trie: Arc<Trie<DictCandidate>>,
    en_trie: Arc<Trie<DictCandidate>>,
    bilingual_index: Arc<BilingualIndex>,
}

impl Pipeline {
    pub fn with_dictionaries(dicts: &LoadedDictionaries) -> Self {
        Self {
            phrase_trie: Arc::clone(&dicts.phrase_trie),
            char_trie: Arc::clone(&dicts.char_trie),
            en_trie: Arc::clone(&dicts.en_trie),
            bilingual_index: Arc::clone(&dicts.bilingual_index),
        }
    }

    /// Run the full pipeline: input pinyin → candidates.
    /// Strategy: phrase trie first, merged trie as fallback for partial coverage.
    pub fn run(&self, input: &str, user_freq: &UserFreq) -> Vec<Candidate> {
        let input: String = input.chars().filter(|c| !c.is_whitespace()).collect();
        if input.is_empty() {
            return Vec::new();
        }

        // Try phrase trie first
        let phrase_edges = build_dag(&input, &self.phrase_trie);
        let full_coverage = has_full_coverage(input.len(), &phrase_edges);

        let mut cands = if full_coverage {
            // Phrase trie covers everything — use it alone
            let dp = beam_search(input.len(), &phrase_edges);
            backtrack(input.len(), &dp)
        } else {
            // Partial coverage — merge both tries so single chars fill gaps
            let merged = merge_edges(&phrase_edges, &build_dag(&input, &self.char_trie));
            let dp = beam_search(input.len(), &merged);
            backtrack(input.len(), &dp)
        };

        // English fallback if no Chinese candidates
        if cands.is_empty() {
            let en_cands = english_prefix_match(&self.en_trie, &input.to_lowercase());
            if !en_cands.is_empty() {
                return en_cands;
            }
        }

        // Deduplicate, apply boosts, annotate, sort
        finalize(&mut cands, user_freq, &self.bilingual_index);
        cands.into_iter().map(|(c, _)| c).collect()
    }
}

/// Check if the DAG has real (non-UNKNOWN) edges covering every position.
fn has_full_coverage(n: usize, edges: &[DagEdge]) -> bool {
    (0..n).all(|i| {
        edges.iter().any(|e| {
            e.from <= i
                && i < e.to
                && e.words.iter().any(|(_, w)| *w > 0)
        })
    })
}

/// Merge two edge lists, combining words for edges with the same (from, to).
fn merge_edges(a: &[DagEdge], b: &[DagEdge]) -> Vec<DagEdge> {
    let mut merged: Vec<DagEdge> = a.to_vec();
    for edge_b in b {
        if let Some(existing) = merged.iter_mut().find(|e| e.from == edge_b.from && e.to == edge_b.to) {
            existing.words.extend(edge_b.words.iter().cloned());
        } else {
            merged.push(edge_b.clone());
        }
    }
    merged
}

// ── DAG construction ──────────────────────────────────────────────────

fn build_dag(input: &str, trie: &Trie<DictCandidate>) -> Vec<DagEdge> {
    let mut edges = Vec::new();
    let n = input.len();

    for i in 0..n {
        let matched = trie.collect_prefixes(input, i, |from, to, entries| {
            let words: Vec<(String, u32)> = entries.iter().map(|e| (e.text.clone(), e.weight)).collect();
            Some(DagEdge { from, to, words })
        });
        let has_match = !matched.is_empty();
        edges.extend(matched);

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

// ── Beam search (log-space, like librime's compiled weights) ──────────

fn beam_search(n: usize, edges: &[DagEdge]) -> Vec<Vec<PathState>> {
    let start = PathState {
        score: 0.0,
        raw_score: 0.0,
        prev_node: 0,
        prev_state_idx: 0,
        word_text: String::new(),
        segment_count: 0,
    };
    let mut dp: Vec<Vec<PathState>> = (0..=n).map(|_| Vec::new()).collect();
    dp[0].push(start);

    for i in 0..=n {
        if dp[i].is_empty() {
            continue;
        }

        let outgoing: Vec<_> = edges.iter().filter(|e| e.from == i).collect();
        if outgoing.is_empty() {
            continue;
        }

        for edge in outgoing {
            let mut next_states: Vec<PathState> = Vec::new();

            for (state_idx, state) in dp[i].iter().enumerate() {
                for (text, weight) in &edge.words {
                    let word_score = if *weight == 0 {
                        UNKNOWN_WORD_PENALTY
                    } else {
                        (*weight as f64).ln()
                    };
                    let new_seg_count = state.segment_count + 1;
                    let raw_score = state.raw_score + word_score;

                    next_states.push(PathState {
                        score: raw_score,
                        raw_score,
                        prev_node: i,
                        prev_state_idx: state_idx,
                        word_text: text.clone(),
                        segment_count: new_seg_count,
                    });
                }
            }

            next_states.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            next_states.truncate(BEAM_WIDTH);

            dp[edge.to].extend(next_states);
            dp[edge.to].sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            dp[edge.to].truncate(BEAM_WIDTH);
        }
    }

    dp
}

// ── Backtrack + librime-style quality ─────────────────────────────────

fn safe_exp(x: f64) -> f64 {
    if x > 500.0 {
        f64::MAX
    } else if x < -500.0 {
        0.0
    } else {
        x.exp()
    }
}

fn backtrack(n: usize, dp: &[Vec<PathState>]) -> Vec<(Candidate, u32)> {
    let mut phrases: Vec<(Candidate, u32)> = Vec::new();
    let mut single_chars: Vec<(Candidate, u32)> = Vec::new();
    let input_len = n.max(1) as f64;

    if let Some(final_states) = dp.last() {
        for state in final_states.iter() {
            let mut segments = Vec::new();
            let mut curr_node = n;
            let mut curr_idx = 0;

            if let Some(pos) = dp[n].iter().position(|s| std::ptr::eq(s, state)) {
                curr_idx = pos;
            }

            loop {
                let st = &dp[curr_node][curr_idx];
                if st.word_text.is_empty() {
                    break;
                }
                segments.push(st.word_text.clone());
                curr_node = st.prev_node;
                curr_idx = st.prev_state_idx;
            }

            segments.reverse();

            let text: String = segments.concat();
            let has_ascii_letter = text.chars().any(|c| c.is_ascii_alphabetic());
            if text.is_empty() || has_ascii_letter {
                continue;
            }

            // librime quality: exp(avg_log_weight) + quality_len_bonus * (matched_chars / input_len)
            let matched_chars = text.chars().count().max(1) as f64;
            let avg_log_weight = state.raw_score / state.segment_count as f64;
            let quality = safe_exp(avg_log_weight) + QUALITY_LEN_BONUS * (matched_chars / input_len);

            let cand = (Candidate::new(text, quality as f32), state.segment_count);
            let is_all_single = segments.iter().all(|s| s.chars().count() == 1);
            if is_all_single {
                single_chars.push(cand);
            } else {
                phrases.push(cand);
            }
        }
    }

    // Phrases always rank above single chars
    phrases.extend(single_chars);
    phrases
}

// ── English prefix match ─────────────────────────────────────────────

fn english_prefix_match(trie: &Trie<DictCandidate>, input: &str) -> Vec<Candidate> {
    let (exact, prefix) = trie.prefix_search(input);

    let mut result: Vec<Candidate> = Vec::new();

    for e in &exact {
        result.push(Candidate::new(e.text.clone(), e.weight as f32));
    }
    result.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    let mut prefix_cands: Vec<Candidate> = Vec::new();
    for e in &prefix {
        prefix_cands.push(Candidate::new(e.text.clone(), e.weight as f32));
    }
    prefix_cands.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    result.extend(prefix_cands);

    result.truncate(MAX_CANDIDATES);
    result
}

// ── Finalize: dedup, boost, annotate, sort ───────────────────────────

fn finalize(
    cands: &mut Vec<(Candidate, u32)>,
    user_freq: &UserFreq,
    bilingual_index: &BilingualIndex,
) {
    // Deduplicate by text
    cands.sort_by(|a, b| a.0.text.cmp(&b.0.text).then_with(|| a.1.cmp(&b.1)));
    cands.dedup_by(|a, b| a.0.text == b.0.text);

    // Apply user frequency boost and bilingual annotations
    for cand in cands.iter_mut() {
        let boost = user_freq.get_boost(&cand.0.text);
        cand.0.score += boost;

        if let Some(translations) = bilingual_index.get(&cand.0.text) {
            if let Some((best_en, _)) = translations.first() {
                cand.0.annotation = Some(best_en.clone());
            }
        }
    }

    // Sort: score descending, segment_count as tie-breaker
    cands.sort_by(|a, b| {
        b.0.score
            .partial_cmp(&a.0.score)
            .unwrap()
            .then_with(|| a.1.cmp(&b.1))
    });
    cands.truncate(MAX_CANDIDATES);
}
