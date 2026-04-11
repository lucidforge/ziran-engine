use std::collections::HashMap;

use crate::candidate::Candidate;
use crate::dict::LoadedDictionaries;

#[derive(Clone)]
pub struct DictEntry {
    pub word: String,
    pub weight: u32,
}

pub struct SimpleTranslator {
    pub phrase_index: HashMap<String, Vec<DictEntry>>,
    pub char_index: HashMap<String, Vec<DictEntry>>,
    en_index: HashMap<String, Vec<DictEntry>>,
}

impl SimpleTranslator {
    /// Create translator from pre-loaded dictionaries (used by Pipeline)
    pub fn from_loaded_dictionaries(dicts: &LoadedDictionaries) -> Self {
        // Convert DictCandidate (text) to DictEntry (word)
        let phrase_index: HashMap<String, Vec<DictEntry>> = dicts
            .phrase_index
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter()
                        .map(|c| DictEntry {
                            word: c.text.clone(),
                            weight: c.weight,
                        })
                        .collect(),
                )
            })
            .collect();

        let char_index: HashMap<String, Vec<DictEntry>> = dicts
            .char_index
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter()
                        .map(|c| DictEntry {
                            word: c.text.clone(),
                            weight: c.weight,
                        })
                        .collect(),
                )
            })
            .collect();

        let en_index: HashMap<String, Vec<DictEntry>> = dicts
            .en_index
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter()
                        .map(|c| DictEntry {
                            word: c.text.clone(),
                            weight: c.weight,
                        })
                        .collect(),
                )
            })
            .collect();

        Self {
            phrase_index,
            char_index,
            en_index,
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

    pub fn translate_en(&self, input: &str) -> Vec<Candidate> {
        let mut result = Vec::new();

        if let Some(entries) = self.en_index.get(input) {
            for entry in entries {
                result.push(Candidate::new(entry.word.clone(), entry.weight as f32));
            }
        }

        result
    }
}
