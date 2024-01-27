use std::{collections::HashMap, path::Display};

pub struct Info {
    keys: HashMap<String, String>,
}

impl Info {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn set_value_for_key(&mut self, key: &str, value: &str) {
        if (key.contains("\\") || value.contains("\\")) {
            panic!("Key or value contains a backslash");
        }

        self.keys.insert(key.to_string(), value.to_string());
    }

    pub fn serialize(&self) -> String {
        // join keys and values with a backslash
        let mut result = String::new();

        let mut keys: Vec<_> = self.keys.keys().collect();
        keys.sort();

        for key in keys {
            result.push_str(&format!("\\{}\\{}", key, self.keys[key]));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_serialize() {
        let mut info = Info::new();

        info.set_value_for_key("key1", "value1");
        info.set_value_for_key("key2", "value2");

        assert_eq!(info.serialize(), "\\key1\\value1\\key2\\value2");
    }
}
