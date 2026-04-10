use std::collections::HashSet;
use std::fs;

use crate::context::Context;
use crate::segment::Segment;
use crate::translator::SimpleTranslator;

pub struct PinyinSegmentor {
    syllables: HashSet<String>,
    max_len: usize,
}

impl PinyinSegmentor {
    pub fn new() -> Self {
        let mut syllables = HashSet::new();
        let mut max_len: usize = 0;

        let cn_files = [
            "data/base.dict.yaml",
            "data/ext.dict.yaml",
            "data/others.dict.yaml",
        ];

        for file in &cn_files {
            let (s, m) = Self::load_syllables(file);
            syllables.extend(s);
            if m > max_len {
                max_len = m;
            }
        }

        Self { syllables, max_len }
    }

    fn load_syllables(path: &str) -> (HashSet<String>, usize) {
        let mut syllables: HashSet<String> = HashSet::new();
        let mut max_len: usize = 0;

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return (syllables, max_len),
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 3 {
                continue;
            }

            let pinyin_str = parts[1];
            let pinyin_parts: Vec<&str> = pinyin_str.split_whitespace().collect();

            for syl in pinyin_parts {
                if syl.len() > max_len {
                    max_len = syl.len();
                }
                syllables.insert(syl.to_string());
            }
        }

        (syllables, max_len)
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

            // 1. Try phrase_index matches starting at position i
            let max_check = (n - i).min(self.max_len);
            for len in 1..=max_check {
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
                for len in (1..=max_check).rev() {
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
