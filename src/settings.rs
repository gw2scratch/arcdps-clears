use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter, Write};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_api_keys")]
    pub api_keys: Vec<ApiKey>,
    #[serde(default = "default_short_name")]
    pub short_names: bool,
    #[serde(default = "default_check_updates")]
    pub check_updates: bool,
    #[serde(default = "default_finished_clear_color")]
    pub finished_clear_color: [f32; 4],
    #[serde(default = "default_unfinished_clear_color")]
    pub unfinished_clear_color: [f32; 4],
    #[serde(default = "default_clears_style")]
    pub clears_style: ClearsStyle,
    #[serde(default = "default_account_header")]
    pub account_header_style: AccountHeaderStyle,
    #[serde(default = "default_main_window_keybind")]
    pub main_window_keybind: Option<usize>,
    #[serde(default = "default_api_window_keybind")]
    pub api_window_keybind: Option<usize>,
    #[serde(default = "default_close_window_with_escape")]
    pub close_window_with_escape: bool
}

fn default_short_name() -> bool {
    true
}
fn default_check_updates() -> bool {
    true
}
fn default_api_keys() -> Vec<ApiKey> {
    Vec::new()
}
fn default_finished_clear_color() -> [f32; 4] {
    [38. / 255., 199. / 255., 29. / 255., 177. / 255.]
}
fn default_unfinished_clear_color() -> [f32; 4] {
    [192. / 255., 24. / 255., 30. / 255., 0.]
}
fn default_clears_style() -> ClearsStyle {
    ClearsStyle::WingRows
}
fn default_account_header() -> AccountHeaderStyle {
    AccountHeaderStyle::CenteredText
}
fn default_api_window_keybind() -> Option<usize> {
    None
}
fn default_main_window_keybind() -> Option<usize> {
    // C (for clears)
    // Note that arcdps uses this as a default keybind, but it seems to be an uncommonly used one.
    // As we eat the input, there should be no issue and the behavior should be obvious. The user
    // can change the conflicting keybind in our plugin or in arcdps itself if needed.
    Some(67)
}
fn default_close_window_with_escape() -> bool {
    true
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum AccountHeaderStyle {
    None,
    CenteredText,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum ClearsStyle {
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
    #[serde(default = "default_show_key_in_clears")]
    show_key_in_clears: bool,
}

fn new_api_key_id() -> Uuid {
    Uuid::new_v4()
}

fn default_show_key_in_clears() -> bool {
    true
}

impl Hash for ApiKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl ApiKey {
    pub fn new(str: &str) -> Self {
        ApiKey {
            id: new_api_key_id(),
            key: str.to_string(),
            data: ApiKeyData::empty(),
            show_key_in_clears: default_show_key_in_clears(),
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
}

#[derive(Serialize, Deserialize)]
pub struct AccountData {
    id: String,
    name: String,
    last_modified: DateTime<Utc>,
}

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

impl Settings {
    fn default() -> Self {
        Settings {
            api_keys: default_api_keys(),
            short_names: default_short_name(),
            check_updates: default_check_updates(),
            finished_clear_color: default_finished_clear_color(),
            unfinished_clear_color: default_unfinished_clear_color(),
            clears_style: default_clears_style(),
            account_header_style: default_account_header(),
            main_window_keybind: default_main_window_keybind(),
            api_window_keybind: default_api_window_keybind(),
            close_window_with_escape: default_close_window_with_escape(),
        }
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

    pub fn short_names(&self) -> bool {
        self.short_names
    }

    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let settings: Settings = serde_json::from_reader(reader)?;

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
        if let Ok(settings) = Settings::load_from_file(filename) {
            *settings_mutex.lock().unwrap() = Some(settings);
        } else {
            // TODO: Log failure (unless file doesn't exist) and reset settings
            *settings_mutex.lock().unwrap() = Some(Settings::default());
        }

        if let Some(function) = continue_with {
            function();
        }
    });
}
