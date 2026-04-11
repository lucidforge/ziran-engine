use crate::candidate::Candidate;
use crate::context::Context;
use crate::dict::LoadedDictionaries;
use crate::segmentor::PinyinSegmentor;
use crate::translator::SimpleTranslator;

pub struct Pipeline {
    segmentor: PinyinSegmentor,
    translator: SimpleTranslator,
}

impl Pipeline {
    pub fn with_dictionaries(dicts: &LoadedDictionaries) -> Self {
        Self {
            segmentor: PinyinSegmentor::with_syllables(
                dicts.syllables.clone(),
                dicts.max_syllable_len,
            ),
            translator: SimpleTranslator::from_loaded_dictionaries(dicts),
        }
    }

    pub fn run(&self, ctx: &mut Context) {
        // 1. DP optimal segmentation with translation
        ctx.segments.clear();
        self.segmentor.segment_dp(ctx, &self.translator);

        // 2. Build candidates from DP result
        let mut all_cands = Vec::new();

        if !ctx.segments.is_empty() {
            // Check if DP produced meaningful translations (not just raw chars)
            let total_weight: u32 = ctx.segments.iter().map(|s| s.weight.unwrap_or(0)).sum();

            if total_weight > 0 {
                // DP found valid translations — build candidate from concatenated segments
                let mut text = String::new();
                for seg in &ctx.segments {
                    if let Some(ref word) = seg.text {
                        text.push_str(word);
                    } else {
                        text.push_str(&seg.code);
                    }
                }
                all_cands.push(Candidate::new(text, total_weight as f32));
            }

            // 3. Fallback: try full phrase lookup if DP result is weak
            if all_cands.is_empty() {
                all_cands = self.translator.translate_phrase(&ctx.raw_input);
            }
        }

        // 4. English fallback
        if all_cands.is_empty() {
            let en_cands = self.translator.translate_en(&ctx.raw_input.to_lowercase());
            if !en_cands.is_empty() {
                all_cands = en_cands;
            }
        }

        // 5. Sort by score descending, truncate to 50
        all_cands.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        if all_cands.len() > 50 {
            all_cands.truncate(50);
        }

        ctx.candidates = all_cands;
    }
}
