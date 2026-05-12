use std::sync::Arc;

use crate::candidate::Candidate;
use crate::context::Context;
use crate::dict::{BilingualIndex, LoadedDictionaries};
use crate::segment::Segment;
use crate::segmentor::PinyinSegmentor;
use crate::trie::Trie;
use crate::user_freq::UserFreq;

const BEAM_WIDTH: usize = 12;

struct PathState {
    score: f32,
    raw_score: f32,
    prev_node: usize,
    prev_state_idx: usize,
    word_text: String,
    segment_count: u32,
}

pub struct Pipeline {
    phrase_trie: Arc<Trie>,
    en_trie: Arc<Trie>,
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

    fn english_prefix_match(&self, input: &str) -> Vec<Candidate> {
        let (exact, prefix) = self.en_trie.prefix_search(input);

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

        result.truncate(50);
        result
    }

    pub fn run(&self, ctx: &mut Context, user_freq: &UserFreq) {
        let input = ctx.raw_input.as_str();
        let n = input.len();

        if n == 0 {
            ctx.candidates.clear();
            return;
        }

        // 0. Build DAG and check for Chinese paths
        let edges = PinyinSegmentor::build_dag(input, &self.phrase_trie);
        let has_chinese_path = edges.iter().any(|e| e.words.iter().any(|(_, w)| *w > 0));

        let all_covered = (0..n).all(|i| {
            edges
                .iter()
                .any(|e| e.from <= i && i < e.to && e.words.iter().any(|(_, w)| *w > 0))
        });

        // English fallback
        if !all_covered {
            let en_cands = self.english_prefix_match(&input.to_lowercase());
            if !en_cands.is_empty() {
                ctx.candidates = en_cands;
                ctx.segments.clear();
                return;
            }
        }

        if !has_chinese_path {
            let en_cands = self.english_prefix_match(&input.to_lowercase());
            if !en_cands.is_empty() {
                ctx.candidates = en_cands;
                ctx.segments.clear();
                return;
            }
        }

        // 1. Beam Search with log-weight scoring, penalized and normalized by segment count.
        // score = (sum(log(weight)) - SEGMENT_PENALTY * seg_count) / seg_count^0.8
        // The per-segment penalty counteracts the bias that sum(log) has toward more segments.
        const SEGMENT_PENALTY: f32 = 3.0;
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
                            -100.0
                        } else {
                            (*weight as f32).ln()
                        };
                        let new_seg_count = state.segment_count + 1;
                        let raw_score = state.raw_score + word_score;
                        let score = (raw_score - SEGMENT_PENALTY * new_seg_count as f32)
                            / (new_seg_count as f32).powf(0.8);

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

        // 2. Backtrack to produce candidates with segment_count
        let mut all_cands: Vec<(Candidate, u32)> = Vec::new();

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
                    segments.push(Segment::new(st.word_text.clone()));
                    curr_node = st.prev_node;
                    curr_idx = st.prev_state_idx;
                }

                segments.reverse();

                let text: String = segments.iter().map(|s| s.text.as_str()).collect();
                let has_ascii_letter = text.chars().any(|c| c.is_ascii_alphabetic());
                if !text.is_empty() && !has_ascii_letter {
                    let seg_count = state.segment_count;
                    all_cands.push((Candidate::new(text, state.score), seg_count));
                }
            }
        }

        // 3. Deduplicate by text
        all_cands.sort_by(|a, b| a.0.text.cmp(&b.0.text).then_with(|| a.1.cmp(&b.1)));
        all_cands.dedup_by(|a, b| a.0.text == b.0.text);

        // 4. Apply user frequency boost and bilingual annotations
        for cand in &mut all_cands {
            let boost = user_freq.get_boost(&cand.0.text);
            cand.0.score += boost;

            // Add bilingual annotation if available
            if let Some(translations) = self.bilingual_index.get(&cand.0.text) {
                if let Some((best_en, _)) = translations.first() {
                    cand.0.annotation = Some(best_en.clone());
                }
            }
        }
        all_cands.sort_by(|a, b| {
            b.0.score
                .partial_cmp(&a.0.score)
                .unwrap()
                .then_with(|| a.1.cmp(&b.1))
        });
        if all_cands.len() > 50 {
            all_cands.truncate(50);
        }

        ctx.candidates = all_cands.into_iter().map(|(c, _)| c).collect();
        ctx.segments.clear();
    }
}
