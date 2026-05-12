use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

const USER_BOOST: f32 = 2.0;

struct FreqEntry {
    count: u32,
    last_used: u64,
}

pub struct UserFreq {
    entries: HashMap<String, FreqEntry>,
    path: String,
    dirty: bool,
}

impl UserFreq {
    pub fn load(path: &str) -> Self {
        let entries = match fs::read_to_string(path) {
            Ok(content) => {
                let mut map = HashMap::new();
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 2 {
                        let word = parts[0].to_string();
                        let count = parts[1].parse::<u32>().unwrap_or(0);
                        let last_used = parts
                            .get(2)
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(0);
                        if count > 0 {
                            map.insert(word, FreqEntry { count, last_used });
                        }
                    }
                }
                map
            }
            Err(_) => HashMap::new(),
        };

        Self {
            entries,
            path: path.to_string(),
            dirty: false,
        }
    }

    pub fn save(&self) {
        if !self.dirty {
            return;
        }

        let mut entries: Vec<(&String, &FreqEntry)> = self.entries.iter().collect();
        entries.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        let mut content = String::from("# user frequency data\n");
        for (word, entry) in entries {
            content.push_str(&format!("{}\t{}\t{}\n", word, entry.count, entry.last_used));
        }

        if let Ok(mut file) = fs::File::create(&self.path) {
            let _ = file.write_all(content.as_bytes());
        }
    }

    pub fn record(&mut self, text: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = self.entries.entry(text.to_string()).or_insert(FreqEntry {
            count: 0,
            last_used: 0,
        });
        entry.count += 1;
        entry.last_used = now;
        self.dirty = true;
    }

    pub fn get_boost(&self, text: &str) -> f32 {
        match self.entries.get(text) {
            Some(entry) => (1.0 + entry.count as f32).ln() * USER_BOOST,
            None => 0.0,
        }
    }
}
