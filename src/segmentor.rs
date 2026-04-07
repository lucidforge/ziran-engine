use std::collections::HashSet;
use std::fs;

use crate::context::Context;
use crate::segment::Segment;

pub struct PinyinSegmentor {
    syllables: HashSet<String>,
    max_len: usize,
}

impl PinyinSegmentor {
    pub fn new() -> Self {
        let mut syllables = HashSet::new();
        let mut max_len: usize = 0;

        let cn_files = [
            "data/dict.txt",
            "data/ext.dict.yaml",
            "data/others.dict.yaml",
        ];

        for file in &cn_files {
            let (s, m) = Self::load_syllables(file);
            syllables.extend(s);
            if m > max_len {
                max_len = m;
            }
        }

        Self { syllables, max_len }
    }

    fn load_syllables(path: &str) -> (HashSet<String>, usize) {
        let mut syllables: HashSet<String> = HashSet::new();
        let mut max_len: usize = 0;

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return (syllables, max_len),
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

            let pinyin_str = parts[1];
            let pinyin_parts: Vec<&str> = pinyin_str.split_whitespace().collect();

            for syl in pinyin_parts {
                if syl.len() > max_len {
                    max_len = syl.len();
                }
                syllables.insert(syl.to_string());
            }
        }

        (syllables, max_len)
    }

    pub fn segment(&self, ctx: &mut Context) {
        if ctx.raw_input.is_empty() {
            return;
        }

        let bytes = ctx.raw_input.as_bytes();
        let len = bytes.len();
        let mut pos = 0;

        while pos < len {
            let mut found = false;
            let remaining = len - pos;
            let max_step = remaining.min(self.max_len);

            for step in (1..=max_step).rev() {
                let segment_str = String::from_utf8_lossy(&bytes[pos..pos + step]).to_string();
                if self.syllables.contains(&segment_str) {
                    ctx.segments.push(Segment::new(segment_str));
                    pos += step;
                    found = true;
                    break;
                }
            }

            if !found {
                let ch = bytes[pos] as char;
                ctx.segments.push(Segment::new(ch.to_string()));
                pos += 1;
            }
        }
    }
}
