use crate::candidate::Candidate;
use crate::dict::load_dictionaries;
use crate::pipeline::Pipeline;
use crate::schema::SchemaConfig;
use crate::user_freq::UserFreq;

pub struct Engine {
    pipeline: Pipeline,
    pub user_freq: UserFreq,
    pub candidates: Vec<Candidate>,
}

impl Engine {
    pub fn new() -> Self {
        let schema =
            SchemaConfig::load("data/default.yaml").expect("failed to load schema");
        let dicts = load_dictionaries(&schema);
        let pipeline = Pipeline::with_dictionaries(&dicts);
        let user_freq = UserFreq::load("data/user_freq.tsv");
        Self {
            pipeline,
            user_freq,
            candidates: Vec::new(),
        }
    }

    pub fn run_pipeline(&mut self, input: &str) {
        self.candidates = self.pipeline.run(input, &self.user_freq);
    }

    pub fn record_selection(&mut self, text: &str) {
        self.user_freq.record(text);
    }

    pub fn save_user_freq(&self) {
        self.user_freq.save();
    }
}
