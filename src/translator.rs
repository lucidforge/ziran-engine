use std::collections::HashMap;
use std::fs;

use crate::candidate::Candidate;
use crate::segment::Segment;

struct DictEntry {
    word: String,
    weight: u32,
}

pub struct SimpleTranslator {
    phrase_index: HashMap<String, Vec<DictEntry>>,
    char_index: HashMap<String, Vec<DictEntry>>,
    en_index: HashMap<String, Vec<DictEntry>>,
}

impl SimpleTranslator {
    pub fn new() -> Self {
        let mut phrase_index = HashMap::new();
        let mut char_index = HashMap::new();

        let cn_files = [
            "data/dict.txt",
            "data/ext.dict.yaml",
            "data/others.dict.yaml",
        ];

        for file in &cn_files {
            let (p, c) = Self::load_cn_dict(file);
            Self::merge_indices(&mut phrase_index, &mut char_index, p, c);
        }

        let en_files = ["data/en.dict.yaml", "data/en_ext.dict.yaml"];

        let mut en_index = HashMap::new();
        for file in &en_files {
            Self::load_en_dict(file, &mut en_index);
        }

        Self {
            phrase_index,
            char_index,
            en_index,
        }
    }

    fn load_cn_dict(
        path: &str,
    ) -> (
        HashMap<String, Vec<DictEntry>>,
        HashMap<String, Vec<DictEntry>>,
    ) {
        let mut phrase_index: HashMap<String, Vec<DictEntry>> = HashMap::new();
        let mut char_index: HashMap<String, Vec<DictEntry>> = HashMap::new();

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Could not load dictionary '{}': {}", path, e);
                return (phrase_index, char_index);
            }
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 3 {
                continue;
            }

            let word = parts[0].to_string();
            let pinyin_str = parts[1];
            let weight: u32 = match parts[2].trim().parse() {
                Ok(w) => w,
                Err(_) => continue,
            };

            let syllables: Vec<&str> = pinyin_str.split_whitespace().collect();
            if syllables.is_empty() {
                continue;
            }

            let key = syllables.concat();
            let entry = DictEntry { word, weight };

            if syllables.len() == 1 {
                char_index
                    .entry(syllables[0].to_string())
                    .or_default()
                    .push(DictEntry {
                        word: entry.word.clone(),
                        weight: entry.weight,
                    });
            }

            phrase_index.entry(key).or_default().push(entry);
        }

        for entries in phrase_index.values_mut() {
            entries.sort_by(|a, b| b.weight.cmp(&a.weight));
        }
        for entries in char_index.values_mut() {
            entries.sort_by(|a, b| b.weight.cmp(&a.weight));
        }

        (phrase_index, char_index)
    }

    fn load_en_dict(path: &str, index: &mut HashMap<String, Vec<DictEntry>>) {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Could not load dictionary '{}': {}", path, e);
                return;
            }
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 {
                continue;
            }

            let word = parts[0].to_string();
            let weight = if parts.len() >= 3 {
                parts[2].trim().parse().unwrap_or(100)
            } else {
                100
            };

            let key = word.to_lowercase();
            index
                .entry(key)
                .or_default()
                .push(DictEntry { word, weight });
        }

        for entries in index.values_mut() {
            entries.sort_by(|a, b| b.weight.cmp(&a.weight));
        }
    }

    fn merge_indices(
        target_phrase: &mut HashMap<String, Vec<DictEntry>>,
        target_char: &mut HashMap<String, Vec<DictEntry>>,
        source_phrase: HashMap<String, Vec<DictEntry>>,
        source_char: HashMap<String, Vec<DictEntry>>,
    ) {
        for (key, entries) in source_phrase {
            target_phrase.entry(key).or_default().extend(entries);
        }
        for (key, entries) in source_char {
            target_char.entry(key).or_default().extend(entries);
        }

        for entries in target_phrase.values_mut() {
            entries.sort_by(|a, b| b.weight.cmp(&a.weight));
        }
        for entries in target_char.values_mut() {
            entries.sort_by(|a, b| b.weight.cmp(&a.weight));
        }
    }

    pub fn translate_phrase(&self, input: &str) -> Vec<Candidate> {
        let mut result = Vec::new();

        if let Some(entries) = self.phrase_index.get(input) {
            for entry in entries {
                result.push(Candidate::new(entry.word.clone(), entry.weight as f32));
            }
        }

        result
    }

    pub fn translate_segment(&self, seg: &Segment) -> Vec<Candidate> {
        let mut result = Vec::new();

        if let Some(entries) = self.char_index.get(&seg.code) {
            for entry in entries {
                result.push(Candidate::new(entry.word.clone(), entry.weight as f32));
            }
        }

        result
    }

    pub fn translate_en(&self, input: &str) -> Vec<Candidate> {
        let mut result = Vec::new();

        if let Some(entries) = self.en_index.get(input) {
            for entry in entries {
                result.push(Candidate::new(entry.word.clone(), entry.weight as f32));
            }
        }

        result
    }

    pub fn get_char_candidates(&self, syllable: &str) -> Vec<(String, u32)> {
        self.char_index
            .get(syllable)
            .map(|entries| entries.iter().map(|e| (e.word.clone(), e.weight)).collect())
            .unwrap_or_default()
    }
}
