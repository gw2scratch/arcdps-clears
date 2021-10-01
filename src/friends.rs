use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha2::{Sha256, Digest};
use std::fmt::Write;
use crate::settings::{ApiKey, TokenType};
use itertools::Itertools;
use std::collections::HashMap;
use crate::clears::{ClearData, RaidClearState};

const USER_AGENT: &str = concat!("arcdps-clears/", env!("CARGO_PKG_VERSION"));

pub const SUBTOKEN_URLS: [&str; 8] = [
    "/v2/tokeninfo",
    "/v2/createsubtoken",
    "/v2/account",
    "/v2/account/achievements",
    "/v2/account/dungeons",
    "/v2/account/worldbosses",
    "/v2/account/masteries",
    "/v2/account/raids",
];

pub const SUBTOKEN_PERMISSIONS: [&str; 2] = [
    "account",
    "progression"
];


#[derive(Deserialize)]
pub struct State {
    keys: Vec<KeyState>,
    friends: Vec<FriendState>,
}

#[derive(Deserialize)]
pub struct ShareState {
    account: String,
    added_at: DateTime<Utc>,
    account_available: bool,
}

#[derive(Deserialize)]
pub struct KeyState {
    key_hash: String,
    shared_to: Vec<ShareState>,
    subtoken_added_at: Option<DateTime<Utc>>,
    subtoken_expires_at: Option<DateTime<Utc>>,
    account: Option<String>,
}

#[derive(Deserialize)]
pub struct FriendState {
    account: String,
    subtoken: String,
    expires_at: DateTime<Utc>,
    shared_with: Vec<String>,
}

impl State {
    pub fn keys(&self) -> &Vec<KeyState> {
        &self.keys
    }
    pub fn friends(&self) -> &Vec<FriendState> {
        &self.friends
    }
}

impl ShareState {
    pub fn account(&self) -> &str {
        &self.account
    }
    pub fn added_at(&self) -> DateTime<Utc> {
        self.added_at
    }
    pub fn account_available(&self) -> bool {
        self.account_available
    }
}

impl KeyState {
    pub fn key_hash(&self) -> &str {
        &self.key_hash
    }
    pub fn shared_to(&self) -> &Vec<ShareState> {
        &self.shared_to
    }
    pub fn subtoken_added_at(&self) -> Option<DateTime<Utc>> {
        self.subtoken_added_at
    }
    pub fn subtoken_expires_at(&self) -> Option<DateTime<Utc>> {
        self.subtoken_expires_at
    }
    pub fn account(&self) -> &Option<String> {
        &self.account
    }
}

impl FriendState {
    pub fn account(&self) -> &str {
        &self.account
    }
    pub fn subtoken(&self) -> &str {
        &self.subtoken
    }
    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
    pub fn shared_with(&self) -> &Vec<String> {
        &self.shared_with
    }
}

pub struct FriendData {
    api_state: Option<State>,
    subtokens_by_account: HashMap<String, String>,
    clears_by_account: HashMap<String, RaidClearState>,
}

impl FriendData {
    pub fn new() -> Self {
        FriendData { api_state: None, clears_by_account: HashMap::new(), subtokens_by_account: HashMap::new() }
    }
    pub fn api_state(&self) -> &Option<State> {
        &self.api_state
    }
    pub fn clears_by_account(&self) -> &HashMap<String, RaidClearState> {
        &self.clears_by_account
    }
    pub fn set_api_state(&mut self, api_state: Option<State>) {
        self.api_state = api_state;
    }
    pub fn set_clears(&mut self, account: String, clear_data: RaidClearState) {
        self.clears_by_account.insert(account, clear_data);
    }
    pub fn set_subtoken(&mut self, account: String, subtoken: String) {
        self.subtokens_by_account.insert(account, subtoken);
    }
    pub fn get_subtoken(&self, account: &str) -> Option<&String> {
        self.subtokens_by_account.get(account)
    }
}

fn parse_state(json: &str) -> Result<State, serde_json::Error> {
    Ok(serde_json::from_str(json)?)
}

pub enum FriendsApiError {
    JsonDeserializationFailed(serde_json::Error),
    UreqError(ureq::Error),
    UnknownError,
}

impl From<serde_json::Error> for FriendsApiError {
    fn from(e: serde_json::Error) -> Self {
        FriendsApiError::JsonDeserializationFailed(e)
    }
}

impl From<ureq::Error> for FriendsApiError {
    fn from(e: ureq::Error) -> Self {
        FriendsApiError::UreqError(e)
    }
}

pub struct FriendsApiClient {
    url: String,
}

impl FriendsApiClient {
    pub fn new(url: String) -> Self {
        FriendsApiClient { url }
    }

