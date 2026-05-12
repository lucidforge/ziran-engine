use std::sync::Arc;

use crate::candidate::Candidate;
use crate::dict::{BilingualIndex, DictCandidate, LoadedDictionaries};
use crate::trie::{DagEdge, Trie};
use crate::user_freq::UserFreq;

const BEAM_WIDTH: usize = 12;
const SEGMENT_PENALTY: f32 = 3.0;
const UNKNOWN_WORD_PENALTY: f32 = -100.0;
const SCORE_EXPONENT: f32 = 0.8;
const MAX_CANDIDATES: usize = 50;

struct PathState {
    score: f32,
    raw_score: f32,
    prev_node: usize,
    prev_state_idx: usize,
    word_text: String,
    segment_count: u32,
}

pub struct Pipeline {
    phrase_trie: Arc<Trie<DictCandidate>>,
    en_trie: Arc<Trie<DictCandidate>>,
    bilingual_index: Arc<BilingualIndex>,
}

impl Pipeline {
    pub fn with_dictionaries(dicts: &LoadedDictionaries) -> Self {
        Self {
            phrase_trie: Arc::clone(&dicts.phrase_trie),
            en_trie: Arc::clone(&dicts.en_trie),
            bilingual_index: Arc::clone(&dicts.bilingual_index),
        }
    }

    /// Run the full pipeline: input pinyin → candidates.
    pub fn run(&self, input: &str, user_freq: &UserFreq) -> Vec<Candidate> {
        if input.is_empty() {
            return Vec::new();
        }

        // 1. Build DAG
        let edges = build_dag(input, &self.phrase_trie);

        // 2. English fallback if no full Chinese coverage
        if let Some(en_cands) = self.try_english_fallback(input, &edges) {
            return en_cands;
        }

        // 3. Beam search
        let dp = beam_search(input.len(), &edges);

        // 4. Backtrack to produce candidates
        let mut cands = backtrack(input.len(), &dp);

        // 5. Deduplicate, apply boosts, annotate, sort
        finalize(&mut cands, user_freq, &self.bilingual_index);
        cands.into_iter().map(|(c, _)| c).collect()
    }

    /// Try English fallback. Returns Some if English should be used instead of Chinese.
    fn try_english_fallback(&self, input: &str, edges: &[DagEdge]) -> Option<Vec<Candidate>> {
        let n = input.len();
        let has_chinese_path = edges.iter().any(|e| e.words.iter().any(|(_, w)| *w > 0));
        let all_covered = (0..n).all(|i| {
            edges
                .iter()
                .any(|e| e.from <= i && i < e.to && e.words.iter().any(|(_, w)| *w > 0))
        });

        if !all_covered || !has_chinese_path {
            let en_cands = english_prefix_match(&self.en_trie, &input.to_lowercase());
            if !en_cands.is_empty() {
                return Some(en_cands);
            }
        }
        None
    }
}

// ── DAG construction (inlined from segmentor) ────────────────────────

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

// ── Beam search ──────────────────────────────────────────────────────

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
                        (*weight as f32).ln()
                    };
                    let new_seg_count = state.segment_count + 1;
                    let raw_score = state.raw_score + word_score;
                    let score = (raw_score - SEGMENT_PENALTY * new_seg_count as f32)
                        / (new_seg_count as f32).powf(SCORE_EXPONENT);

                    next_states.push(PathState {
                        score,
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

// ── Backtrack ────────────────────────────────────────────────────────

fn backtrack(n: usize, dp: &[Vec<PathState>]) -> Vec<(Candidate, u32)> {
    let mut cands: Vec<(Candidate, u32)> = Vec::new();

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
            if !text.is_empty() && !has_ascii_letter {
                cands.push((Candidate::new(text, state.score), state.segment_count));
            }
        }
    }

    cands
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
