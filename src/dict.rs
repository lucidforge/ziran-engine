use std::collections::{HashMap, HashSet};
use std::fs;

use crate::schema::SchemaConfig;

#[derive(Clone)]
pub struct DictCandidate {
    pub text: String,
    pub weight: u32,
}

pub type DictIndex = HashMap<String, Vec<DictCandidate>>;
type PendingDictIndex = HashMap<String, Vec<(DictCandidate, u64)>>;

pub struct LoadedDictionaries {
    pub phrase_index: DictIndex,
    pub char_index: DictIndex,
    pub en_index: DictIndex,
    pub syllables: HashSet<String>,
    pub max_syllable_len: usize,
}

pub fn load_dictionaries(schema: &SchemaConfig) -> LoadedDictionaries {
    let chinese_paths: Vec<String> = schema
        .dictionaries
        .iter()
        .map(|name| resolve_dict_path(name))
        .collect();
    let english_paths: Vec<String> = schema
        .english_dictionaries
        .iter()
        .map(|name| resolve_dict_path(name))
        .collect();

    let (phrase_index, char_index, syllables, max_syllable_len) =
        load_chinese_dicts(&chinese_paths);
    let en_index = load_english_dicts(&english_paths);

    LoadedDictionaries {
        phrase_index,
        char_index,
        en_index,
        syllables,
        max_syllable_len,
    }
}

fn resolve_dict_path(name: &str) -> String {
    if name.contains('/') || name.ends_with(".dict.yaml") {
        name.to_string()
    } else {
        format!("data/{}.dict.yaml", name)
    }
}

fn load_chinese_dicts(paths: &[String]) -> (DictIndex, DictIndex, HashSet<String>, usize) {
    let mut phrase_index: PendingDictIndex = HashMap::new();
    let mut char_index: PendingDictIndex = HashMap::new();
    let mut syllables = HashSet::new();
    let mut max_syllable_len = 0;
    let mut order = 0_u64;

    for path in paths {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Warning: Could not load dictionary '{}': {}", path, err);
                continue;
            }
        };

        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty()
                || line.starts_with('#')
                || line == "---"
                || line == "..."
            {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 {
                continue;
            }

            let word = parts[0].trim();
            let code = parts[1].trim();
            if word.is_empty() || code.is_empty() {
                continue;
            }

            let syllable_parts: Vec<&str> = code.split_whitespace().collect();
            if syllable_parts.is_empty() {
                continue;
            }

            let weight = parts
                .get(2)
                .and_then(|value| value.trim().parse::<u32>().ok())
                .unwrap_or(1);

            for syllable in &syllable_parts {
                syllables.insert((*syllable).to_string());
                max_syllable_len = max_syllable_len.max(syllable.len());
            }

            let key = syllable_parts.concat();
            push_entry(
                &mut phrase_index,
                key,
                DictCandidate {
                    text: word.to_string(),
                    weight,
                },
                order,
            );

            if syllable_parts.len() == 1 {
                push_entry(
                    &mut char_index,
                    syllable_parts[0].to_string(),
                    DictCandidate {
                        text: word.to_string(),
                        weight,
                    },
                    order,
                );
            }

            order += 1;
        }
    }

    let phrase_index = finalize_index(phrase_index);
    let char_index = finalize_index(char_index);

    (phrase_index, char_index, syllables, max_syllable_len)
}

fn load_english_dicts(paths: &[String]) -> DictIndex {
    let mut en_index: PendingDictIndex = HashMap::new();
    let mut order = 0_u64;

    for path in paths {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Warning: Could not load dictionary '{}': {}", path, err);
                continue;
            }
        };

        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty()
                || line.starts_with('#')
                || line == "---"
                || line == "..."
            {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 {
                continue;
            }

            let word = parts[0].trim();
            let code = parts[1].trim().to_lowercase();
            if word.is_empty() || code.is_empty() {
                continue;
            }

            let weight = parts
                .get(2)
                .and_then(|value| value.trim().parse::<u32>().ok())
                .unwrap_or(1);

            push_entry(
                &mut en_index,
                code,
                DictCandidate {
                    text: word.to_string(),
                    weight,
                },
                order,
            );

            order += 1;
        }
    }

    finalize_index(en_index)
}

fn push_entry(index: &mut PendingDictIndex, key: String, entry: DictCandidate, order: u64) {
    index.entry(key).or_default().push((entry, order));
}

fn finalize_index(index: PendingDictIndex) -> DictIndex {
    let mut finalized = HashMap::new();

    for (key, mut entries) in index {
        entries.sort_by(|a, b| {
            b.0.weight
                .cmp(&a.0.weight)
                .then_with(|| a.1.cmp(&b.1))
        });
        finalized.insert(
            key,
            entries.into_iter().map(|(entry, _)| entry).collect(),
        );
    }

    finalized
}
