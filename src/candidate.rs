pub struct Candidate {
    pub text: String,
    pub score: f32,
    pub annotation: Option<String>,
}

impl Candidate {
    pub fn new(text: String, score: f32) -> Self {
        Self {
            text,
            score,
            annotation: None,
        }
    }
}
