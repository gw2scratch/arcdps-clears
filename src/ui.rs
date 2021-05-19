use arcdps::imgui;
use crate::translations::{Translation, encounter_english_name};
use crate::settings::{ApiKey, Settings};
use arcdps::imgui::{im_str, ImString, TabItem, TableBgTarget, TableFlags, TabBar, Window, Ui};
use crate::{Data};
use crate::workers::BackgroundWorkers;
use std::time::{SystemTime, Instant};
use crate::updates::Release;

pub struct UiState {
    pub main_window: MainWindowState,
    pub update_window: UpdateWindowState
}

impl UiState {
    pub fn new() -> Self {
        UiState {
            main_window: MainWindowState { shown: false },
            update_window: UpdateWindowState { shown: false, release: None },
        }
    }
}

pub struct MainWindowState {
    pub shown: bool
}

pub struct UpdateWindowState {
    pub shown: bool,
    pub release: Option<Release>
}

pub fn draw_ui(ui: &Ui, ui_state: &mut UiState, data: &mut Data, settings: &mut Settings, bg_workers: &BackgroundWorkers, tr: &Translation) {
    if ui_state.main_window.shown {
        Window::new(&tr.im_string("window-title"))
            .always_auto_resize(true)
            .focus_on_appearing(false)
            .no_nav()
            .collapsible(false)
            .opened(&mut ui_state.main_window.shown)
            .build(&ui, || {
                TabBar::new(im_str!("main_tabs"))
                    .build(&ui, || {
                        TabItem::new(&tr.im_string("clears-tab-title"))
                            .build(&ui, || { clears(ui, data, bg_workers, settings, tr) });
                        TabItem::new(&tr.im_string("friends-tab-title"))
                            .build(&ui, || { friends(ui, tr) });
                        TabItem::new(&tr.im_string("settings-tab-title"))
                            .build(&ui, || { self::settings(ui, settings, tr) });
                    });
            });
    }

    if ui_state.update_window.shown {
        let release = &ui_state.update_window.release;
        let mut shown = ui_state.update_window.shown;
        let mut close = false;
        Window::new(&tr.im_string("update-window-title"))
            .always_auto_resize(true)
            .focus_on_appearing(false)
            .no_nav()
            .collapsible(false)
            .opened(&mut shown)
            .build(&ui, || {
                if let Some(release) = release {
                    ui.text(tr.im_string("update-available"));
                    ui.separator();
                    if ui.begin_table(im_str!("UpdateVersionColumns"), 2) {
                        ui.table_next_row();
                        ui.table_next_column();
                        ui.text(tr.im_string("update-current-version-prefix"));
                        ui.table_next_column();
                        ui.text(env!("CARGO_PKG_VERSION"));
                        ui.table_next_row();
                        ui.table_next_column();
                        ui.text(tr.im_string("update-new-version-prefix"));
                        ui.table_next_column();
                        ui.text(release.version());
                        ui.end_table();
                    }
                    ui.separator();
                    if ui.button(&tr.im_string("update-button-ignore"), [0.0, 0.0]) {
                        settings.ignore_version(release.version());
                        close = true;
                    }
                    ui.same_line(0.0);
                    if ui.button(&tr.im_string("update-button-changelog"), [0.0, 0.0]) {
                        open::that(release.changelog_url());
                    }
                    ui.same_line(0.0);
                    if ui.button(&tr.im_string("update-button-download"), [0.0, 0.0]) {
                        open::that(release.tool_site_url());
                    }
                } else {
                    ui.text(tr.im_string("update-not-available"))
                }
            });

        ui_state.update_window.shown = shown && !close;
    }
}

