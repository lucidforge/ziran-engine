use crate::candidate::Candidate;
use crate::context::Context;
use crate::segmentor::PinyinSegmentor;
use crate::translator::SimpleTranslator;

pub struct Pipeline {
    segmentor: PinyinSegmentor,
    translator: SimpleTranslator,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            segmentor: PinyinSegmentor::new(),
            translator: SimpleTranslator::new(),
        }
    }

    pub fn run(&self, ctx: &mut Context) {
        // 1. 分段
        ctx.segments.clear();
        self.segmentor.segment(ctx);

        // 2. 翻译 - 先查完整短语
        let mut all_cands = self.translator.translate_phrase(&ctx.raw_input);

        // 3. 如果短语匹配结果少，再按分段翻译生成组合
        if all_cands.len() < 5 && ctx.segments.len() > 1 {
            let mut char_candidates: Vec<Vec<(String, u32)>> = Vec::new();
            for seg in &ctx.segments {
                let cands = self.translator.get_char_candidates(&seg.code);
                if cands.is_empty() {
                    char_candidates.clear();
                    break;
                }
                char_candidates.push(cands);
            }

            if !char_candidates.is_empty() {
                let mut combinations = Vec::new();
                Self::generate_combinations(
                    &char_candidates,
                    &mut String::new(),
                    0,
                    1.0,
                    &mut combinations,
                );
                for (text, score) in combinations {
                    all_cands.push(Candidate::new(text, score as f32));
                }
            }
        }

        // 4. 单段翻译（短语未匹配时）
        if all_cands.is_empty() && ctx.segments.len() == 1 {
            all_cands = self.translator.translate_segment(&ctx.segments[0]);
        }

        // 5. 英文匹配（当中文未匹配时）
        if all_cands.is_empty() {
            let en_cands = self.translator.translate_en(&ctx.raw_input.to_lowercase());
            if !en_cands.is_empty() {
                all_cands = en_cands;
            }
        }

        // 6. 排序
        all_cands.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // 7. 限制候选数量
        if all_cands.len() > 50 {
            all_cands.truncate(50);
        }

        ctx.candidates = all_cands;
    }

    fn generate_combinations(
        candidates: &[Vec<(String, u32)>],
        current: &mut String,
        index: usize,
        score: f32,
        results: &mut Vec<(String, u32)>,
    ) {
        if index == candidates.len() {
            results.push((current.clone(), score as u32));
            return;
        }

        let start_len = current.len();
        for (i, (text, weight)) in candidates[index].iter().enumerate() {
            if i >= 10 {
                break;
            }
            current.push_str(text);
            Self::generate_combinations(
                candidates,
                current,
                index + 1,
                score * (*weight as f32),
                results,
            );
            current.truncate(start_len);
        }
    }
}
