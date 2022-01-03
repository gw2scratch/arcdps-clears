use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use chrono::{DateTime, Duration, TimeZone, Utc};
use uuid::Uuid;
use crate::settings::ApiKey;

pub struct ClearData {
    raids: Option<RaidWings>,
    state: HashMap<Uuid, RaidClearState>,
}

impl ClearData {
    pub fn new() -> Self {
        ClearData {
            raids: None,
            state: HashMap::new(),
        }
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
    pub fn finished_encounters(&self, key: &ApiKey) -> Option<&FinishedEncountersStore> {
        static EMPTY_CLEARS: FinishedEncountersStore = FinishedEncountersStore::empty();

        let state = self.state(key)?;
        let last_reset = last_raid_reset(Utc::now());
        if last_reset >= state.last_api_update_time {
            Some(&EMPTY_CLEARS)
        } else {
            Some(&state.finished_encounters)
        }
    }
}

pub struct RaidWings {
    wings: Vec<RaidWing>,
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

pub struct FinishedEncountersStore {
    finished_encounter_ids: Vec<String>,
}

impl FinishedEncountersStore {
    pub fn new(finished_encounter_ids: Vec<String>) -> Self {
        FinishedEncountersStore { finished_encounter_ids }
    }

    pub const fn empty() -> Self {
        FinishedEncountersStore { finished_encounter_ids: Vec::new() }
    }
}

pub struct RaidClearState {
    finished_encounters: FinishedEncountersStore,
    last_check_time: DateTime<Utc>,
    last_api_update_time: DateTime<Utc>,
}

impl RaidClearState {
    pub fn new(finished_encounters: FinishedEncountersStore, last_check_time: DateTime<Utc>, last_api_update_time: DateTime<Utc>) -> Self {
        RaidClearState { finished_encounters, last_check_time, last_api_update_time }
    }
    pub fn finished_encounters(&self) -> &FinishedEncountersStore {
        &self.finished_encounters
    }
    pub fn last_check_time(&self) -> DateTime<Utc> {
        self.last_check_time
    }
    pub fn last_api_update_time(&self) -> DateTime<Utc> {
        self.last_api_update_time
    }
}

#[allow(dead_code)]
impl FinishedEncountersStore {
    pub fn is_finished(&self, encounter: &RaidEncounter) -> bool {
        self.finished_encounter_ids.iter().any(|x| *x == encounter.id)
    }

    pub fn finished_encounter_ids(&self) -> &Vec<String> {
        &self.finished_encounter_ids
    }
}

pub(crate) fn last_raid_reset(current_time: DateTime<Utc>) -> DateTime<Utc> {
    pub const WEEK_IN_SECONDS: i64 = 604800;

    let past_reset = Utc.ymd(2012, 1, 2).and_hms(7, 30, 0);
    current_time - Duration::seconds((current_time - past_reset).num_seconds() % WEEK_IN_SECONDS)
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};
    use super::*;

    #[test]
    fn last_raid_reset_is_previous_monday_within_month() {
        // Previous monday is 2021-12-27
        let time = Utc.ymd(2021, 12, 31).and_hms(15, 0, 0);
        let reset = last_raid_reset(time);
        assert_eq!(2021, reset.year());
        assert_eq!(12, reset.month());
        assert_eq!(27, reset.day());
        assert_eq!(7, reset.hour());
        assert_eq!(30, reset.minute());
        assert_eq!(0, reset.second());
    }

    #[test]
    fn last_raid_reset_is_previous_monday_on_monday_before_reset() {
        // Previous monday is 2021-12-20
        let time = Utc.ymd(2021, 12, 27).and_hms(0, 10, 0);
        let reset = last_raid_reset(time);
        assert_eq!(2021, reset.year());
        assert_eq!(12, reset.month());
        assert_eq!(20, reset.day());
        assert_eq!(7, reset.hour());
        assert_eq!(30, reset.minute());
        assert_eq!(0, reset.second());
    }

    #[test]
    fn last_raid_reset_is_previous_monday_on_reset() {
        // This is the reset time
        let time = Utc.ymd(2021, 12, 27).and_hms(7, 30, 0);
        let reset = last_raid_reset(time);
        assert_eq!(2021, reset.year());
        assert_eq!(12, reset.month());
        assert_eq!(27, reset.day());
        assert_eq!(7, reset.hour());
        assert_eq!(30, reset.minute());
        assert_eq!(0, reset.second());
    }

    #[test]
    fn last_raid_reset_is_previous_monday_after_month_change() {
        // Previous monday is 2021-11-29
        let time = Utc.ymd(2021, 12, 3).and_hms(15, 0, 0);
        let reset = last_raid_reset(time);
        assert_eq!(2021, reset.year());
        assert_eq!(11, reset.month());
        assert_eq!(29, reset.day());
        assert_eq!(7, reset.hour());
        assert_eq!(30, reset.minute());
        assert_eq!(0, reset.second());
    }

    #[test]
    fn last_raid_reset_is_previous_monday_after_year_change() {
        // Previous monday is 2021-12-27
        let time = Utc.ymd(2022, 1, 2).and_hms(15, 0, 0);
        let reset = last_raid_reset(time);
        assert_eq!(2021, reset.year());
        assert_eq!(12, reset.month());
        assert_eq!(27, reset.day());
        assert_eq!(7, reset.hour());
        assert_eq!(30, reset.minute());
        assert_eq!(0, reset.second());
    }

    #[test]
    fn last_raid_reset_is_previous_monday_on_monday_before_reset_after_year_change() {
        // Previous monday is 2021-12-27
        let time = Utc.ymd(2022, 1, 3).and_hms(0, 10, 0);
        let reset = last_raid_reset(time);
        assert_eq!(2021, reset.year());
        assert_eq!(12, reset.month());
        assert_eq!(27, reset.day());
        assert_eq!(7, reset.hour());
        assert_eq!(30, reset.minute());
        assert_eq!(0, reset.second());
    }
}