    pub fn get_state(&self, api_keys: Vec<String>) -> Result<State, FriendsApiError> {
        let response = ureq::get(&format!("{}state", self.url))
            .set("User-Agent", USER_AGENT)
            .set("x-auth-keys", &Self::auth_keys(&api_keys))
            .call()?;

        if let Ok(text) = response.into_string() {
            Ok(parse_state(&text)?)
        } else {
            Err(FriendsApiError::UnknownError)
        }
    }

    pub fn add_subtoken(&self, api_keys: Vec<String>, api_key: &str, subtoken: String) -> Result<State, FriendsApiError> {
        let response = ureq::post(&format!("{}key/add", self.url))
            .set("User-Agent", USER_AGENT)
            .set("x-auth-keys", &Self::auth_keys(&api_keys))
            .send_form(&[
                ("key_hash", &key_hash(api_key)),
                ("subtoken", &subtoken)
            ])?;

        if let Ok(text) = response.into_string() {
            Ok(parse_state(&text)?)
        } else {
            Err(FriendsApiError::UnknownError)
        }
    }

    pub fn share(&self, api_keys: Vec<String>, api_key: &ApiKey, friend_account: String) -> Result<State, FriendsApiError> {
        let response = ureq::post(&format!("{}key/share", self.url))
            .set("User-Agent", USER_AGENT)
            .set("x-auth-keys", &Self::auth_keys(&api_keys))
            .send_form(&[
                ("key_hash", &key_hash(api_key.key())),
                ("account", &friend_account)
            ])?;

        if let Ok(text) = response.into_string() {
            Ok(parse_state(&text)?)
        } else {
            Err(FriendsApiError::UnknownError)
        }
    }

    pub fn unshare(&self, api_keys: Vec<String>, api_key: &ApiKey, friend_account: String) -> Result<State, FriendsApiError> {
        let response = ureq::post(&format!("{}key/unshare", self.url))
            .set("User-Agent", USER_AGENT)
            .set("x-auth-keys", &Self::auth_keys(&api_keys))
            .send_form(&[
                ("key_hash", &key_hash(api_key.key())),
                ("account", &friend_account)
            ])?;

        if let Ok(text) = response.into_string() {
            Ok(parse_state(&text)?)
        } else {
            Err(FriendsApiError::UnknownError)
        }
    }

    fn auth_keys(api_keys: &[String]) -> String {
        api_keys.iter().map(|key| key_hash(key)).join(",")
    }
}

pub fn key_hash(api_key: &str) -> String {
    let mut hash = String::new();
    write!(hash, "{:x}", Sha256::digest(api_key.as_bytes())).unwrap();
    hash
}

pub enum KeyUsability {
    NoTokenInfo,
    Usable,
    InsufficientPermissions,
    InsufficientSubtokenUrls,
    SubtokenExpired,
}

impl KeyUsability {
    pub fn is_usable(&self) -> bool {
        match self {
            KeyUsability::Usable => true,
            _ => false
        }
    }
}

