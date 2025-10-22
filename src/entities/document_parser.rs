use std::fs::read_to_string;

use serde_json::{Value, from_str};

pub struct DocumentParser;

impl DocumentParser {
    pub fn load_file(value: impl Into<String>) -> Vec<Value> {
        let file_content = read_to_string(value.into()).expect("Failed to read demo.json");
        from_str(&file_content).expect("Failed to parse JSON from demo.json")
    }
}
