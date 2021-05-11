use itertools::Itertools;
use serde::{Deserialize, Serialize};
use ureq::{Error, Response};

use crate::clears::{EncounterType, RaidClearState, RaidEncounter, RaidWing, RaidWings};

const LIVE_GW2_API_URL: &str = "https://api.guildwars2.com/";

pub enum ApiError {
    UnknownError,
    InvalidKey,
    JsonDeserializationFailed,
    TooManyRequests,
}

fn parse_raids(json: &str) -> Result<RaidWings, serde_json::Error> {
    // We do not care about individual raids, we extract wings from them and only work with wings.
    // For this reason, we only define this struct locally.
    #[derive(Serialize, Deserialize)]
    struct Raid {
        id: String,
        wings: Vec<RaidWing>,
    }
    let raids: Vec<Raid> = serde_json::from_str(json)?;
    Ok(RaidWings::new(raids.into_iter().flat_map(|x| x.wings).collect()))
}

fn parse_clears(json: &str) -> Result<RaidClearState, serde_json::Error> {
    let cleared_ids: Vec<String> = serde_json::from_str(json)?;
    Ok(RaidClearState::new(cleared_ids))
}

pub trait Gw2Api {
    fn get_raids(&self) -> Result<RaidWings, ApiError>;
    fn get_raids_state(&self, api_key: &str) -> Result<RaidClearState, ApiError>;
}

pub struct LiveApi {
    url: String
}

impl LiveApi {
    pub fn new(url: String) -> Self {
        LiveApi { url }
    }

    pub fn official() -> Self {
        Self::new(LIVE_GW2_API_URL.to_string())
    }
}

impl Gw2Api for LiveApi {
    fn get_raids(&self) -> Result<RaidWings, ApiError> {
        eprintln!("get_raids");
        // TODO: user agent once we have a name (call header)
        match ureq::get(&format!("{}v2/raids?ids=all", self.url)).call()
        {
            Ok(response) => {
                if let Ok(text) = response.into_string() {
                    if let Ok(raids) = parse_raids(&text) {
                        Ok(raids)
                    } else {
                        Err(ApiError::JsonDeserializationFailed)
                    }
                } else {
                    Err(ApiError::UnknownError)
                }
            }
            Err(_) => {
                Err(ApiError::UnknownError)
            }
        }
    }

    fn get_raids_state(&self, api_key: &str) -> Result<RaidClearState, ApiError> {
        // TODO: Check last update in account data
        match ureq::get(&format!("{}v2/account/raids", self.url))
            .set("Authorization", &format!("Bearer {}", api_key))
            .call() {
            Ok(response) => {
                if let Ok(text) = response.into_string() {
                    if let Ok(clears) = parse_clears(&text) {
                        Ok(clears)
                    } else {
                        Err(ApiError::JsonDeserializationFailed)
                    }
                } else {
                    // TODO: Implement
                    Err(ApiError::UnknownError)
                }
            }
            Err(err) => {
                match err {
                    Error::Status(code, _response) => {
                        if code == 401 {
                            Err(ApiError::InvalidKey)
                        } else if code == 429 {
                            Err(ApiError::TooManyRequests)
                        } else {
                            // TODO: Add logging
                            Err(ApiError::UnknownError)
                        }
                    }
                    // TODO: Logging
                    _ => Err(ApiError::UnknownError)
                }
            }
        }
    }
}

pub struct ApiMock {}

impl ApiMock {
    pub fn new() -> Self {
        ApiMock {}
    }
}

impl Gw2Api for ApiMock {
    fn get_raids(&self) -> Result<RaidWings, ApiError> {
        Ok(RaidWings::new(vec![
            RaidWing::new("spirit_vale".to_string(), vec![
                RaidEncounter::new("vale_guardian".to_string(), EncounterType::Boss),
                RaidEncounter::new("spirit_woods".to_string(), EncounterType::Checkpoint),
                RaidEncounter::new("gorseval".to_string(), EncounterType::Boss),
                RaidEncounter::new("sabetha".to_string(), EncounterType::Boss),
            ]),
            RaidWing::new("salvation_pass".to_string(), vec![
                RaidEncounter::new("slothasor".to_string(), EncounterType::Boss),
                RaidEncounter::new("bandit_trio".to_string(), EncounterType::Boss),
                RaidEncounter::new("matthias".to_string(), EncounterType::Boss),
            ]),
        ]))
    }