pub fn get_key_usability(key: &ApiKey) -> KeyUsability {
    if let Some(info) = key.data().token_info() {
        if !SUBTOKEN_PERMISSIONS.iter().all(|permission| info.has_permission(permission)) {
            return KeyUsability::InsufficientPermissions;
        }

        match info.token_type() {
            TokenType::Unknown => KeyUsability::NoTokenInfo,
            TokenType::ApiKey => {
                KeyUsability::Usable
            }
            TokenType::Subtoken { expires_at, urls, .. } => {
                if let Some(urls) = urls {
                    if !SUBTOKEN_URLS.iter().all(|url| urls.iter().any(|x| x == url)) {
                        return KeyUsability::InsufficientSubtokenUrls;
                    }
                    if *expires_at < Utc::now() {
                        return KeyUsability::SubtokenExpired;
                    }
                }
                KeyUsability::Usable
            }
        }
    } else {
        KeyUsability::NoTokenInfo
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_no_subtokens_parsed_correctly() {
        // This is the "before adding subtokens" sample from the API docs
        let json = r#"{
  "keys": [
    {
      "key_hash": "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b",
      "shared_to": [],
      "subtoken_added_at": null,
      "subtoken_expires_at": null,
      "account": null
    },
    {
      "key_hash": "4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865",
      "shared_to": [],
      "subtoken_added_at": null,
      "subtoken_expires_at": null,
      "account": null
    }
  ],
  "friends": []
}"#;
        let parsed: State = parse_state(&json).expect("Failed to deserialize api data json.");
        assert_eq!(parsed.keys.len(), 2);
        assert_eq!(parsed.keys[0].key_hash, "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b");
        assert_eq!(parsed.keys[1].key_hash, "4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865");
        assert!(parsed.keys[0].shared_to.is_empty());
        assert!(parsed.keys[1].shared_to.is_empty());
        assert!(parsed.keys[0].subtoken_added_at.is_none());
        assert!(parsed.keys[1].subtoken_added_at.is_none());
        assert!(parsed.keys[0].subtoken_expires_at.is_none());
        assert!(parsed.keys[1].subtoken_expires_at.is_none());
        assert!(parsed.keys[0].account.is_none());
        assert!(parsed.keys[1].account.is_none());
        assert!(parsed.friends.is_empty());
    }

    #[test]
    fn state_with_subtokens_and_shared_parsed_correctly() {
        // This is the "with subtokens and shared" sample from the API docs
        let json = r#"{
  "keys": [
    {
      "key_hash": "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b",
      "shared_to": [
        {
          "account": "Account.1234",
          "added_at": "2020-04-28T16:29:04.644008111Z",
          "account_available": true
        },
        {
          "account": "Account.5678",
          "added_at": "2020-04-29T16:29:04.644008111Z",
          "account_available": false
        }
      ],
      "subtoken_added_at": "2020-03-28T16:29:04.644008111Z",
      "subtoken_expires_at": "2021-03-28T16:29:04.644008111Z",
      "account": "OurAccount.1234"
    },
    {
      "key_hash": "4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865",
      "shared_to": [],
      "subtoken_added_at": "2020-03-28T16:30:04.644008111Z",
      "subtoken_expires_at": "2021-03-28T16:30:04.644008111Z",
      "account": "OurAccount.5678"
    }
  ],
  "friends": []
}"#;
        let parsed: State = parse_state(&json).expect("Failed to deserialize api data json.");
        assert_eq!(parsed.keys.len(), 2);
        assert_eq!(parsed.keys[0].key_hash, "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b");
        assert_eq!(parsed.keys[1].key_hash, "4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865");

        assert_eq!(parsed.keys[0].shared_to.len(), 2);
        assert_eq!(parsed.keys[0].shared_to[0].account, "Account.1234");
        assert_eq!(parsed.keys[0].shared_to[1].account, "Account.5678");
        assert_eq!(parsed.keys[0].shared_to[0].account_available, true);
        assert_eq!(parsed.keys[0].shared_to[1].account_available, false);
        assert!(parsed.keys[1].shared_to.is_empty());

        assert!(parsed.keys[0].subtoken_added_at.is_some());
        assert!(parsed.keys[1].subtoken_added_at.is_some());

        assert!(parsed.keys[0].subtoken_expires_at.is_some());
        assert!(parsed.keys[1].subtoken_expires_at.is_some());

        assert_eq!(parsed.keys[0].account, Some("OurAccount.1234".to_string()));
        assert_eq!(parsed.keys[1].account, Some("OurAccount.5678".to_string()));

        assert!(parsed.friends.is_empty());
    }

    #[test]
    fn state_with_friends_parsed_correctly() {
        // This is the "with friends" sample from the API docs
        let json = r#"{
  "keys": [
    {
      "key_hash": "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b",
      "shared_to": [],
      "subtoken_added_at": "2020-03-28T16:29:04.644008111Z",
      "subtoken_expires_at": "2021-03-28T16:29:04.644008111Z",
      "account": "OurAccount.1234"
    },
    {
      "key_hash": "4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865",
      "shared_to": [],
      "subtoken_added_at": "2020-03-28T16:30:04.644008111Z",
      "subtoken_expires_at": "2021-03-28T16:30:04.644008111Z",
      "account": "OurAccount.5678"
    }
  ],
  "friends": [
    {
      "account": "Friend.1234",
      "subtoken": "long.jwt.token.here",
      "expires_at": "2021-04-28T17:25:00.181828132Z",
      "shared_with": [
        "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b"
      ]
    },
    {
      "account": "Friend.5678",
      "subtoken": "another.long.jwt.token.here",
      "expires_at": "2021-04-28T17:45:00.000000000Z",
      "shared_with": [
        "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b",
        "4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865"
      ]
    }
  ]
}"#;
        let parsed: State = parse_state(&json).expect("Failed to deserialize api data json.");
        assert_eq!(parsed.keys.len(), 2);
        assert_eq!(parsed.keys[0].key_hash, "01ba4719c80b6fe911b091a7c05124b64eeece964e09c058ef8f9805daca546b");
        assert_eq!(parsed.keys[1].key_hash, "4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865");

        assert_eq!(parsed.friends.len(), 2);
        assert_eq!(parsed.friends[0].account, "Friend.1234");
        assert_eq!(parsed.friends[1].account, "Friend.5678");
        assert_eq!(parsed.friends[0].subtoken, "long.jwt.token.here");
        assert_eq!(parsed.friends[1].subtoken, "another.long.jwt.token.here");
        assert_eq!(parsed.friends[0].shared_with.len(), 1);
        assert_eq!(parsed.friends[1].shared_with.len(), 2);
    }

    #[test]
    fn hash_is_sha256() {
        let api_key = "EDBBF0DE-1234-5678-8E7A-00000000000091B33521-6816-D711-70C3-ADB1D78A5C72";
        let hash = key_hash(&api_key);
        assert_eq!(hash, "27e6da1e6e2a277cbaf23df8213159a9862f6b4d0f6b82d72652a672e01d76f4");
    }
}