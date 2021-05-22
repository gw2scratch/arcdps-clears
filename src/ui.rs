use crate::settings::{ApiKey, Settings, TokenType};
use crate::translations::{encounter_english_name, Translation};
use crate::updates::Release;
use crate::workers::{ApiJob, BackgroundWorkers};
use crate::Data;
use arcdps::imgui;
use arcdps::imgui::{
    im_str, ChildWindow, Condition, ImStr, ImString, Selectable, StyleVar, TabBar, TabItem,
    TableBgTarget, TableFlags, Ui, Window,
};
use std::time::{Instant, SystemTime};
use uuid::Uuid;

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
                        .build(&ui, || clears(ui, ui_state, data, bg_workers, settings, tr));
                    TabItem::new(&tr.im_string("friends-tab-title")).build(&ui, || friends(ui, tr));
                    TabItem::new(&tr.im_string("settings-tab-title"))
                        .build(&ui, || self::settings(ui, ui_state, settings, tr));
                });
            });
        ui_state.main_window.shown = shown;
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
                    if ui.begin_table_with_flags(im_str!("UpdateVersionColumns"), 2, TableFlags::SIZING_FIXED_FIT) {
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

    if ui_state.api_key_window.shown {
        let mut shown = ui_state.api_key_window.shown;
        Window::new(&tr.im_string("api-key-window-title"))
            .size([ui.current_font_size() * 40.0, ui.current_font_size() * 25.0], Condition::FirstUseEver)
            .resizable(true)
            .focus_on_appearing(false)
            .no_nav()
            .collapsible(false)
            .opened(&mut shown)
            .build(&ui, || {
                let max_name_width = settings.api_keys.iter()
                    .map(|api_key| {
                        ui.calc_text_size(&get_api_key_name(api_key, tr), true, 0.0)[0]
                    })
                    .fold(0.0, f32::max);
                let left_pane_width = max_name_width.max(ui.current_font_size() * 10.0);

                let group = ui.begin_group();
                ChildWindow::new("ApiLeftPane")
                    // The -frame_height is to make space for buttons at the bottom.
                    .size([left_pane_width, -ui.frame_height_with_spacing()])
                    .build(&ui, || {
                        for (i, api_key) in settings.api_keys.iter().enumerate() {
                            let name = get_api_key_name(api_key, tr);
                            if Selectable::new(&im_str!("{}##{}", name, api_key.id().to_string()))
                                .selected(ui_state.api_key_window.is_key_selected(api_key))
                                .build(&ui) {
                                ui_state.api_key_window.selected_key = SelectedApiKey::Id(*api_key.id());
                            }
                        }
                    });
                if ui.button(&tr.im_string("api-key-add-new-button"), [0., 0.]) {
                    let new_key = ApiKey::new("");
                    ui_state.api_key_window.selected_key = SelectedApiKey::Id(*new_key.id());
                    settings.api_keys.push(new_key);
                }
                group.end(&ui);
                ui.same_line(0.0);
                let group = ui.begin_group();
                ChildWindow::new("ApiRightPane")
                    // The -frame_height is to make space for buttons at the bottom.
                    .size([0.0, -ui.frame_height_with_spacing()])
                    .build(&ui, || {
                        let mut selected_key = match ui_state.api_key_window.selected_key {
                            SelectedApiKey::None => None,
                            SelectedApiKey::Id(id) => settings.get_key_mut(&id)
                        };
                        if let Some(key) = selected_key {
                            let mut key_text = ImString::new(key.key());
                            let mut key_changed = false;

                            // This has to be a password box to protect streamers and users
                            // who may have others watching them.
                            if ui.input_text(&tr.im_string("api-key-key-label"), &mut key_text)
                                .password(true)
                                .resize_buffer(true)
                                .build() {
                                key_changed = true;
                            }
                            if key.data().account_data().is_none() && key.data().token_info().is_none() {
                                if ui.button(&tr.im_string("api-key-check-api-key-button"), [0.0, 0.0]) {
                                    let sender = bg_workers.api_sender();
                                    sender.send(ApiJob::UpdateAccountData(*key.id()));
                                    sender.send(ApiJob::UpdateTokenInfo(*key.id()));
                                    sender.send(ApiJob::UpdateClears(*key.id()));
                                }
                                ui.separator();
                                ui.text_wrapped(&tr.im_string("api-key-guide-step1-prefix"));
                                ui.same_line(0.0);
                                if ui.small_button(&tr.im_string("api-key-guide-step1-open")) {
                                    open::that("https://account.arena.net/applications/create");
                                }
                                ui.same_line(0.0);
                                ui.text(tr.im_string("api-key-guide-step1-url"));
                                ui.text_wrapped(&tr.im_string("api-key-guide-step2"));
                                ui.text_wrapped(&tr.im_string("api-key-guide-step3"));
                                ui.text_wrapped(&tr.im_string("api-key-guide-step4"));
                                ui.text_wrapped(&tr.im_string("api-key-guide-step5"));
                            }
                            if key.data().account_data().is_some() || key.data().token_info().is_some() {
                                ui.separator();
                                if ui.begin_table_with_flags(im_str!("ApiKeyData"), 2, TableFlags::SIZING_FIXED_FIT) {
                                    ui.table_next_row();
                                    ui.table_next_column();
                                    ui.text(tr.im_string("api-key-details-account-name"));
                                    ui.table_next_column();
                                    if let Some(name) = key.data().account_data().as_ref().map(|x| x.name()) {
                                        ui.text(name);
                                    } else {
                                        ui.text(tr.im_string("api-key-details-unknown-value"));
                                    }
                                    ui.table_next_row();
                                    ui.table_next_column();
                                    ui.text(tr.im_string("api-key-details-key-name"));
                                    ui.table_next_column();
                                    if let Some(name) = key.data().token_info().as_ref().map(|x| x.name()) {
                                        ui.text(name);
                                    } else {
                                        ui.text(tr.im_string("api-key-details-unknown-value"));
                                    }
                                    ui.table_next_row();
                                    ui.table_next_column();
                                    ui.text(tr.im_string("api-key-details-permissions"));
                                    ui.table_next_column();
                                    if let Some(token_info) = key.data().token_info() {
                                        let mut account = token_info.has_permission("account");
                                        let mut progression = token_info.has_permission("progression");

                                        let extra_perms: Vec<_> = token_info.permissions().iter()
                                            .filter(|x| *x != "account" && *x != "progression")
                                            .collect();

                                        // Account permission checkmark
                                        let disabled_style = ui.push_style_var(StyleVar::Alpha(0.75));
                                        let width = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
                                        ui.checkbox(im_str!("##Account"), &mut account);
                                        width.pop(&ui);
                                        disabled_style.pop(&ui);
                                        if ui.is_item_hovered() {
                                            ui.tooltip(|| {
                                                ui.text(tr.im_string("api-key-details-permission-account"))
                                            });
                                        }
                                        ui.same_line(0.0);

                                        // Progression permission checkmark
                                        let disabled_style = ui.push_style_var(StyleVar::Alpha(0.75));
                                        let width = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
                                        ui.checkbox(im_str!("##Progression"), &mut progression);
                                        width.pop(&ui);
                                        disabled_style.pop(&ui);
                                        if ui.is_item_hovered() {
                                            ui.tooltip(|| {
                                                ui.text(tr.im_string("api-key-details-permission-progression"))
                                            });
                                        }

                                        // Extra permission indicator
                                        if extra_perms.len() > 0 {
                                            ui.same_line(0.0);
                                            ui.text(format!("{}{}{}",
                                                            tr.im_string("api-key-details-permissions-extra-prefix"),
                                                            extra_perms.len(),
                                                            tr.im_string("api-key-details-permissions-extra-suffix")));
                                            if ui.is_item_hovered() {
                                                ui.tooltip(|| {
                                                    for extra_perm in extra_perms {
                                                        ui.text(extra_perm);
                                                    }
                                                });
                                            }
                                        }
                                    } else {
                                        ui.text(tr.im_string("api-key-details-unknown-value"));
                                    }
                                    ui.table_next_row();
                                    ui.table_next_column();
                                    ui.text(tr.im_string("api-key-details-key-type"));
                                    ui.table_next_column();
                                    let key_type = key.data().token_info().as_ref().map(|x| x.token_type());
                                    if let Some(TokenType::ApiKey) = key_type {
                                        ui.text(tr.im_string("api-key-details-key-type-apikey"));
                                    } else if let Some(TokenType::Subtoken { expires_at, issued_at, .. }) = key_type {
                                        ui.text(tr.im_string("api-key-details-key-type-subtoken"));
                                        ui.table_next_row();
                                        ui.table_next_column();
                                        ui.text(tr.im_string("api-key-details-subtoken-issued-at"));
                                        ui.table_next_column();
                                        ui.text(issued_at.to_rfc2822());
                                        ui.table_next_row();
                                        ui.table_next_column();
                                        ui.text(tr.im_string("api-key-details-subtoken-expires-at"));
                                        ui.table_next_column();
                                        ui.text(expires_at.to_rfc2822());
                                    } else {
                                        ui.text(tr.im_string("api-key-details-unknown-value"));
                                    }
                                    ui.end_table();
                                }
                            }

                            // Missing permission/access warnings
                            if let Some(token_info) = key.data().token_info() {
                                const WARNING_RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

                                if !token_info.has_permission("account") {
                                    ui.text_colored(WARNING_RED, tr.im_string("api-key-warning-permission-account-missing"));
                                }
                                if !token_info.has_permission("progression") {
                                    ui.text_colored(WARNING_RED, tr.im_string("api-key-warning-permission-progression-missing"));
                                }

                                if let TokenType::Subtoken { urls, .. } = token_info.token_type() {
                                    if let Some(urls) = urls {
                                        let account_access = urls.iter().any(|x| x == "/v2/account");
                                        let clears_access = urls.iter().any(|x| x == "/v2/account/raids");
                                        if !account_access {
                                            ui.text_colored(WARNING_RED, tr.im_string("api-key-warning-subtoken-url-missing-account"));
                                        }
                                        if !clears_access {
                                            ui.text_colored(WARNING_RED, tr.im_string("api-key-warning-subtoken-url-missing-account-raids"));
                                        }
                                    }
                                }
                            }


                            ui.separator();
                            if ui.checkbox(&tr.im_string("api-key-show-in-my-clears-checkbox"), key.show_key_in_clears_mut()) {
                                if key.show_key_in_clears() {
                                    bg_workers.api_sender().send(ApiJob::UpdateClears(*key.id()));
                                }
                            }

                            if key_changed {
                                key.change_key(key_text.to_str());
                            }
                        } else {
                            if settings.api_keys.len() == 0 {
                                ui.text_wrapped(&tr.im_string("api-key-window-intro-first-key"));
                            } else {
                                ui.text_wrapped(&tr.im_string("api-key-window-intro"));
                            }
                        }
                    });
                if let SelectedApiKey::Id(uuid) = ui_state.api_key_window.selected_key {
                    if ui.button(&tr.im_string("api-key-remove-key-button"), [0., 0.]) {
                        // TODO: Confirmation box
                        settings.remove_key(&uuid);
                    }
                }
                group.end(&ui);
            });

        ui_state.api_key_window.shown = shown;
    }
}

