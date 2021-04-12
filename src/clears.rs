use crate::api::{Gw2Api, RaidClearState, RaidWings};

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