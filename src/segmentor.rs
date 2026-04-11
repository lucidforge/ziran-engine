use std::collections::HashSet;

use crate::context::Context;
use crate::segment::Segment;
use crate::translator::SimpleTranslator;

pub struct PinyinSegmentor {
    syllables: HashSet<String>,
    max_len: usize,
}

impl PinyinSegmentor {
    /// Create segmentor with pre-loaded syllables (used by Pipeline with LoadedDictionaries)
    pub fn with_syllables(syllables: HashSet<String>, max_len: usize) -> Self {
        Self { syllables, max_len }
    }

    /// DP optimal segmentation — considers phrase weights to find globally optimal path.
    /// Takes translator reference to access phrase_index and char_index for scoring.
    pub fn segment_dp(&self, ctx: &mut Context, translator: &SimpleTranslator) {
        if ctx.raw_input.is_empty() {
            return;
        }

        let input = ctx.raw_input.as_str();
        let n = input.len();
        let min_score: i64 = i64::MIN / 2; // Avoid overflow when adding

        // dp[i] = best cumulative score to reach position i
        let mut dp = vec![min_score; n + 1];
        dp[0] = 0;

        // backptr[i] = how we reached position i
        #[derive(Clone)]
        struct Backptr {
            prev_pos: usize,
            word: String,
            weight: u32,
        }
        let mut backptr: Vec<Option<Backptr>> = vec![None; n + 1];

        for i in 0..n {
            // Skip if position i is not reachable
            if dp[i] == min_score {
                continue;
            }

            // 1. Try phrase_index matches starting at position i (no length limit)
            for len in 1..=n - i {
                let key = &input[i..i + len];

                if let Some(entries) = translator.phrase_index.get(key) {
                    for entry in entries {
                        let j = i + len;
                        let cand_score = dp[i] + entry.weight as i64;
                        if cand_score > dp[j] {
                            dp[j] = cand_score;
                            backptr[j] = Some(Backptr {
                                prev_pos: i,
                                word: entry.word.clone(),
                                weight: entry.weight,
                            });
                        }
                    }
                }
            }

            // 2. Fallback: try char_index via valid syllable
            if backptr[i].is_none() {
                let max_char_len = (n - i).min(self.max_len);
                for len in (1..=max_char_len).rev() {
                    if i + len > n {
                        continue;
                    }
                    let syl = &input[i..i + len];
                    if !self.syllables.contains(syl) {
                        continue;
                    }

                    if let Some(entries) = translator.char_index.get(syl) {
                        if let Some(entry) = entries.first() {
                            let j = i + len;
                            let cand_score = dp[i] + entry.weight as i64;
                            if cand_score > dp[j] {
                                dp[j] = cand_score;
                                backptr[j] = Some(Backptr {
                                    prev_pos: i,
                                    word: entry.word.clone(),
                                    weight: entry.weight,
                                });
                            }
                        }
                        break; // Found valid syllable, stop trying longer
                    }
                }
            }

            // 3. Ultimate fallback: raw single character (weight 0)
            if backptr[i].is_none() {
                let j = i + 1;
                dp[j] = dp[i]; // No score change
                backptr[j] = Some(Backptr {
                    prev_pos: i,
                    word: input[i..j].to_string(),
                    weight: 0,
                });
            }
        }

        // Backtrack from position n to build segments in reverse
        let mut pos = n;
        while pos > 0 {
            if let Some(bp) = &backptr[pos] {
                let code = input[bp.prev_pos..pos].to_string();
                ctx.segments.push(Segment::with_translation(
                    code,
                    bp.word.clone(),
                    bp.weight,
                ));
                pos = bp.prev_pos;
            } else {
                // Should not happen if DP worked correctly
                break;
            }
        }
        ctx.segments.reverse();
    }
}
