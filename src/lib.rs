use arcdps::arcdps_export;
use arcdps::imgui;
use arcdps::imgui::{im_str, Window, StyleColor, ImString, StyleVar, TabBar, TabItem, TableBgTarget, TableFlags, ImStr};
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::clears::ClearData;
use crate::settings::{Settings, ApiKey};
use crate::translations::{Translation};
use crate::api::{ApiMock, LiveApi};
use crate::workers::BackgroundWorkers;
use crate::ui::{UiState, draw_ui};
use std::ops::{Deref, DerefMut};
use std::error::Error;
use itertools::Itertools;

mod api;
mod clears;
mod workers;
mod settings;
mod ui;
mod translations;
mod updates;

const SETTINGS_FILENAME: &str = "addons/arcdps/settings_clears.json";
const TRANSLATION_FILENAME: &str = "addons/arcdps/arcdps_lang_clears.json";

arcdps_export! {
    name: "Clears",
    sig: 0xC1EA55u32,
    options_windows: options,
    imgui: imgui,
    init: init,
    release: release,
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
}

impl Data {
    pub fn new() -> Self {
        Data { clears: ClearData::new() }
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
        *BACKGROUND_WORKERS.lock().unwrap() = Some(workers::start_workers(&DATA, &SETTINGS, LiveApi::official()));
    }));
}

fn release() {
    if let Some(settings) = SETTINGS.lock().unwrap().deref() {
        match settings.save_to_file(SETTINGS_FILENAME) {
            Ok(_) => {}
            Err(err) => {
                // TODO: Proper logging
                eprintln!("Failed to save settings: {:?}", err)
            }
        }
    }
}

fn imgui(imgui_ui: &imgui::Ui, not_loading_or_character_selection: bool) {
    if !not_loading_or_character_selection {
        return;
    }

    let mut ui_state = UI_STATE.lock().unwrap();
    let mut settings = SETTINGS.lock().unwrap();
    if settings.is_none() {
        // We wait for settings to get loaded first.
        return;
    }
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
        /*
        if ui.button(im_str!("Clears"), [ui.current_column_width(), ui.current_font_size() + 8.0]) {
            ui_state.main_window.shown = true;
        }
        */
        ui.checkbox(&tr.im_string("arcdps-menu-name"), &mut ui_state.main_window.shown);
    }

    return false;
}