fn clears(
    ui: &Ui,
    ui_state: &mut UiState,
    data: &Data,
    bg_workers: &BackgroundWorkers,
    settings: &Settings,
    tr: &Translation,
) {
    if let Some(raids) = data.clears.raids() {
        let mut first = true;
        for (i, key) in settings.api_keys().iter().enumerate() {
            if !key.show_key_in_clears() {
                continue;
            }

            if !first {
                ui.separator();
            }
            first = false;

            utils::centered_text(ui, &get_api_key_name(key, tr));
            if let Some(clears) = data.clears.state(key) {
                let max_bosses = raids
                    .wings()
                    .iter()
                    .map(|x| x.encounters().len())
                    .max()
                    .unwrap_or_default();
                ui.begin_table_with_flags(
                    &im_str!("ClearsTable##{}", key.id()),
                    (max_bosses + 1) as i32,
                    TableFlags::BORDERS | TableFlags::NO_HOST_EXTEND_X,
                );
                ui.table_setup_column(&im_str!(""));
                for boss in 0..max_bosses {
                    ui.table_setup_column(&im_str!(
                        "{} {}",
                        tr.im_string("clears-header-boss"),
                        boss + 1
                    ));
                }
                ui.table_headers_row();
                for (wing_index, wing) in raids.wings().iter().enumerate() {
                    ui.table_next_row();
                    ui.table_next_column();
                    ui.text(im_str!(
                        "{}{}",
                        tr.im_string("clears-wing-prefix"),
                        wing_index + 1
                    ));
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
                                utils::centered_text(
                                    &ui,
                                    &tr.encounter_short_name_im_string(encounter),
                                );
                            } else {
                                utils::centered_text(
                                    &ui,
                                    &ImString::new(encounter_english_name(encounter)),
                                );
                            }

                            ui.table_set_bg_color(TableBgTarget::CELL_BG, bg_color);
                        }
                        ui.next_column()
                    }
                }
                ui.end_table();
            } else {
                utils::centered_text(&ui, &tr.im_string("clears-no-clears-data-yet"));
                ui.text("");
                // TODO: Custom prompt for missing perms

                let time = *bg_workers.api_worker_next_wakeup().lock().unwrap();
                let until_wakeup = time.saturating_duration_since(Instant::now());
                utils::centered_text(
                    &ui,
                    &im_str!(
                        "{}{}{}",
                        tr.im_string("next-refresh-secs-prefix"),
                        until_wakeup.as_secs(),
                        tr.im_string("next-refresh-secs-suffix")
                    ),
                );
            }
        }
    } else {
        ui.text(tr.im_string("clears-no-public-data-yet"));
        ui.text("");

        let time = *bg_workers.api_worker_next_wakeup().lock().unwrap();
        let until_wakeup = time.saturating_duration_since(Instant::now());
        utils::centered_text(
            &ui,
            &im_str!(
                "{}{}{}",
                tr.im_string("next-refresh-secs-prefix"),
                until_wakeup.as_secs(),
                tr.im_string("next-refresh-secs-suffix")
            ),
        );
    }
}

