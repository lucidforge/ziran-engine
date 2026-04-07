use crate::candidate::Candidate;
use crate::segment::Segment;

pub struct Context {
    pub raw_input: String,
    pub segments: Vec<Segment>,
    pub candidates: Vec<Candidate>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            raw_input: String::new(),
            segments: Vec::new(),
            candidates: Vec::new(),
        }
    }
}
