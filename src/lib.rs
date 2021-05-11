use arcdps::arcdps_export;
use arcdps::imgui;
use arcdps::imgui::{im_str, Window, StyleColor, ImString, StyleVar, TabBar, TabItem, TableBgTarget, TableFlags, ImStr};
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::clears::ClearData;
use crate::settings::{Settings, ApiKey};
use crate::api::{ApiMock, LiveApi};
use crate::workers::BackgroundWorkers;
use std::ops::{Deref, DerefMut};
use std::error::Error;
use itertools::Itertools;

mod api;
mod clears;
mod workers;
mod settings;

arcdps_export! {
    name: "Clears",
    sig: 0xC1EA55u32,
    options_end: options_end,
    imgui: imgui,
    init: init,
    release: release,
}

// This entire thing is probably overcomplicated.
lazy_static! {
    static ref BACKGROUND_WORKERS: Mutex<Option<BackgroundWorkers>> = Mutex::new(None);
    static ref DATA: Mutex<Data> = Mutex::new(Data {clears: ClearData::new()});
    static ref UI_STATE: Mutex<UiState> = Mutex::new(UiState {ui_shown: false});
    static ref SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);
}

struct UiState {
    ui_shown: bool,
}

pub struct Data {
    clears: ClearData
}

const SETTINGS_FILENAME: &str = "addons/arcdps/settings_clears.json";

fn init() {
    settings::load_bg(&SETTINGS, SETTINGS_FILENAME, Some(|| {
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

fn imgui(ui: &imgui::Ui, not_loading_or_character_selection: bool) {
    if !not_loading_or_character_selection {
        return;
    }

    let mut ui_state = UI_STATE.lock().unwrap();
    if !ui_state.ui_shown {
        return;
    }

    let mut data = DATA.lock().unwrap();
    let mut settings = SETTINGS.lock().unwrap();

    //ui.show_demo_window(&mut ui_state.ui_shown);

    Window::new(im_str!("Clears"))
        .always_auto_resize(true)
        .focus_on_appearing(false)
        .no_nav()
        .collapsible(false)
        .opened(&mut ui_state.ui_shown)
        .build(&ui, || {
            TabBar::new(im_str!("main_tabs"))
                .build(&ui, || {
                    TabItem::new(im_str!("My clears"))
                        .build(&ui, || {
                            if let Some(raids) = data.clears.raids() {
                                if let Some(clears) = data.clears.state() {
                                    let max_bosses = raids.wings().iter().map(|x| x.encounters().len()).max().unwrap_or_default();
                                    ui.begin_table_with_flags(im_str!("ClearsTable"), (max_bosses + 1) as i32, TableFlags::BORDERS);
                                    ui.table_setup_column(&im_str!(""));
                                    for boss in 0..max_bosses {
                                        ui.table_setup_column(&im_str!("Boss {}", boss + 1));
                                    }
                                    ui.table_headers_row();
                                    for (wing_index, wing) in raids.wings().iter().enumerate() {
                                        ui.table_next_row();
                                        ui.table_next_column();
                                        ui.text(im_str!("W{}", wing_index + 1));
                                        for column in 0..max_bosses {
                                            ui.table_next_column();
                                            if let Some(encounter) = wing.encounters().get(column) {
                                                let finished = clears.is_finished(&encounter);

                                                let bg_color = if finished {
                                                    [8. / 255., 148. / 255., 0. / 255., 1.]
                                                } else {
                                                    [157. / 255., 0. / 255., 6. / 255., 1.]
                                                };

                                                let text = ImString::new(encounter.english_name());

                                                // Center the text
                                                let current_x = ui.cursor_pos()[0];
                                                let text_width = ui.calc_text_size(&text, false, -1.0)[0];
                                                let column_width = ui.current_column_width();
                                                let new_x = (current_x + column_width / 2. - text_width / 2.).max(current_x);
                                                ui.set_cursor_pos([new_x, ui.cursor_pos()[1]]);
                                                ui.text(text);

                                                ui.table_set_bg_color(TableBgTarget::CELL_BG, bg_color);
                                            }
                                            ui.next_column()
                                        }
                                    }
                                    ui.end_table();
                                }
                            }
                        });
                    TabItem::new(im_str!("Friends"))
                        .build(&ui, || {
                            ui.text("Not implemented yet.")
                        });
                    TabItem::new(im_str!("Settings"))
                        .build(&ui, || {
                            if let Some(settings) = settings.deref_mut() {
                                let mut api_key = settings.main_api_key().as_ref()
                                    .map(|x| ImString::new(x.key())).unwrap_or_default();

                                if ui.input_text(im_str!("GW2 API Key"), &mut api_key)
                                    .resize_buffer(true)
                                    .build() {
                                    settings.set_main_api_key(Some(ApiKey::new(api_key.to_str())));
                                }
                            }
                        });
                });
        });
}

fn options_end(ui: &imgui::Ui) {
    let mut ui_state = UI_STATE.lock().unwrap();
    /*
    if ui.button(im_str!("Clears"), [ui.current_column_width(), ui.current_font_size() + 8.0]) {
        ui_state.ui_shown = true;
    }
    */
    ui.checkbox(im_str!("Clears"), &mut ui_state.ui_shown);
}
