use std::collections::HashMap;
use std::fs;

use crate::candidate::Candidate;
use crate::segment::Segment;

pub struct SimpleTranslator {
    table: HashMap<String, Vec<String>>,
}

impl SimpleTranslator {
    pub fn new() -> Self {
        let table = Self::load_dict("data/dict.txt");
        Self { table }
    }

    fn load_dict(path: &str) -> HashMap<String, Vec<String>> {
        let mut dict: HashMap<String, Vec<String>> = HashMap::new();

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Could not load dictionary '{}': {}", path, e);
                return dict;
            }
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
            if parts.len() == 2 {
                let code = parts[0].to_string();
                let word = parts[1].to_string();
                dict.entry(code).or_default().push(word);
            }
        }

        dict
    }

    pub fn translate(&self, seg: &Segment) -> Vec<Candidate> {
        let mut result = Vec::new();

        if let Some(words) = self.table.get(&seg.code) {
            for (i, word) in words.iter().enumerate() {
                let score = 1.0 - (i as f32 * 0.1);
                result.push(Candidate::new(word.clone(), score));
            }
        }

        if result.is_empty() {
            result.push(Candidate::new(seg.code.clone(), 0.1));
        }

        result
    }
}
