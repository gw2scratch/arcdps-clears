mod defaults;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Read, Write};
use std::sync::Mutex;
use log::error;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "defaults::friends_api_url")]
    pub friends_api_url: String,
    #[serde(default = "defaults::api_keys")]
    pub api_keys: Vec<ApiKey>,
    #[serde(default = "defaults::friends")]
    pub friend_list: FriendList,
    #[serde(default = "defaults::friend_default_show_state")]
    pub friend_default_show_state: bool,
    #[serde(default = "defaults::check_updates")]
    pub check_updates: bool,
    #[serde(default = "defaults::short_names")]
    pub short_names: bool,
    #[serde(default = "defaults::my_clears_style")]
    pub my_clears_style: ClearsStyle,
    #[serde(default = "defaults::friends_clears_style")]
    pub friends_clears_style: ClearsStyle,
    #[serde(default = "defaults::main_window_keybind")]
    pub main_window_keybind: Option<usize>,
    #[serde(default = "defaults::api_window_keybind")]
    pub api_window_keybind: Option<usize>,
    #[serde(default = "defaults::close_window_with_escape")]
    pub close_window_with_escape: bool,
    #[serde(default = "defaults::hide_in_loading_screens")]
    pub hide_in_loading_screens: bool,
    #[serde(default = "defaults::main_window_show_bg")]
    pub main_window_show_bg: bool,
    #[serde(default = "defaults::main_window_show_title")]
    pub main_window_show_title: bool,
    // Are you adding a new style option? Make sure to add to `reset_style()`!
}

