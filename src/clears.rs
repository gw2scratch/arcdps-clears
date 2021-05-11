use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::api::Gw2Api;

pub struct ClearData {
    raids: Option<RaidWings>,
    state: Option<RaidClearState>,
}

impl ClearData {
    pub fn new() -> Self {
        ClearData { raids: None, state: None }
    }
}

impl ClearData {
    pub fn raids(&self) -> &Option<RaidWings> {
        &self.raids
    }
    pub fn state(&self) -> &Option<RaidClearState> {
        &self.state
    }
    pub fn set_raids(&mut self, raids: Option<RaidWings>) {
        self.raids = raids;
    }
    pub fn set_state(&mut self, state: Option<RaidClearState>) {
        self.state = state;
    }
}

pub struct RaidWings {
    wings: Vec<RaidWing>
}

impl RaidWings {
    pub fn new(wings: Vec<RaidWing>) -> Self {
        RaidWings { wings }
    }
    pub fn wings(&self) -> &Vec<RaidWing> {
        &self.wings
    }
}

#[derive(Serialize, Deserialize)]
pub struct RaidWing {
    id: String,
    #[serde(rename(deserialize = "events"))]
    encounters: Vec<RaidEncounter>,
}

impl RaidWing {
    pub fn new(id: String, encounters: Vec<RaidEncounter>) -> Self {
        RaidWing { id, encounters }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn encounters(&self) -> &Vec<RaidEncounter> {
        &self.encounters
    }
}

#[derive(Serialize, Deserialize)]
pub struct RaidEncounter {
    id: String,
    #[serde(rename(deserialize = "type"))]
    encounter_type: EncounterType,
}

impl RaidEncounter {
    pub fn new(id: String, encounter_type: EncounterType) -> Self {
        RaidEncounter { id, encounter_type }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn encounter_type(&self) -> &EncounterType {
        &self.encounter_type
    }
    pub fn english_name(&self) -> String {
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
        let parts = self.id.split("_");
        parts.enumerate().map(|(i, x)| {
            // The first word should always get capitalized
            if i > 0 && ["of", "in", "the"].contains(&x) {
                x.to_string()
            } else {
                capitalize(x)
            }
        }).join(" ")
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum EncounterType {
    Unknown,
    Checkpoint,
    Boss,
}

pub struct RaidClearState {
    // TODO: Store last update time as well
    finished_encounter_ids: Vec<String>
}

impl RaidClearState {
    pub fn new(finished_encounter_ids: Vec<String>) -> Self {
        RaidClearState { finished_encounter_ids }
    }
}

impl RaidClearState {
    pub fn is_finished(&self, encounter: &RaidEncounter) -> bool {
        self.finished_encounter_ids.iter().any(|x| *x == encounter.id)
    }
}
