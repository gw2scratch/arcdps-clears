use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use arcdps::imgui::{ImString};
use serde::{Serialize, Deserialize};
use crate::clears::RaidEncounter;
use itertools::Itertools;

#[derive(Serialize, Deserialize)]
pub struct Translation {
    strings: HashMap<String, String>,
    encounter_short_names: HashMap<String, String>
}

impl Translation {
    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let translation: Translation = serde_json::from_reader(reader)?;

        Ok(translation)
    }

    pub fn load_from_string(str: &str) -> Result<Self, Box<dyn Error>> {
        let translation: Translation = serde_json::from_str(str)?;

        Ok(translation)
    }

    pub fn im_string(&self, key: &str) -> ImString {
        if let Some(translation) = self.strings.get(key) {
            ImString::new(translation)
        } else {
            ImString::new(format!("(({}))", key))
        }
    }

    pub fn encounter_short_name_im_string(&self, encounter: &RaidEncounter) -> ImString {
        if let Some(translation) = self.encounter_short_names.get(encounter.id()) {
            ImString::new(translation)
        } else {
            // We fall back to an English name from the API if there is no short name
            // defined in the translation.
            ImString::new(encounter_english_name(encounter))
        }
    }

}

pub fn get_default_translation_contents() -> &'static str {
    include_str!("../translations/arcdps_lang_clears.json")
}

pub fn encounter_english_name(encounter: &RaidEncounter) -> String {
    fn capitalize(str: &str) -> String {
        let capitalized = str.chars().enumerate().map(|(i, char)| {
            if i == 0 {
                char.to_uppercase().next().unwrap()
            } else {
                char
            }
        }).collect();
        capitalized
    }
    let parts = encounter.id().split("_");
    parts.enumerate().map(|(i, x)| {
        // The first word should always get capitalized
        if i > 0 && ["of", "in", "the"].contains(&x) {
            x.to_string()
        } else {
            capitalize(x)
        }
    }).join(" ")
}
