use crate::settings::{ApiKey, Settings};
use crate::translations::Translation;
use crate::updates::Release;
use crate::workers::BackgroundWorkers;
use crate::Data;
use arcdps::imgui::{im_str, TabItem, TabBar, Window, Ui, ImString, MouseButton, StyleVar};
use uuid::Uuid;

mod settings;
mod apikeys;
mod updates;
mod clears;
mod utils;
mod friends;

pub struct UiState {
    pub main_window: MainWindowState,
    pub update_window: UpdateWindowState,
    pub api_key_window: ApiKeyWindowState,
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            main_window: MainWindowState { shown: false },
            update_window: UpdateWindowState {
                shown: false,
                release: None,
            },
            api_key_window: ApiKeyWindowState {
                shown: false,
                selected_key: SelectedApiKey::None,
            },
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
}

pub struct MainWindowState {
    pub shown: bool,
}

pub struct UpdateWindowState {
    pub shown: bool,
    pub release: Option<Release>,
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

fn get_api_key_name(api_key: &ApiKey, tr: &Translation) -> ImString {
    if let Some(name) = api_key.data().account_data().as_ref().map(|x| x.name()) {
        ImString::new(name)
    } else {
        tr.im_string("api-key-new-key-name")
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
        Window::new(&tr.im_string("window-title"))
            .always_auto_resize(true)
            .focus_on_appearing(false)
            .no_nav()
            .collapsible(false)
            .opened(&mut shown)
            .build(&ui, || {
                TabBar::new(im_str!("main_tabs")).build(&ui, || {
                    TabItem::new(&tr.im_string("clears-tab-title"))
                        .build(&ui, || clears::clears(ui, ui_state, data, bg_workers, settings, tr));
                    TabItem::new(&tr.im_string("friends-tab-title"))
                        .build(&ui, || friends::friends(ui, tr));
                    TabItem::new(&tr.im_string("settings-tab-title"))
                        .build(&ui, || settings::settings(ui, ui_state, settings, tr));
                });

                if ui.is_mouse_released(MouseButton::Right) && ui.is_window_hovered() {
                    ui.open_popup(im_str!("##RightClickMenu"));
                }

                ui.popup(im_str!("##RightClickMenu"), || {
                    let small_frame_padding = ui.push_style_var(StyleVar::FramePadding([1.0, 1.0]));
                    settings::style_section(&ui, settings, tr);
                    small_frame_padding.pop(&ui);
                });
            });
        ui_state.main_window.shown = shown;
    }

    updates::update_window(ui, ui_state, tr);

    apikeys::api_keys_window(ui, ui_state, bg_workers, settings, tr);
}