#[derive(Serialize, Deserialize)]
pub struct ClearsStyle {
    pub table_style: ClearsTableStyle,
    pub account_header_style: AccountHeaderStyle,
    pub show_clears_table_headers: bool,
    pub show_clears_table_row_names: bool,
    pub finished_clear_color: [f32; 4],
    pub unfinished_clear_color: [f32; 4],
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum AccountHeaderStyle {
    None,
    CenteredText,
    Collapsible,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum ClearsTableStyle {
    WingColumns,
    WingRows,
    SingleRow,
}

#[derive(Serialize, Deserialize)]
pub struct ApiKey {
    #[serde(default = "new_api_key_id")]
    id: Uuid,
    key: String,
    data: ApiKeyData,
    #[serde(default = "defaults::show_key_in_clears")]
    show_key_in_clears: bool,
    #[serde(default = "defaults::expanded_in_clears")]
    expanded_in_clears: bool,
}

fn new_api_key_id() -> Uuid {
    Uuid::new_v4()
}

impl Hash for ApiKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[allow(dead_code)]
impl ApiKey {
    pub fn new(str: &str) -> Self {
        ApiKey {
            id: new_api_key_id(),
            key: str.to_string(),
            data: ApiKeyData::empty(),
            show_key_in_clears: defaults::show_key_in_clears(),
            expanded_in_clears: defaults::expanded_in_clears(),
        }
    }
    pub fn change_key(&mut self, str: &str) {
        self.key = str.to_string();
        self.data = ApiKeyData::empty();
    }
    pub fn id(&self) -> &Uuid {
        &self.id
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn data(&self) -> &ApiKeyData {
        &self.data
    }
    pub fn set_account_data(&mut self, account_data: Option<AccountData>) {
        self.data.account_data = account_data;
    }
    pub fn set_token_info(&mut self, token_info: Option<TokenInfo>) {
        self.data.token_info = token_info;
    }
    pub fn show_key_in_clears(&self) -> bool {
        self.show_key_in_clears
    }
    pub fn show_key_in_clears_mut(&mut self) -> &mut bool {
        &mut self.show_key_in_clears
    }
    pub fn expanded_in_clears(&self) -> bool {
        self.expanded_in_clears
    }
    pub fn expanded_in_clears_mut(&mut self) -> &mut bool {
        &mut self.expanded_in_clears
    }
}

#[derive(Serialize, Deserialize)]
pub struct AccountData {
    id: String,
    name: String,
    last_modified: DateTime<Utc>,
}

#[allow(dead_code)]
impl AccountData {
    pub fn new(id: String, name: String, last_modified: DateTime<Utc>) -> Self {
        AccountData { id, name, last_modified }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn last_modified(&self) -> DateTime<Utc> {
        self.last_modified
    }
}

#[derive(Serialize, Deserialize)]
pub enum TokenType {
    Unknown,
    ApiKey,
    Subtoken {
        expires_at: DateTime<Utc>,
        issued_at: DateTime<Utc>,
        urls: Option<Vec<String>>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct TokenInfo {
    id: String,
    name: String,
    permissions: Vec<String>,
    token_type: TokenType,
}

#[allow(dead_code)]
impl TokenInfo {
    pub fn new(id: String, name: String, permissions: Vec<String>, key_type: TokenType) -> Self {
        TokenInfo {
            id,
            name,
            permissions,
            token_type: key_type,
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn permissions(&self) -> &Vec<String> {
        &self.permissions
    }
    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }
    pub fn has_permission(&self, name: &str) -> bool {
        self.permissions.iter().any(|x| x == name)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ApiKeyData {
    account_data: Option<AccountData>,
    token_info: Option<TokenInfo>,
}

impl ApiKeyData {
    pub fn empty() -> Self {
        ApiKeyData {
            account_data: None,
            token_info: None,
        }
    }
    pub fn account_data(&self) -> &Option<AccountData> {
        &self.account_data
    }
    pub fn token_info(&self) -> &Option<TokenInfo> {
        &self.token_info
    }
}

#[derive(Serialize, Deserialize)]
pub struct Friend {
    account_name: String,
    show_in_friends: bool,
    expanded_in_friends: bool,
    public: bool,
}

#[allow(dead_code)]
impl Friend {
    pub fn new(account_name: String, show_in_friends: bool, public: bool) -> Self {
        Friend { account_name, show_in_friends, public, expanded_in_friends: true }
    }
    pub fn account_name(&self) -> &str {
        &self.account_name
    }
    pub fn show_in_friends(&self) -> bool {
        self.show_in_friends
    }
    pub fn show_in_friends_mut(&mut self) -> &mut bool {
        &mut self.show_in_friends
    }
    pub fn expanded_in_friends(&self) -> bool {
        self.expanded_in_friends
    }
    pub fn expanded_in_friends_mut(&mut self) -> &mut bool {
        &mut self.expanded_in_friends
    }
    pub fn public(&self) -> bool {
        self.public
    }
}

#[derive(Serialize, Deserialize)]
pub struct FriendList {
    friends: Vec<Friend>,
}

impl FriendList {
    pub fn get(&self, account_name: &str) -> Option<&Friend> {
        self.friends.iter().filter(|f| f.account_name() == account_name).next()
    }
    pub fn add(&mut self, friend: Friend) {
        self.friends.push(friend)
    }
    pub fn remove(&mut self, account_name: &str) {
        self.friends.retain(|x| x.account_name != account_name);
    }
    pub fn friends(&self) -> &Vec<Friend> {
        &self.friends
    }
    pub fn friends_mut(&mut self) -> &mut Vec<Friend> {
        &mut self.friends
    }
}

impl Settings {
    fn default() -> Self {
        Settings {
            friends_api_url: defaults::friends_api_url(),
            api_keys: defaults::api_keys(),
            friend_list: defaults::friends(),
            friend_default_show_state: defaults::friend_default_show_state(),
            check_updates: defaults::check_updates(),
            short_names: defaults::short_names(),
            my_clears_style: defaults::my_clears_style(),
            friends_clears_style: defaults::friends_clears_style(),
            main_window_keybind: defaults::main_window_keybind(),
            api_window_keybind: defaults::api_window_keybind(),
            close_window_with_escape: defaults::close_window_with_escape(),
            hide_in_loading_screens: defaults::hide_in_loading_screens(),
            main_window_show_bg: defaults::main_window_show_bg(),
            main_window_show_title: defaults::main_window_show_title(),
            // Are you adding a new style option? Make sure to add to `reset_style()`!
        }
    }

    pub fn reset_style(&mut self) {
        self.short_names = defaults::short_names();
        self.my_clears_style = defaults::my_clears_style();
        self.friends_clears_style = defaults::friends_clears_style();
        self.main_window_show_bg = defaults::main_window_show_bg();
        self.main_window_show_title = defaults::main_window_show_title();
    }

    pub fn api_keys(&self) -> &Vec<ApiKey> {
        &self.api_keys
    }

    pub fn get_key(&self, id: &Uuid) -> Option<&ApiKey> {
        self.api_keys.iter().filter(|x| x.id == *id).next()
    }

    pub fn get_key_mut(&mut self, id: &Uuid) -> Option<&mut ApiKey> {
        self.api_keys.iter_mut().filter(|x| x.id == *id).next()
    }

    pub fn remove_key(&mut self, id: &Uuid) {
        self.api_keys.retain(|x| x.id() != id)
    }

    #[allow(dead_code)]
    pub fn short_names(&self) -> bool {
        self.short_names
    }

    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(filename)?;
        let mut settings_json = String::new();
        file.read_to_string(&mut settings_json)?;

        // Try deserialization of settings from version 0.1.0 first
        if let Some(settings) = load_old_settings(&settings_json) {
            return Ok(settings);
        }

        let settings = serde_json::from_str(&settings_json)?;
        Ok(settings)
    }

    #[must_use]
    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        // We first serialize settings into a temporary file and then move the file
        let tmp_filename = format!("{}.tmp", filename);
        let tmp_file = File::create(&tmp_filename)?;
        let mut writer = BufWriter::new(tmp_file);
        serde_json::to_writer(&mut writer, &self)?;
        writer.flush()?;

        std::fs::rename(tmp_filename, filename)?;
        Ok(())
    }
}

pub fn load_bg(
    settings_mutex: &'static Mutex<Option<Settings>>,
    filename: &'static str,
    continue_with: Option<fn()>,
) {
    std::thread::spawn(move || {
        match Settings::load_from_file(filename) {
            Ok(settings) => {
                *settings_mutex.lock().unwrap() = Some(settings);
            }
            Err(e) => {
                error!("Failed to read settings; resetting: {}", e);
                *settings_mutex.lock().unwrap() = Some(Settings::default());
            }
        }

        if let Some(function) = continue_with {
            function();
        }
    });
}


fn load_old_settings(json: &str) -> Option<Settings> {
    /// Settings from version 0.1.0
    #[derive(Serialize, Deserialize)]
    struct OldApiKey {
        key: String,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct OldSettings {
        main_api_key: Option<OldApiKey>,
        short_names: bool,
    }

    let old_setting_result: serde_json::Result<OldSettings> = serde_json::from_str(&json);
    if let Ok(old_settings) = old_setting_result {
        let mut settings = Settings::default();
        settings.short_names = old_settings.short_names;
        if let Some(main_key) = old_settings.main_api_key {
            settings.api_keys.push(ApiKey::new(&main_key.key))
        }

        return Some(settings);
    }
    None
}