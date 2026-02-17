// src/assets.rs

use serde::Deserialize;
use std::collections::HashMap;
use once_cell::sync::Lazy;

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

pub static LANG_MAP: Lazy<HashMap<String, Language>> = Lazy::new(|| {
    serde_yaml::from_str(RAW_YML).expect("languages.yml is malformed")
});

pub static EXTENSION_LOOKUP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for (name, lang) in LANG_MAP.iter() {
        if let Some(exts) = &lang.extensions {
            for ext in exts {
                map.insert(ext.clone(), name.clone());
            }
        }
    }
    map
});

pub static FILENAME_LOOKUP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for (name, lang) in LANG_MAP.iter() {
        if let Some(files) = &lang.filenames {
            for f in files {
                map.insert(f.clone(), name.clone());
            }
        }
    }
    map
});
