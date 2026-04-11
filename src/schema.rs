use std::fs;

pub struct SchemaConfig {
    pub dictionaries: Vec<String>,
    pub english_dictionaries: Vec<String>,
}

impl SchemaConfig {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("failed to read schema '{}': {}", path, e))?;

        let mut dictionaries = Vec::new();
        let mut english_dictionaries = Vec::new();
        let mut current_section = String::new();
        let mut current_list = String::new();

        for raw_line in content.lines() {
            let line = raw_line.trim_end();
            let trimmed = line.trim();

            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed == "---"
                || trimmed == "..."
            {
                continue;
            }

            let indent = line.len() - line.trim_start().len();

            if indent == 0 && trimmed.ends_with(':') {
                current_section = trimmed.trim_end_matches(':').to_string();
                current_list.clear();
                continue;
            }

            if indent == 2 && trimmed.ends_with(':') {
                current_list = trimmed.trim_end_matches(':').to_string();
                continue;
            }

            if indent >= 4 && trimmed.starts_with("- ") {
                let item = trimmed
                    .trim_start_matches("- ")
                    .split_once('#')
                    .map(|(value, _)| value.trim())
                    .unwrap_or_else(|| trimmed.trim_start_matches("- ").trim());

                if item.is_empty() {
                    continue;
                }

                if current_section == "translator" && current_list == "dictionaries" {
                    dictionaries.push(item.to_string());
                } else if current_section == "translator"
                    && current_list == "english_dictionaries"
                {
                    english_dictionaries.push(item.to_string());
                }
            }
        }

        if dictionaries.is_empty() {
            return Err("schema translator.dictionaries is empty".to_string());
        }

        Ok(Self {
            dictionaries,
            english_dictionaries,
        })
    }
}