    fn get_raids_state(&self, _: &str) -> Result<RaidClearState, ApiError> {
        Ok(RaidClearState::new(vec!["gorseval".to_string(), "slothasor".to_string(), "bandit_trio".to_string()]))
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{parse_clears, parse_raids};
    use crate::clears::{EncounterType, RaidEncounter};

    #[test]
    fn raids_parsed_correctly() {
        let api_data_json = r#"[
  {
    "id": "forsaken_thicket",
    "wings": [
      {
        "id": "spirit_vale",
        "events": [
          {
            "id": "vale_guardian",
            "type": "Boss"
          },
          {
            "id": "spirit_woods",
            "type": "Checkpoint"
          },
          {
            "id": "gorseval",
            "type": "Boss"
          },
          {
            "id": "sabetha",
            "type": "Boss"
          }
        ]
      },
      {
        "id": "salvation_pass",
        "events": [
          {
            "id": "slothasor",
            "type": "Boss"
          },
          {
            "id": "bandit_trio",
            "type": "Boss"
          },
          {
            "id": "matthias",
            "type": "Boss"
          }
        ]
      },
      {
        "id": "stronghold_of_the_faithful",
        "events": [
          {
            "id": "escort",
            "type": "Boss"
          },
          {
            "id": "keep_construct",
            "type": "Boss"
          },
          {
            "id": "twisted_castle",
            "type": "Checkpoint"
          },
          {
            "id": "xera",
            "type": "Boss"
          }
        ]
      }
    ]
  },
  {
    "id": "bastion_of_the_penitent",
    "wings": [
      {
        "id": "bastion_of_the_penitent",
        "events": [
          {
            "id": "cairn",
            "type": "Boss"
          },
          {
            "id": "mursaat_overseer",
            "type": "Boss"
          },
          {
            "id": "samarog",
            "type": "Boss"
          },
          {
            "id": "deimos",
            "type": "Boss"
          }
        ]
      }
    ]
  },
  {
    "id": "hall_of_chains",
    "wings": [
      {
        "id": "hall_of_chains",
        "events": [
          {
            "id": "soulless_horror",
            "type": "Boss"
          },
          {
            "id": "river_of_souls",
            "type": "Boss"
          },
          {
            "id": "statues_of_grenth",
            "type": "Boss"
          },
          {
            "id": "voice_in_the_void",
            "type": "Boss"
          }
        ]
      }
    ]
  },
  {
    "id": "mythwright_gambit",
    "wings": [
      {
        "id": "mythwright_gambit",
        "events": [
          {
            "id": "conjured_amalgamate",
            "type": "Boss"
          },
          {
            "id": "twin_largos",
            "type": "Boss"
          },
          {
            "id": "qadim",
            "type": "Boss"
          }
        ]
      }
    ]
  },
  {
    "id": "the_key_of_ahdashim",
    "wings": [
      {
        "id": "the_key_of_ahdashim",
        "events": [
          {
            "id": "gate",
            "type": "Checkpoint"
          },
          {
            "id": "adina",
            "type": "Boss"
          },
          {
            "id": "sabir",
            "type": "Boss"
          },
          {
            "id": "qadim_the_peerless",
            "type": "Boss"
          }
        ]
      }
    ]
  }
]
"#;
        let parsed = parse_raids(&api_data_json).expect("Failed to deserialize api data json.");
        assert_eq!(parsed.wings().len(), 7);
        assert_eq!(parsed.wings()[0].id(), "spirit_vale");
        assert_eq!(parsed.wings()[0].encounters()[0].id(), "vale_guardian");
        assert_eq!(parsed.wings()[0].encounters()[0].encounter_type(), EncounterType::Boss);
        assert_eq!(parsed.wings()[0].encounters()[1].id(), "spirit_woods");
        assert_eq!(parsed.wings()[0].encounters()[1].encounter_type(), EncounterType::Checkpoint);
        assert_eq!(parsed.wings()[3].id(), "bastion_of_the_penitent");
        assert_eq!(parsed.wings()[3].encounters().len(), 4);
    }

    #[test]
    fn clears_parsed_correctly() {
        let api_response_json = r#"[
    "gorseval",
    "bandit_trio",
    "slothasor",
    "sabetha",
    "matthias",
    "xera",
    "samarog",
    "deimos",
    "mursaat_overseer",
    "cairn",
    "voice_in_the_void",
    "soulless_horror",
    "conjured_amalgamate",
    "adina",
    "sabir"
]
"#;
        let parsed = parse_clears(&api_response_json).expect("Failed to deserialize api data json.");
        assert_eq!(parsed.finished_encounter_ids().len(), 15);
        assert!(parsed.is_finished(&RaidEncounter::new("gorseval".to_string(), EncounterType::Boss)));
        assert!(parsed.is_finished(&RaidEncounter::new("adina".to_string(), EncounterType::Boss)));
        assert!(!parsed.is_finished(&RaidEncounter::new("vale_guardian".to_string(), EncounterType::Boss)));
    }
}