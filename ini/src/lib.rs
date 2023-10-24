#![forbid(unsafe_code)]

use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////

pub type IniFile = HashMap<String, HashMap<String, String>>;

pub fn parse(content: &str) -> IniFile {
    let mut out_hash_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut key_value_map: HashMap<String, String> = HashMap::new();
    let mut last_section: &str = "-1";
    if content.trim().is_empty() {
        return out_hash_map;
    }
    for line in str::lines(content) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('[') {
            let section = &trimmed.trim()[1..trimmed.len() - 1];
            if section.starts_with('[') || !trimmed.trim().ends_with(']') {
                panic!()
            }
            if last_section != section {
                key_value_map.clear();
            }

            last_section = section;
            out_hash_map.insert(last_section.to_string(), key_value_map.clone());
        } else {
            let key;
            let mut value = "";
            if trimmed.contains('=') {
                let v: Vec<&str> = trimmed.split('=').collect();
                if v.len() > 2 || last_section == "-1" {
                    panic!();
                }
                key = v[0].trim();
                value = v[1].trim();
            } else {
                key = trimmed.trim();
            }
            key_value_map.insert(key.to_string(), value.to_string());
            out_hash_map.insert(last_section.to_string(), key_value_map.clone());
        }
    }
    out_hash_map
}
