use itertools::Itertools;
use serde::{Deserialize, Serialize};
use ureq::{Error, Response};

use crate::clears::{EncounterType, RaidClearState, RaidEncounter, RaidWing, RaidWings};
use chrono::{DateTime, Utc, TimeZone};
use crate::settings::{ApiKeyData, AccountData, TokenInfo, TokenType};

const USER_AGENT: &str = concat!("arcdps-clears v", env!("CARGO_PKG_VERSION"));
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
    Ok(RaidWings::new(
        raids.into_iter().flat_map(|x| x.wings).collect(),
    ))
}

fn parse_clears(json: &str) -> Result<RaidClearState, serde_json::Error> {
    let cleared_ids: Vec<String> = serde_json::from_str(json)?;
    Ok(RaidClearState::new(cleared_ids))
}

fn parse_account_data(json: &str) -> Result<AccountData, serde_json::Error> {
    let data: AccountData = serde_json::from_str(json)?;
    Ok(data)
}

fn parse_token_info(json: &str) -> Result<TokenInfo, serde_json::Error> {
    #[derive(Serialize, Deserialize)]
    struct TokenInfoResponse {
        id: String,
        name: String,
        permissions: Vec<String>,
        #[serde(rename = "type")]
        token_type: String,
        expires_at: Option<DateTime<Utc>>,
        issued_at: Option<DateTime<Utc>>,
        urls: Option<Vec<String>>,
    }
    let response: TokenInfoResponse = serde_json::from_str(json)?;
    let info = TokenInfo::new(
        response.id,
        response.name,
        response.permissions,
        match response.token_type.as_str() {
            "APIKey" => TokenType::ApiKey,
            "Subtoken" => {
                if response.expires_at.is_none() || response.issued_at.is_none() {
                    // Should never be missing unless something changes in the future.
                    TokenType::Unknown
                } else {
                    TokenType::Subtoken {
                        expires_at: response.expires_at.unwrap(),
                        issued_at: response.issued_at.unwrap(),
                        urls: response.urls
                    }
                }
            }
            _ => TokenType::Unknown
        }
    );

    Ok(info)
}

pub trait Gw2Api {
    fn get_raids(&self) -> Result<RaidWings, ApiError>;
    fn get_raids_state(&self, api_key: &str) -> Result<RaidClearState, ApiError>;
    fn get_account_data(&self, api_key: &str) -> Result<AccountData, ApiError>;
    fn get_token_info(&self, api_key: &str) -> Result<TokenInfo, ApiError>;
}

