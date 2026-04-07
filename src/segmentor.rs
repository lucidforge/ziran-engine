use crate::context::Context;
use crate::segment::Segment;

pub struct PinyinSegmentor;

impl PinyinSegmentor {
    pub fn new() -> Self {
        Self
    }

    pub fn segment(&self, ctx: &mut Context) {
        if ctx.raw_input.is_empty() {
            return;
        }
        ctx.segments.push(Segment::new(ctx.raw_input.clone()));
    }
}
