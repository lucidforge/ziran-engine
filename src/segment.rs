pub struct Segment {
    pub code: String,           // pinyin syllable(s), e.g., "ni", "xian"
    pub text: Option<String>,   // Chinese translation, e.g., Some("你")
    pub weight: Option<u32>,    // weight of this word
}

impl Segment {
    pub fn with_translation(code: String, text: String, weight: u32) -> Self {
        Self { code, text: Some(text), weight: Some(weight) }
    }
}
