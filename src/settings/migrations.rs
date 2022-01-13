use chrono::{DateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Settings;
use crate::settings::{AccountData, AccountHeaderStyle, ApiKey, ApiKeyData, ClearsTableStyle, FeatureAdverts, Keybinds, TokenInfo, TokenType};

pub fn load_old_settings(json: &str) -> Option<Settings> {
    if let Some(settings) = load_0_1_0(json) {
        info!("Migrated clears 0.1 settings");
        Some(settings)
    } else if let Some(settings) = load_0_3(json) {
        info!("Migrated clears 0.3 settings");
        Some(settings)
    } else {
        None
    }
}

pub fn load_0_1_0(json: &str) -> Option<Settings> {
    /// Settings from version 0.1.0
    #[derive(Serialize, Deserialize)]
    struct ApiKey0_1 {
        key: String,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Settings0_1 {
        main_api_key: Option<ApiKey0_1>,
        short_names: bool,
    }

    let old_setting_result: serde_json::Result<Settings0_1> = serde_json::from_str(json);
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

pub fn load_0_3(json: &str) -> Option<Settings> {
    /// Settings from version 0.3
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Settings0_3 {
        api_keys: Vec<ApiKey0_3>,
        short_names: bool,
        check_updates: bool,
        finished_clear_color: [f32; 4],
        unfinished_clear_color: [f32; 4],
        clears_style: ClearsStyle0_3,
        account_header_style: AccountHeaderStyle0_3,
        main_window_keybind: Option<usize>,
        api_window_keybind: Option<usize>,
        close_window_with_escape: bool,
        hide_in_loading_screens: bool,
        show_clears_table_headers: bool,
        show_clears_table_row_names: bool,
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    pub enum AccountHeaderStyle0_3 {
        None,
        CenteredText,
        Collapsible,
    }

    #[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    pub enum ClearsStyle0_3 {
        WingColumns,
        WingRows,
        SingleRow,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ApiKey0_3 {
        id: Uuid,
        key: String,
        data: ApiKeyData0_3,
        show_key_in_clears: bool,
        expanded_in_clears: bool,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct AccountData0_3 {
        id: String,
        name: String,
        last_modified: DateTime<Utc>,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ApiKeyData0_3 {
        account_data: Option<AccountData0_3>,
        token_info: Option<TokenInfo0_3>,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct TokenInfo0_3 {
        id: String,
        name: String,
        permissions: Vec<String>,
        token_type: TokenType0_3,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub enum TokenType0_3 {
        Unknown,
        ApiKey,
        Subtoken {
            expires_at: DateTime<Utc>,
            issued_at: DateTime<Utc>,
            urls: Option<Vec<String>>,
        },
    }

    let old_setting_result: serde_json::Result<Settings0_3> = serde_json::from_str(json);
    if let Ok(old_settings) = old_setting_result {
        let mut settings = Settings::default();

        for api_key in old_settings.api_keys {
            settings.api_keys.push(ApiKey {
                id: api_key.id,
                key: api_key.key,
                data: ApiKeyData {
                    account_data: api_key.data.account_data.map(|x| {
                        AccountData {
                            id: x.id,
                            name: x.name,
                            last_modified: x.last_modified,
                        }
                    }),
                    token_info: api_key.data.token_info.map(|x| {
                        TokenInfo {
                            id: x.id,
                            name: x.name,
                            permissions: x.permissions,
                            token_type: match x.token_type {
                                TokenType0_3::Unknown => TokenType::Unknown,
                                TokenType0_3::ApiKey => TokenType::ApiKey,
                                TokenType0_3::Subtoken { expires_at, issued_at, urls } => TokenType::Subtoken { expires_at, issued_at, urls }
                            },
                        }
                    }),
                },
                show_key_in_clears: api_key.show_key_in_clears,
                expanded_in_clears: api_key.expanded_in_clears,
            })
        }

        settings.short_names = old_settings.short_names;
        settings.check_updates = old_settings.check_updates;

        settings.my_clears_style.finished_clear_color = old_settings.finished_clear_color;
        settings.friends_clears_style.finished_clear_color = old_settings.finished_clear_color;

        settings.my_clears_style.unfinished_clear_color = old_settings.unfinished_clear_color;
        settings.friends_clears_style.unfinished_clear_color = old_settings.unfinished_clear_color;

        settings.my_clears_style.table_style = match old_settings.clears_style {
            ClearsStyle0_3::WingColumns => ClearsTableStyle::WingColumns,
            ClearsStyle0_3::WingRows => ClearsTableStyle::WingRows,
            ClearsStyle0_3::SingleRow => ClearsTableStyle::SingleRow,
        };
        settings.my_clears_style.account_header_style = match old_settings.account_header_style {
            AccountHeaderStyle0_3::None => AccountHeaderStyle::None,
            AccountHeaderStyle0_3::CenteredText => AccountHeaderStyle::CenteredText,
            AccountHeaderStyle0_3::Collapsible => AccountHeaderStyle::Collapsible,
        };

        settings.keybinds = Keybinds {
            main_window: old_settings.main_window_keybind,
            api_window: old_settings.api_window_keybind,
        };

        settings.close_window_with_escape = old_settings.close_window_with_escape;
        settings.hide_in_loading_screens = old_settings.hide_in_loading_screens;

        settings.my_clears_style.show_clears_table_headers = old_settings.show_clears_table_headers;
        settings.my_clears_style.show_clears_table_row_names = old_settings.show_clears_table_row_names;

        settings.feature_adverts = FeatureAdverts {
            friends_shown: false
        };

        return Some(settings);
    }
    None
}