pub struct LiveApi {
    url: String,
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
        match ureq::get(&format!("{}v2/raids?ids=all", self.url))
            .set("User-Agent", USER_AGENT)
            .call()
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
            Err(_) => Err(ApiError::UnknownError),
        }
    }

    fn get_raids_state(&self, api_key: &str) -> Result<RaidClearState, ApiError> {
        // TODO: Check last update in account data
        match ureq::get(&format!("{}v2/account/raids", self.url))
            .set("User-Agent", USER_AGENT)
            .set("Authorization", &format!("Bearer {}", api_key))
            .call()
        {
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
                    _ => Err(ApiError::UnknownError),
                }
            }
        }
    }

    fn get_account_data(&self, api_key: &str) -> Result<AccountData, ApiError> {
        match ureq::get(&format!("{}v2/account", self.url))
            .set("User-Agent", USER_AGENT)
            .set("X-Schema-Version", "2021-05-20T00:00:00.000Z")
            .set("Authorization", &format!("Bearer {}", api_key))
            .call()
        {
            Ok(response) => {
                if let Ok(text) = response.into_string() {
                    if let Ok(data) = parse_account_data(&text) {
                        Ok(data)
                    } else {
                        Err(ApiError::JsonDeserializationFailed)
                    }
                } else {
                    Err(ApiError::UnknownError)
                }
            }
            Err(_) => Err(ApiError::UnknownError),
        }
    }

    fn get_token_info(&self, api_key: &str) -> Result<TokenInfo, ApiError> {
        match ureq::get(&format!("{}v2/tokeninfo", self.url))
            .set("User-Agent", USER_AGENT)
            .set("X-Schema-Version", "2021-05-20T00:00:00.000Z")
            .set("Authorization", &format!("Bearer {}", api_key))
            .call()
        {
            Ok(response) => {
                if let Ok(text) = response.into_string() {
                    if let Ok(data) = parse_token_info(&text) {
                        Ok(data)
                    } else {
                        Err(ApiError::JsonDeserializationFailed)
                    }
                } else {
                    Err(ApiError::UnknownError)
                }
            }
            Err(_) => Err(ApiError::UnknownError),
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
            RaidWing::new(
                "spirit_vale".to_string(),
                vec![
                    RaidEncounter::new("vale_guardian".to_string(), EncounterType::Boss),
                    RaidEncounter::new("spirit_woods".to_string(), EncounterType::Checkpoint),
                    RaidEncounter::new("gorseval".to_string(), EncounterType::Boss),
                    RaidEncounter::new("sabetha".to_string(), EncounterType::Boss),
                ],
            ),
            RaidWing::new(
                "salvation_pass".to_string(),
                vec![
                    RaidEncounter::new("slothasor".to_string(), EncounterType::Boss),
                    RaidEncounter::new("bandit_trio".to_string(), EncounterType::Boss),
                    RaidEncounter::new("matthias".to_string(), EncounterType::Boss),
                ],
            ),
        ]))
    }

    fn get_raids_state(&self, _: &str) -> Result<RaidClearState, ApiError> {
        Ok(RaidClearState::new(vec![
            "gorseval".to_string(),
            "slothasor".to_string(),
            "bandit_trio".to_string(),
        ]))
    }

    fn get_account_data(&self, _: &str) -> Result<AccountData, ApiError> {
        Ok(AccountData::new(
            "91B33521-1234-5678-9ABCD-ADB1D78A5C72".to_string(),
            "TestName.4321".to_string(),
            Utc.ymd(2021, 5, 21).and_hms(8, 35, 0)
        ))
    }

    fn get_token_info(&self, api_key: &str) -> Result<TokenInfo, ApiError> {
        Ok(TokenInfo::new(
            api_key.to_string(),
            "mock".to_string(),
            vec!["account".to_string(), "progression".to_string()],
            TokenType::ApiKey
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{parse_account_data, parse_clears, parse_raids, parse_token_info};
    use crate::clears::{EncounterType, RaidEncounter};
    use chrono::{Utc, TimeZone, DateTime};
    use crate::settings::TokenType;

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
        assert_eq!(
            *parsed.wings()[0].encounters()[0].encounter_type(),
            EncounterType::Boss
        );
        assert_eq!(parsed.wings()[0].encounters()[1].id(), "spirit_woods");
        assert_eq!(
            *parsed.wings()[0].encounters()[1].encounter_type(),
            EncounterType::Checkpoint
        );
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
        let parsed =
            parse_clears(&api_response_json).expect("Failed to deserialize api data json.");
        assert_eq!(parsed.finished_encounter_ids().len(), 15);
        assert!(parsed.is_finished(&RaidEncounter::new(
            "gorseval".to_string(),
            EncounterType::Boss
        )));
        assert!(parsed.is_finished(&RaidEncounter::new(
            "adina".to_string(),
            EncounterType::Boss
        )));
        assert!(!parsed.is_finished(&RaidEncounter::new(
            "vale_guardian".to_string(),
            EncounterType::Boss
        )));
    }

    #[test]
    fn account_parsed_correctly() {
        let api_response_json = r#"{
  "id": "91B33521-6816-D711-70C3-ADB1D78A5C72",
  "name": "Name.1234",
  "age": 16853340,
  "last_modified": "2021-05-21T08:35:00Z",
  "world": 2006,
  "guilds": [
    "01D1DADF-751E-E411-ADEE-AC162DC0070D"
  ],
  "created": "2017-03-11T14:37:00Z",
  "access": [
    "GuildWars2",
    "HeartOfThorns",
    "PlayForFree",
    "PathOfFire"
  ],
  "commander": true,
  "fractal_level": 100,
  "daily_ap": 3500,
  "monthly_ap": 0,
  "wvw_rank": 87
}
"#;
        let parsed = parse_account_data(&api_response_json).expect("Failed to deserialize api data json.");
        assert_eq!(parsed.id(), "91B33521-6816-D711-70C3-ADB1D78A5C72");
        assert_eq!(parsed.name(), "Name.1234");
        assert_eq!(parsed.last_modified(), Utc.ymd(2021, 5, 21).and_hms(8, 35, 0));
    }

    #[test]
    fn token_info_parsed_correctly_subtoken() {
        let api_response_json = r#"{
  "id": "EDBBF0DE-1234-5678-8E7A-000000000000",
  "name": "Clears",
  "permissions": [
    "account",
    "progression"
  ],
  "type": "Subtoken",
  "expires_at": "2021-06-20T14:34:47.000Z",
  "issued_at": "2021-05-21T14:34:47.000Z"
}
"#;
        let parsed = parse_token_info(&api_response_json).expect("Failed to deserialize api data json.");
        assert_eq!(parsed.id(), "EDBBF0DE-1234-5678-8E7A-000000000000");
        assert_eq!(parsed.name(), "Clears");
        assert_eq!(parsed.permissions().len(), 2);
        assert_eq!(parsed.permissions()[0], "account");
        assert_eq!(parsed.permissions()[1], "progression");
        let expiration: DateTime<Utc> = Utc.ymd(2021, 6, 20).and_hms(14, 34, 47);
        let issued: DateTime<Utc> = Utc.ymd(2021, 5, 21).and_hms(14, 34, 47);
        assert!(match parsed.token_type() {
            TokenType::Subtoken { expires_at: expiration, issued_at: issued, urls: None } => true,
            _ => false
        })
    }

    #[test]
    fn token_info_parsed_correctly() {
        let api_response_json = r#"{
  "id": "EDBBF0DE-1234-5678-8E7A-000000000000",
  "name": "Clears",
  "permissions": [
    "account",
    "progression"
  ],
  "type": "APIKey"
}
"#;
        let parsed = parse_token_info(&api_response_json).expect("Failed to deserialize api data json.");
        assert_eq!(parsed.id(), "EDBBF0DE-1234-5678-8E7A-000000000000");
        assert_eq!(parsed.name(), "Clears");
        assert_eq!(parsed.permissions().len(), 2);
        assert_eq!(parsed.permissions()[0], "account");
        assert_eq!(parsed.permissions()[1], "progression");
        assert!(match parsed.token_type() {
            TokenType::ApiKey => true,
            _ => false
        })
    }
}
