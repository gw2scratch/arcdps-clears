use arcdps::arcdps_export;
use arcdps::imgui;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::clears::ClearData;
use crate::settings::Settings;
use crate::translations::{Translation};
use crate::api::LiveApi;
use crate::workers::BackgroundWorkers;
use crate::ui::UiState;
use std::ops::Deref;
use log::error;
use crate::friends::{FriendData, FriendsApiClient};


mod api;
mod clears;
mod workers;
mod settings;
mod ui;
mod translations;
mod updates;
mod input;
mod friends;
mod urls;

const SETTINGS_FILENAME: &str = "addons/arcdps/settings_clears.json";
const TRANSLATION_FILENAME: &str = "addons/arcdps/arcdps_lang_clears.json";

arcdps_export! {
    name: "Clears",
    sig: 0xC1EA55u32,
    options_windows: options,
    options_end: options_end,
    imgui: imgui,
    init: init,
    release: release,
    wnd_filter: wnd_filter,
    wnd_nofilter: wnd_nofilter,
}

// This entire thing is probably overcomplicated.
lazy_static! {
    static ref BACKGROUND_WORKERS: Mutex<Option<BackgroundWorkers>> = Mutex::new(None);
    static ref DATA: Mutex<Data> = Mutex::new(Data::new());
    static ref UI_STATE: Mutex<UiState> = Mutex::new(UiState::new());
    static ref SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);
    // We fall back to the default translation before there's an attempt to load a translation.
    static ref TRANSLATION: Mutex<Translation> = Mutex::new(Translation::load_from_string(translations::get_default_translation_contents()).expect("Failed to load default translation!"));
}

pub struct Data {
    clears: ClearData,
    friends: FriendData,
}

impl Data {
    pub fn new() -> Self {
        Data { clears: ClearData::new(), friends: FriendData::new() }
    }
}

fn init() {
    std::thread::spawn(move || {
        // If this fails in any way, the default translation is kept.
        if let Ok(translation) = Translation::load_from_file(TRANSLATION_FILENAME) {
            *TRANSLATION.lock().unwrap() = translation;
        }
    });
    settings::load_bg(&SETTINGS, SETTINGS_FILENAME, Some(|| {
        if SETTINGS.lock().unwrap().as_ref().expect("Settings should be loaded by now.").check_updates {
            std::thread::spawn(move || {
                // TODO: Log update check failures
                if let Ok(Some(release)) = updates::get_update(&SETTINGS.lock().unwrap().as_ref().unwrap()) {
                    let mut ui_state = UI_STATE.lock().unwrap();
                    ui_state.update_window.release = Some(release);
                    ui_state.update_window.shown = true;
                }
            });
        }

        let friends_api_url = SETTINGS.lock().unwrap().as_ref()
            .expect("Settings should be loaded by now.").friends.friends_api_url.to_string();

        *BACKGROUND_WORKERS.lock().unwrap() = Some(workers::start_workers(
            &DATA,
            &SETTINGS,
            LiveApi::official(),
            FriendsApiClient::new(friends_api_url),
        ));
    }));
}

fn release() {
    if let Some(settings) = SETTINGS.lock().unwrap().deref() {
        match settings.save_to_file(SETTINGS_FILENAME) {
            Ok(_) => {}
            Err(err) => {
                error!("Failed to save settings: {:?}", err)
            }
        }
    }
}

fn imgui(imgui_ui: &imgui::Ui, not_loading_or_character_selection: bool) {
    let mut settings = SETTINGS.lock().unwrap();
    if settings.is_none() {
        // We wait for settings to get loaded first.
        return;
    }

    if settings.as_ref().expect("Settings should be loaded at this point").hide_in_loading_screens
        && !not_loading_or_character_selection {
        return;
    }

    let mut ui_state = UI_STATE.lock().unwrap();
    let mut data = DATA.lock().unwrap();
    let translation = TRANSLATION.lock().unwrap();
    let workers = BACKGROUND_WORKERS.lock().unwrap();
    if workers.is_none() {
        // We wait for workers to get started first.
        return;
    }

    #[cfg(debug_assertions)]
        imgui_ui.show_demo_window(&mut ui_state.main_window.shown);

    ui::draw_ui(imgui_ui,
                &mut ui_state,
                &mut data,
                &mut settings.as_mut().expect("Settings should be loaded at this point."),
                &workers.as_ref().expect("Workers should be created at this point."),
                &translation,
    );
}

fn options(ui: &imgui::Ui, window_name: Option<&str>) -> bool {
    if window_name.is_none() {
        let tr = TRANSLATION.lock().unwrap();
        let mut ui_state = UI_STATE.lock().unwrap();
        ui.checkbox(&tr.translate("arcdps-menu-name"), &mut ui_state.main_window.shown);
    }

    return false;
}

fn options_end(ui: &imgui::Ui) {
    let mut ui_state = UI_STATE.lock().unwrap();
    let mut settings = SETTINGS.lock().unwrap();
    let tr = TRANSLATION.lock().unwrap();

    if settings.is_none() {
        // We wait for settings to get loaded first.
        return;
    }
    ui::settings::settings(ui,
                           &mut ui_state,
                           settings.as_mut().expect("Settings should be loaded at this point."),
                           &tr,
    );
}


fn wnd_filter(key: usize, key_down: bool, _prev_key_down: bool) -> bool {
    if let Some(settings) = SETTINGS.lock().unwrap().as_ref() {
        if let Some(main_window_keybind) = settings.keybinds.main_window {
            if key_down && key == main_window_keybind {
                let shown = UI_STATE.lock().unwrap().main_window.shown;
                UI_STATE.lock().unwrap().main_window.shown = !shown;
                return false;
            }
        }

        if let Some(api_window_keybind) = settings.keybinds.api_window {
            if key_down && key == api_window_keybind {
                let shown = UI_STATE.lock().unwrap().api_key_window.shown;
                UI_STATE.lock().unwrap().api_key_window.shown = !shown;
                return false;
            }
        }
    }

    return true;
}

fn wnd_nofilter(key: usize, key_down: bool, _prev_key_down: bool) -> bool {
    if let Some(settings) = SETTINGS.lock().unwrap().as_ref() {
        if settings.close_window_with_escape {
            if key_down && key == input::KEY_ESCAPE {
                // We do not close the update window to avoid accidental closes.

                if UI_STATE.lock().unwrap().api_key_window.shown {
                    UI_STATE.lock().unwrap().api_key_window.shown = false;
                    return false;
                }

                if UI_STATE.lock().unwrap().main_window.shown {
                    UI_STATE.lock().unwrap().main_window.shown = false;
                    return false;
                }
            }
        }
    }

    return true;
}
