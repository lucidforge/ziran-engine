use crate::context::Context;
use crate::dict::load_dictionaries;
use crate::schema::SchemaConfig;
use crate::pipeline::Pipeline;

pub struct Engine {
    pub context: Context,
    pipeline: Pipeline,
}

impl Engine {
    pub fn new() -> Self {
        let schema = SchemaConfig::load("data/default.yaml")
            .expect("failed to load schema");
        let dicts = load_dictionaries(&schema);
        let pipeline = Pipeline::with_dictionaries(&dicts);
        let context = Context::new();
        Self { context, pipeline }
    }

    pub fn run_pipeline(&mut self) {
        self.pipeline.run(&mut self.context);
    }
}
