use crate::context::Context;
use crate::pipeline::Pipeline;

pub struct Engine {
    pub context: Context,
    pipeline: Pipeline,
}

impl Engine {
    pub fn new() -> Self {
        let pipeline = Pipeline::new();
        let context = Context::new();
        Self { context, pipeline }
    }

    pub fn run_pipeline(&mut self) {
        self.pipeline.run(&mut self.context);
    }
}
