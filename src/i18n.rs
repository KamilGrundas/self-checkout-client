use std::collections::HashMap;
use std::fs;

pub struct I18n {
    translations: HashMap<String, String>,
}

impl I18n {
    pub fn load(lang: &str) -> Self {
        let path = format!("assets/lang/{}.json", lang);

        let data = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());

        let translations: HashMap<String, String> = serde_json::from_str(&data).unwrap_or_default();

        Self { translations }
    }

    pub fn t(&self, key: &str) -> String {
        self.translations
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}
