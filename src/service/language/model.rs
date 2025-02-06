use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    Chinese,
    German,
    French,
    Japanese,
    Spanish,
}

impl FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Ok(Language::English),
            "zh" | "chinese" => Ok(Language::Chinese),
            "de" | "german" => Ok(Language::German),
            "fr" | "french" => Ok(Language::French),
            "ja" | "japanese" => Ok(Language::Japanese),
            "es" | "spanish" => Ok(Language::Spanish),
            _ => Err(format!("Unknown language code: {}", s)),
        }
    }
}

impl ToString for Language {
    fn to_string(&self) -> String {
        match self {
            Language::English => "en".to_string(),
            Language::Chinese => "zh".to_string(),
            Language::German => "de".to_string(),
            Language::French => "fr".to_string(),
            Language::Japanese => "ja".to_string(),
            Language::Spanish => "es".to_string(),
        }
    }
}
