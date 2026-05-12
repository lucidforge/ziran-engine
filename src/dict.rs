use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use crate::dict_compiler;
use crate::schema::SchemaConfig;
use crate::trie::Trie;

#[derive(Clone)]
pub struct DictCandidate {
    pub text: String,
    pub weight: u32,
}

type PendingDictIndex = HashMap<String, Vec<(DictCandidate, u64)>>;

/// Bilingual mapping: Chinese text -> Vec<(English translation, weight)>
pub type BilingualIndex = HashMap<String, Vec<(String, u32)>>;

pub struct LoadedDictionaries {
    pub phrase_trie: Arc<Trie<DictCandidate>>,
    pub char_trie: Arc<Trie<DictCandidate>>,
    pub en_trie: Arc<Trie<DictCandidate>>,
    pub bilingual_index: Arc<BilingualIndex>,
}

pub fn load_dictionaries(schema: &SchemaConfig) -> LoadedDictionaries {
    let chinese_paths: Vec<String> = schema
        .dictionaries
        .iter()
        .map(|name| resolve_dict_path(name))
        .collect();
    let char_paths: Vec<String> = schema
        .char_dictionaries
        .iter()
        .map(|name| resolve_dict_path(name))
        .collect();
    let english_paths: Vec<String> = schema
        .english_dictionaries
        .iter()
        .map(|name| resolve_dict_path(name))
        .collect();

    let phrase_trie = Arc::new(load_trie_with_cache(
        &chinese_paths,
        "data/cache_zh.bin",
        parse_chinese_dicts,
    ));
    let char_trie = Arc::new(load_trie_with_cache(
        &char_paths,
        "data/cache_char.bin",
        parse_chinese_dicts,
    ));
    let en_trie = Arc::new(load_trie_with_cache(
        &english_paths,
        "data/cache_en.bin",
        parse_english_dicts,
    ));

    let bilingual_paths: Vec<String> = schema
        .bilingual_dictionaries
        .iter()
        .map(|name| resolve_dict_path(name))
        .collect();
    let bilingual_index = Arc::new(load_bilingual_dicts(&bilingual_paths));

    LoadedDictionaries {
        phrase_trie,
        char_trie,
        en_trie,
        bilingual_index,
    }
}

fn resolve_dict_path(name: &str) -> String {
    if name.contains('/') || name.ends_with(".dict.yaml") {
        name.to_string()
    } else {
        format!("data/{}.dict.yaml", name)
    }
}

// ── Generic dict file parser ──────────────────────────────────────────

fn parse_dict_file<T, F>(path: &str, mut line_handler: F) -> Vec<T>
where
    F: FnMut(&str, &str, u32) -> Option<T>,
{
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Warning: Could not load dictionary '{}': {}", path, err);
            return Vec::new();
        }
    };

    let mut result = Vec::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line == "---" || line == "..." {
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

        let weight = parts
            .get(2)
            .and_then(|v| v.trim().parse::<u32>().ok())
            .unwrap_or(1);

        if let Some(item) = line_handler(word, code, weight) {
            result.push(item);
        }
    }
    result
}

// ── Chinese dict parsing ──────────────────────────────────────────────

fn parse_chinese_dicts(paths: &[String]) -> Vec<(String, DictCandidate)> {
    let mut pending: PendingDictIndex = HashMap::new();
    let mut order = 0_u64;

    for path in paths {
        parse_dict_file(path, |word, code, raw_weight| {
            let syllable_parts: Vec<&str> = code.split_whitespace().collect();
            if syllable_parts.is_empty() {
                return None::<()>;
            }
            let char_count = word.chars().count();
            let weight = adjust_weight(raw_weight, char_count);
            let key = syllable_parts.concat();
            push_entry(
                &mut pending,
                key,
                DictCandidate {
                    text: word.to_string(),
                    weight,
                },
                order,
            );
            order += 1;
            Some(())
        });
    }

    finalize_to_entries(pending)
}

fn parse_english_dicts(paths: &[String]) -> Vec<(String, DictCandidate)> {
    let mut pending: PendingDictIndex = HashMap::new();
    let mut order = 0_u64;

    for path in paths {
        parse_dict_file(path, |word, code, weight| {
            let code = code.to_lowercase();
            if code.is_empty() {
                return None::<()>;
            }
            push_entry(
                &mut pending,
                code,
                DictCandidate {
                    text: word.to_string(),
                    weight,
                },
                order,
            );
            order += 1;
            Some(())
        });
    }

    finalize_to_entries(pending)
}

fn load_bilingual_dicts(paths: &[String]) -> BilingualIndex {
    let mut index: BilingualIndex = HashMap::new();

    for path in paths {
        parse_dict_file(path, |chinese, english, weight| {
            if chinese.is_empty() || english.is_empty() {
                return None::<()>;
            }
            index
                .entry(chinese.to_string())
                .or_default()
                .push((english.to_string(), weight));
            Some(())
        });
    }

    for entries in index.values_mut() {
        entries.sort_by(|a, b| b.1.cmp(&a.1));
    }

    index
}

// ── Trie loading with cache ───────────────────────────────────────────

fn load_trie_with_cache(
    paths: &[String],
    cache_path: &str,
    parse_fn: fn(&[String]) -> Vec<(String, DictCandidate)>,
) -> Trie<DictCandidate> {
    let checksum = dict_compiler::compute_source_checksum(paths);

    if let Some(trie) = dict_compiler::load_from_cache(cache_path, checksum) {
        eprintln!("Loaded from cache: {}", cache_path);
        return trie;
    }

    eprintln!("Building trie from source files...");
    let entries = parse_fn(paths);
    let trie = build_trie_from_entries(&entries);

    dict_compiler::compile_to_cache(cache_path, &entries, checksum);
    eprintln!("Compiled cache: {}", cache_path);

    trie
}

fn build_trie_from_entries(entries: &[(String, DictCandidate)]) -> Trie<DictCandidate> {
    let mut trie = Trie::new();
    for (key, entry) in entries {
        trie.insert(key, entry.clone());
    }
    trie
}

// ── Helpers ───────────────────────────────────────────────────────────

fn push_entry(index: &mut PendingDictIndex, key: String, entry: DictCandidate, order: u64) {
    index.entry(key).or_default().push((entry, order));
}

/// Adjust dictionary weight by character count.
/// Multi-char words get a boost to favor phrases over single-char splits.
fn adjust_weight(raw_weight: u32, char_count: usize) -> u32 {
    let multiplier: f32 = match char_count {
        0 | 1 => 1.0,
        2 => 2.0,
        3 => 3.0,
        _ => 4.0,
    };
    ((raw_weight as f32 * multiplier) as u32).max(1)
}

fn finalize_to_entries(mut pending: PendingDictIndex) -> Vec<(String, DictCandidate)> {
    let mut result = Vec::new();
    for (key, mut entries) in pending.drain() {
        entries.sort_by(|a, b| {
            b.0.weight
                .cmp(&a.0.weight)
                .then_with(|| a.1.cmp(&b.1))
        });
        for (entry, _) in entries {
            result.push((key.clone(), entry));
        }
    }
    result
}
