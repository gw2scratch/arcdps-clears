use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use std::sync::Mutex;
use arcdps::imgui::{ImStr, ImString};

pub struct Translation {
    translations: HashMap<String, String>
}

impl Translation {
    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let pairs: HashMap<String, String> = serde_json::from_reader(reader)?;

        Ok(Translation {translations: pairs})
    }

    pub fn load_from_string(str: &str) -> Result<Self, Box<dyn Error>> {
        let pairs: HashMap<String, String> = serde_json::from_str(str)?;

        Ok(Translation {translations: pairs})
    }

    pub fn im_string(&self, key: &str) -> ImString {
        if let Some(translation) = self.translations.get(key) {
            ImString::new(translation)
        } else {
            ImString::new(format!("(({}))", key))
        }
    }
}

pub fn get_default_translation_contents() -> &'static str {
    include_str!("../translations/arcdps_lang_clears.json")
}