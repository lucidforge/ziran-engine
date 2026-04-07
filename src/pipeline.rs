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

        // 2. 翻译
        let mut all_cands = Vec::new();
        for seg in &ctx.segments {
            let mut cands = self.translator.translate(seg);
            all_cands.append(&mut cands);
        }

        // 3. 简单排序（这里就按 score 降序）
        all_cands.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        ctx.candidates = all_cands;
    }
}