fn clears(ui: &Ui, data: &Data, bg_workers: &BackgroundWorkers, settings: &Settings, tr: &Translation) {
    if let Some(raids) = data.clears.raids() {
        if let Some(clears) = data.clears.state() {
            let max_bosses = raids.wings().iter().map(|x| x.encounters().len()).max().unwrap_or_default();
            ui.begin_table_with_flags(im_str!("ClearsTable"), (max_bosses + 1) as i32, TableFlags::BORDERS);
            ui.table_setup_column(&im_str!(""));
            for boss in 0..max_bosses {
                ui.table_setup_column(&im_str!("{} {}", tr.im_string("clears-header-boss"), boss + 1));
            }
            ui.table_headers_row();
            for (wing_index, wing) in raids.wings().iter().enumerate() {
                ui.table_next_row();
                ui.table_next_column();
                ui.text(im_str!("{}{}", tr.im_string("clears-wing-prefix"), wing_index + 1));
                for column in 0..max_bosses {
                    ui.table_next_column();
                    if let Some(encounter) = wing.encounters().get(column) {
                        let finished = clears.is_finished(&encounter);

                        let bg_color = if finished {
                            [8. / 255., 148. / 255., 0. / 255., 1.]
                        } else {
                            [157. / 255., 0. / 255., 6. / 255., 1.]
                        };

                        if settings.short_names() {
                            utils::centered_text(&ui, &tr.encounter_short_name_im_string(encounter));
                        } else {
                            utils::centered_text(&ui, &ImString::new(encounter_english_name(encounter)));
                        }

                        ui.table_set_bg_color(TableBgTarget::CELL_BG, bg_color);
                    }
                    ui.next_column()
                }
            }
            ui.end_table();
        } else {
            utils::centered_text(&ui, &tr.im_string("clears-no-clears-data-yet"));
            utils::centered_text(&ui, &tr.im_string("clears-no-clears-data-api-key-prompt"));
            ui.text("");

            let time = *bg_workers.api_worker_next_wakeup().lock().unwrap();
            let until_wakeup = time.saturating_duration_since(Instant::now());
            utils::centered_text(&ui, &im_str!("{}{}{}", tr.im_string("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.im_string("next-refresh-secs-suffix")));
        }
    } else {
        ui.text(tr.im_string("clears-no-public-data-yet"));
        ui.text("");

        let time = *bg_workers.api_worker_next_wakeup().lock().unwrap();
        let until_wakeup = time.saturating_duration_since(Instant::now());
        utils::centered_text(&ui, &im_str!("{}{}{}", tr.im_string("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.im_string("next-refresh-secs-suffix")));
    }
}

fn friends(ui: &Ui, tr: &Translation) {
    ui.text(tr.im_string("not-implemented-yet"))
}

fn settings(ui: &Ui, settings: &mut Settings, tr: &Translation) {
    let mut api_key = settings.main_api_key().as_ref()
        .map(|x| ImString::new(x.key())).unwrap_or_default();

    // This has to be a password box to protect streamers and users
    // who may have others watching them.
    if ui.input_text(&tr.im_string("setting-gw2-api-key"), &mut api_key)
        .password(true)
        .resize_buffer(true)
        .build() {
        settings.set_main_api_key(Some(ApiKey::new(api_key.to_str())));
    }
    ui.same_line(0.0);
    utils::help_marker(ui, tr.im_string("setting-gw2-api-key-description"));

    ui.checkbox(&tr.im_string("setting-short-encounter-names"), &mut settings.short_names);
    ui.same_line(0.0);
    utils::help_marker(ui, tr.im_string("setting-short-encounter-names-description"));

    ui.checkbox(&tr.im_string("setting-check-updates"), &mut settings.check_updates);
    ui.same_line(0.0);
    utils::help_marker(ui, tr.im_string("setting-check-updates-description"));
}

mod utils {
    use super::*;
    use arcdps::imgui::ImStr;

    pub fn centered_text(ui: &Ui, text: &ImStr)  {
        let current_x = ui.cursor_pos()[0];
        let text_width = ui.calc_text_size(&text, false, -1.0)[0];
        let column_width = ui.current_column_width();
        let new_x = (current_x + column_width / 2. - text_width / 2.).max(current_x);
        ui.set_cursor_pos([new_x, ui.cursor_pos()[1]]);
        ui.text(text);
    }

    pub fn help_marker<T : AsRef<str>>(ui: &Ui, text: T) {
        ui.text_disabled("(?)");
        if ui.is_item_hovered() {
            ui.tooltip(|| {
                let wrap = ui.push_text_wrap_pos(ui.current_font_size() * 35.0);
                ui.text(text);
                wrap.pop(&ui);
            });
        }
    }
}
