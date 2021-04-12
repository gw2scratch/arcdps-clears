use arcdps::arcdps_export;
use arcdps::imgui;
use arcdps::imgui::{im_str, Window, StyleColor, ImString, StyleVar, TabBar, TabItem};
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::clears::ClearData;
use crate::settings::{Settings, ApiKey};
use crate::api::{ApiMock, LiveApi};
use crate::workers::BackgroundWorkers;
use std::ops::{Deref, DerefMut};
use std::error::Error;

mod api;
mod clears;
mod workers;
mod settings;

arcdps_export! {
    name: "Clears",
    sig: 894894615u32,
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

    //ui.show_demo_window(&mut true);
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

                                    for wing in raids.wings() {
                                        for column in 0..max_bosses {
                                            if let Some(encounter) = wing.encounters().get(column) {
                                                if column != 0 {
                                                    ui.same_line(0.0);
                                                }

                                                let finished = clears.is_finished(&encounter);

                                                if finished {
                                                    let green = [0.0, 0.6, 0.0, 0.85];
                                                    let color = ui.push_style_color(StyleColor::Button, green);
                                                    let color2 = ui.push_style_color(StyleColor::ButtonHovered, green);
                                                    let color3 = ui.push_style_color(StyleColor::ButtonActive, green);
                                                    let var = ui.push_style_var(StyleVar::FramePadding([3.0, 3.0]));
                                                    ui.button(&ImString::new(encounter.english_name()), [0., 0.]);
                                                    color.pop(&ui);
                                                    color2.pop(&ui);
                                                    color3.pop(&ui);
                                                    var.pop(&ui);
                                                } else {
                                                    let red = [0.6, 0.0, 0.0, 0.85];
                                                    let color = ui.push_style_color(StyleColor::Button, red);
                                                    let color2 = ui.push_style_color(StyleColor::ButtonHovered, red);
                                                    let color3 = ui.push_style_color(StyleColor::ButtonActive, red);
                                                    let var = ui.push_style_var(StyleVar::FramePadding([2.0, 3.0]));
                                                    ui.button(&ImString::new(encounter.english_name()), [0., 0.]);
                                                    color.pop(&ui);
                                                    color2.pop(&ui);
                                                    color3.pop(&ui);
                                                    var.pop(&ui);
                                                }
                                            }
                                        }
                                    }
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
