use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use uuid::Uuid;
use crate::settings::ApiKey;

pub struct ClearData {
    raids: Option<RaidWings>,
    state: HashMap<Uuid, RaidClearState>,
}

impl ClearData {
    pub fn new() -> Self {
        ClearData { raids: None, state: HashMap::new() }
    }
}

impl ClearData {
    pub fn raids(&self) -> &Option<RaidWings> {
        &self.raids
    }
    pub fn state(&self, key: &ApiKey) -> Option<&RaidClearState> {
        self.state.get(key.id())
    }
    pub fn set_raids(&mut self, raids: Option<RaidWings>) {
        self.raids = raids;
    }
    pub fn set_state(&mut self, uuid: Uuid, state: Option<RaidClearState>) {
        if let Some(state) = state {
            self.state.insert(uuid, state);
        } else {
            self.state.remove(&uuid);
        }
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
impl RaidClearState {
    pub fn is_finished(&self, encounter: &RaidEncounter) -> bool {
        self.finished_encounter_ids.iter().any(|x| *x == encounter.id)
    }

    pub fn finished_encounter_ids(&self) -> &Vec<String> {
        &self.finished_encounter_ids
    }
}
