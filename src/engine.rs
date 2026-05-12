use crate::context::Context;
use crate::dict::load_dictionaries;
use crate::pipeline::Pipeline;
use crate::schema::SchemaConfig;
use crate::user_freq::UserFreq;

pub struct Engine {
    pub context: Context,
    pipeline: Pipeline,
    pub user_freq: UserFreq,
}

impl Engine {
    pub fn new() -> Self {
        let schema =
            SchemaConfig::load("data/default.yaml").expect("failed to load schema");
        let dicts = load_dictionaries(&schema);
        let pipeline = Pipeline::with_dictionaries(&dicts);
        let context = Context::new();
        let user_freq = UserFreq::load("data/user_freq.tsv");
        Self {
            context,
            pipeline,
            user_freq,
        }
    }

    pub fn run_pipeline(&mut self) {
        self.pipeline.run(&mut self.context, &self.user_freq);
    }

    pub fn record_selection(&mut self, text: &str) {
        self.user_freq.record(text);
    }

    pub fn save_user_freq(&self) {
        self.user_freq.save();
    }
}
