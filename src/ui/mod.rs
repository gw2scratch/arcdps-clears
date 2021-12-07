// Do not warn when using the `if let _token = ui.push_style() {}` pattern.
#![allow(irrefutable_let_patterns)]

use crate::settings::{ApiKey, Settings};
use crate::translations::Translation;
use crate::updates::Release;
use crate::workers::BackgroundWorkers;
use crate::Data;
use arcdps::imgui::{TabItem, TabBar, Window, Ui};
use uuid::Uuid;
use std::time::Instant;

pub mod settings;
mod apikeys;
mod updates;
mod clears;
mod utils;
mod friends;
mod style;

pub struct UiState {
    pub main_window: MainWindowState,
    pub update_window: UpdateWindowState,
    pub api_key_window: ApiKeyWindowState,
    pub friends_window: FriendsWindowState,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            main_window: MainWindowState {
                shown: false
            },
            update_window: UpdateWindowState {
                shown: false,
                release: None,
            },
            api_key_window: ApiKeyWindowState {
                shown: false,
                selected_key: SelectedApiKey::None,
                new_friend_name: String::new(),
            },
            friends_window: FriendsWindowState {
                shown: false,
                last_refresh_use: Instant::now(),
                new_friend_name: String::new(),
            }
        }
    }
}

pub enum SelectedApiKey {
    None,
    Id(Uuid),
}

pub struct ApiKeyWindowState {
    pub shown: bool,
    pub selected_key: SelectedApiKey,
    pub new_friend_name: String,
}

pub struct MainWindowState {
    pub shown: bool,
}

pub struct UpdateWindowState {
    pub shown: bool,
    pub release: Option<Release>,
}

pub struct FriendsWindowState {
    pub shown: bool,
    pub last_refresh_use: Instant,
    pub new_friend_name: String,
}

impl ApiKeyWindowState {
    pub fn is_key_selected(&self, key: &ApiKey) -> bool {
        if let SelectedApiKey::Id(uuid) = self.selected_key {
            *key.id() == uuid
        } else {
            false
        }
    }
}

fn get_api_key_name(api_key: &ApiKey, tr: &Translation) -> String {
    if let Some(name) = api_key.data().account_data().as_ref().map(|x| x.name()) {
        name.to_string()
    } else {
        tr.translate("api-key-new-key-name")
    }
}

pub fn draw_ui(
    ui: &Ui,
    ui_state: &mut UiState,
    data: &mut Data,
    settings: &mut Settings,
    bg_workers: &BackgroundWorkers,
    tr: &Translation,
) {
    if ui_state.main_window.shown {
        let mut shown = ui_state.main_window.shown;
        Window::new(&tr.translate("window-title"))
            .always_auto_resize(true)
            .focus_on_appearing(false)
            .no_nav()
            .title_bar(settings.main_window_show_title)
            .draw_background(settings.main_window_show_bg)
            .collapsible(false)
            .opened(&mut shown)
            .build(&ui, || {
                TabBar::new("main_tabs").build(&ui, || {
                    TabItem::new(&tr.translate("clears-tab-title"))
                        .build(&ui, || {
                            clears::my_clears(ui, ui_state, data, bg_workers, settings, tr);
                        });
                    TabItem::new(&tr.translate("friends-tab-title"))
                        .build(&ui, || friends::friends(ui, ui_state, data, bg_workers, settings, tr));
                    TabItem::new(&tr.translate("settings-tab-title"))
                        .build(&ui, || settings::settings(ui, ui_state, settings, tr));
                });
            });
        ui_state.main_window.shown = shown;
    }


    friends::friends_window(ui, ui_state, data, bg_workers, settings, tr);

    updates::update_window(ui, ui_state, tr);

    apikeys::api_keys_window(ui, ui_state, data, bg_workers, settings, tr);
}