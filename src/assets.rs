// src/assets.rs

use std::sync::LazyLock;
use serde::Deserialize;
use std::collections::HashMap;

#[allow(dead_code)] // This clears the "never read" warnings
#[derive(Debug, Deserialize, Clone)]

pub struct Language {
    #[serde(rename = "type")]
    pub lang_type: String,
    pub color: Option<String>,
    pub extensions: Option<Vec<String>>,
    pub filenames: Option<Vec<String>>,
}

const RAW_YML: &str = include_str!("../data/languages.yml");

// 1. Update LANG_MAP
pub static LANG_MAP: LazyLock<HashMap<String, Language>> =
    LazyLock::new(|| serde_yml::from_str(RAW_YML).expect("languages.yml is malformed"));

// 2. Update EXTENSION_LOOKUP
pub static EXTENSION_LOOKUP: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (name, lang) in LANG_MAP.iter() {
        if let Some(extensions) = &lang.extensions {
            for ext in extensions {
                let clean_ext = ext.trim_start_matches('.').to_lowercase();
                map.insert(clean_ext, name.as_str()); 
            }
        }
    }
    map
});

// 3. Update FILENAME_LOOKUP
pub static FILENAME_LOOKUP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (name, lang) in LANG_MAP.iter() {
        if let Some(filenames) = &lang.filenames {
            for f in filenames {
                map.insert(f.clone(), name.clone());
            }
        }
    }
    map
});