fn friends(ui: &Ui, tr: &Translation) {
    ui.text(tr.im_string("not-implemented-yet"))
}

fn settings(ui: &Ui, ui_state: &mut UiState, settings: &mut Settings, tr: &Translation) {
    ui.checkbox(
        &tr.im_string("setting-short-encounter-names"),
        &mut settings.short_names,
    );
    ui.same_line(0.0);
    utils::help_marker(
        ui,
        tr.im_string("setting-short-encounter-names-description"),
    );

    ui.checkbox(
        &tr.im_string("setting-check-updates"),
        &mut settings.check_updates,
    );
    ui.same_line(0.0);
    utils::help_marker(ui, tr.im_string("setting-check-updates-description"));

    if ui.button(&tr.im_string("setting-button-manage-api-keys"), [0.0, 0.0]) {
        ui_state.api_key_window.shown = true;
    }
}

mod utils {
    use super::*;
    use arcdps::imgui::ImStr;

    pub fn centered_text(ui: &Ui, text: &ImStr) {
        let current_x = ui.cursor_pos()[0];
        let text_width = ui.calc_text_size(&text, false, -1.0)[0];
        let column_width = ui.current_column_width();
        let new_x = (current_x + column_width / 2. - text_width / 2.).max(current_x);
        ui.set_cursor_pos([new_x, ui.cursor_pos()[1]]);
        ui.text(text);
    }

    pub fn help_marker<T: AsRef<str>>(ui: &Ui, text: T) {
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
