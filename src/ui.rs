use crate::settings::{ApiKey, Settings, TokenType, ClearsStyle, AccountHeaderStyle};
use crate::translations::{encounter_english_name, Translation};
use crate::updates::Release;
use crate::workers::{ApiJob, BackgroundWorkers};
use crate::Data;
use arcdps::imgui;
use arcdps::imgui::{im_str, ChildWindow, Condition, ImStr, ImString, Selectable, StyleVar, TabBar, TabItem, TableBgTarget, TableFlags, Ui, Window, ColorEdit, ColorEditFlags, ComboBox, ComboBoxFlags, TableColumnFlags, StyleColor, TableRowFlags, ItemFlag, InputText, ImGuiInputTextFlags, CollapsingHeader, TreeNodeFlags};
use std::time::{Instant, SystemTime};
use uuid::Uuid;
use std::borrow::Cow;

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
        let max_bosses = raids
            .wings()
            .iter()
            .map(|x| x.encounters().len())
            .max()
            .unwrap_or_default();

        match settings.clears_style {
            ClearsStyle::WingRows => {
                let mut first_key = true;
                for (i, key) in settings.api_keys().iter().enumerate() {
                    if !key.show_key_in_clears() {
                        continue;
                    }

                    if !first_key {
                        ui.separator();
                    }
                    first_key = false;

                    match settings.account_header_style {
                        AccountHeaderStyle::None => {}
                        AccountHeaderStyle::CenteredText => utils::centered_text(ui, &get_api_key_name(key, tr)),
                    };

                    if let Some(clears) = data.clears.state(key) {
                        ui.begin_table_with_flags(
                            &im_str!("ClearsTableRows##{}", key.id()),
                            (max_bosses + 1) as i32,
                            TableFlags::BORDERS | TableFlags::NO_HOST_EXTEND_X,
                        );
                        ui.table_setup_column(&im_str!(""));
                        for boss in 0..max_bosses {
                            ui.table_setup_column(&im_str!("{} {}", tr.im_string("clears-header-boss"), boss + 1 ));
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
                                        settings.finished_clear_color
                                    } else {
                                        settings.unfinished_clear_color
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
                        // TODO: Deduplicate
                        utils::centered_text(&ui, &tr.im_string("clears-no-clears-data-yet"));
                        ui.text("");
                        // TODO: Custom prompt for missing perms

                        let time = *bg_workers.api_worker_next_wakeup().lock().unwrap();
                        let until_wakeup = time.saturating_duration_since(Instant::now());
                        utils::centered_text(
                            &ui,
                            &im_str!("{}{}{}", tr.im_string("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.im_string("next-refresh-secs-suffix")),
                        );
                    }
                }
            }
            ClearsStyle::WingColumns => {
                let mut first_key = true;
                for (i, key) in settings.api_keys().iter().enumerate() {
                    if !key.show_key_in_clears() {
                        continue;
                    }

                    if !first_key {
                        ui.separator();
                    }
                    first_key = false;

                    match settings.account_header_style {
                        AccountHeaderStyle::None => {}
                        AccountHeaderStyle::CenteredText => utils::centered_text(ui, &get_api_key_name(key, tr)),
                    };

                    if let Some(clears) = data.clears.state(key) {
                        ui.begin_table_with_flags(
                            &im_str!("ClearsTableColumns##{}", key.id()),
                            (raids.wings().len() + 1) as i32,
                            TableFlags::BORDERS | TableFlags::NO_HOST_EXTEND_X,
                        );
                        ui.table_setup_column(&im_str!(""));
                        for (wing_index, wing) in raids.wings().iter().enumerate() {
                            ui.table_setup_column(&im_str!("{} {}", tr.im_string("clears-wing-prefix-full"), wing_index + 1));
                        }
                        ui.table_headers_row();
                        for boss in 0..max_bosses {
                            ui.table_next_row();
                            ui.table_next_column();
                            ui.text(&im_str!("{} {}", tr.im_string("clears-header-boss"), boss + 1 ));
                            for (wing_index, wing) in raids.wings().iter().enumerate() {
                                ui.table_next_column();
                                if let Some(encounter) = wing.encounters().get(boss) {
                                    let finished = clears.is_finished(&encounter);

                                    let bg_color = if finished {
                                        settings.finished_clear_color
                                    } else {
                                        settings.unfinished_clear_color
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
                        // TODO: Deduplicate
                        utils::centered_text(&ui, &tr.im_string("clears-no-clears-data-yet"));
                        ui.text("");
                        // TODO: Custom prompt for missing perms

                        let time = *bg_workers.api_worker_next_wakeup().lock().unwrap();
                        let until_wakeup = time.saturating_duration_since(Instant::now());
                        utils::centered_text(
                            &ui,
                            &im_str!("{}{}{}", tr.im_string("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.im_string("next-refresh-secs-suffix")),
                        );
                    }
                }
            }
            ClearsStyle::SingleRow => {
                /*
                Beware, this is significantly cursed, attempts to simplify this are likely to fail.

                Our aim is to have one header per wing, and multiple columns under each header.
                We also want to have wider borders between each wing.

                The imgui API doesn't quite offer that, and instead we have an an outer table
                containing an inner table per wing, and an extra table with rows for account names.

                There are a couple problems with this approach that we work around:

                Dynamic sizing does not work properly without specifying fixed column sizes on
                the outer table. Without that, the window had to be shaken for the inner tables
                to properly unfold to full size. Do not ask me why.

                Instead, we compute the width of or inner tables, accounting for borders and use
                fixed width columns.

                Another issue is that we had some extra cell padding added due to the extra layer
                of tables, which looks ugly with horizontal borders between rows enabled - horizontal
                borders add a border on the top as well, which only looks fine if there is
                no gap between it and the header. We reduce this padding to zero and reintroduce it
                manually in the headers.

                Also note that getting good rendering of borders requires a very specific
                configuration of flags that we found by experimenting, and in some cases vertical
                borders are affected by horizontal ones.
                */

                let max_name_width = settings.api_keys.iter()
                    .filter(|api_key| api_key.show_key_in_clears())
                    .map(|api_key| ui.calc_text_size(&get_api_key_name(api_key, tr), true, 0.0)[0])
                    .fold(0.0, f32::max);

                let cell_padding = ui.clone_style().cell_padding;
                // We need to remove cell padding of the outer table, but keep it in inner tables.
                // The cell padding is saved into the table on construction, which means we cannot
                // selectively apply it only to relevant parts.
                let outer_cell_padding = ui.push_style_var(StyleVar::CellPadding([0., 0.]));
                ui.begin_table_with_flags(
                    &im_str!("ClearsTableCompactOuter"),
                    (raids.wings().len() + 1) as i32,
                    TableFlags::BORDERS_OUTER | TableFlags::BORDERS_INNER_V | TableFlags::NO_HOST_EXTEND_X | TableFlags::SIZING_FIXED_FIT | TableFlags::NO_PAD_INNER_X,
                );
                outer_cell_padding.pop(&ui);

                let mut table_headers_names = Vec::new();
                table_headers_names.push(tr.im_string("clears-account-column-header"));
                // We add 10.0 to the account name width as right padding
                ui.table_setup_column_with_weight(table_headers_names.last().unwrap(), TableColumnFlags::WIDTH_FIXED, max_name_width + 10.0);
                for (wing_index, wing) in raids.wings().iter().enumerate() {
                    let inner_width = (ui.current_font_size() * 1.5).ceil() * wing.encounters().len() as f32
                        + (wing.encounters().len() - 1) as f32 // Inner borders
                        + 2.0; // Outer borders
                    table_headers_names.push(im_str!("{} {}", tr.im_string("clears-wing-prefix-full"), wing_index + 1));
                    ui.table_setup_column_with_weight(table_headers_names.last().unwrap(), TableColumnFlags::WIDTH_FIXED, inner_width);
                }

                // We construct headers manually to add missing padding instead of using table_headers_row()
                ui.table_next_row_with_flags(TableRowFlags::HEADERS);
                for i in 0..ui.table_get_column_count() {
                    if !ui.table_set_column_index(i) {
                        continue;
                    }
                    //ui.push_style_var(StyleVar::)
                    ui.dummy([0.0, ui.current_font_size() + cell_padding[1]]);
                    ui.same_line_with_spacing(0.0, 0.0);
                    ui.set_cursor_pos([ui.cursor_pos()[0] + cell_padding[0], ui.cursor_pos()[1] + cell_padding[1]]);
                    ui.table_header(&table_headers_names[i as usize]);
                }

                ui.table_next_column();

                // Account table
                ui.begin_table_with_flags(
                    &im_str!("ClearsTableCompactAccounts"),
                    1,
                    TableFlags::BORDERS_INNER_H | TableFlags::PAD_OUTER_X,
                );
                for (i, key) in settings.api_keys().iter().enumerate() {
                    if !key.show_key_in_clears() {
                        continue;
                    }
                    ui.table_next_column();
                    ui.text(&get_api_key_name(key, tr));
                }
                ui.end_table();

                // Wing tables
                for (wing_index, wing) in raids.wings().iter().enumerate() {
                    ui.table_next_column();
                    ui.begin_table_with_flags(
                        &im_str!("ClearsTableCompactWing{}", wing_index),
                        wing.encounters().len() as i32,
                        TableFlags::NO_PAD_OUTER_X | TableFlags::NO_PAD_INNER_X | TableFlags::BORDERS_INNER | TableFlags::BORDERS_OUTER_V | TableFlags::BORDERS_OUTER_H,
                    );

                    for (encounter_index, encounter) in wing.encounters().iter().enumerate() {
                        ui.table_setup_column_with_weight(&im_str!("W{}B{}", wing_index, encounter_index), TableColumnFlags::WIDTH_FIXED, (ui.current_font_size() * 1.5).ceil());
                    }

                    for (i, key) in settings.api_keys().iter().enumerate() {
                        if !key.show_key_in_clears() {
                            continue;
                        }

                        let state = data.clears.state(key);
                        for (encounter_index, encounter) in wing.encounters().iter().enumerate() {
                            ui.table_next_column();
                            if let Some(clears) = state {
                                let finished = clears.is_finished(&encounter);

                                let bg_color = if finished {
                                    settings.finished_clear_color
                                } else {
                                    settings.unfinished_clear_color
                                };

                                ui.table_set_bg_color(TableBgTarget::CELL_BG, bg_color);

                                // A centered checkbox with transparent background
                                let padding_style = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
                                let standard_style = ui.push_style_color(StyleColor::FrameBg, [0.0, 0.0, 0.0, 0.0]);
                                let hover_style = ui.push_style_color(StyleColor::FrameBgHovered, ui.style_color(StyleColor::FrameBg));
                                let active_style = ui.push_style_color(StyleColor::FrameBgActive, ui.style_color(StyleColor::FrameBg));

                                let current_x = ui.cursor_pos()[0];
                                let checkbox_width = ui.frame_height();
                                let column_width = ui.current_column_width();
                                let new_x = (current_x + column_width / 2. - checkbox_width / 2.).max(current_x);
                                ui.set_cursor_pos([new_x, ui.cursor_pos()[1]]);

                                let mut finished_checkbox_copy = finished;
                                if ui.checkbox(im_str!(""), &mut finished_checkbox_copy) {
                                    finished_checkbox_copy = finished;
                                }
                                active_style.pop(&ui);
                                hover_style.pop(&ui);
                                standard_style.pop(&ui);
                                padding_style.pop(&ui);
                            } else {
                                utils::centered_text(&ui, &tr.im_string("clears-compressed-layout-short-unknown"));
                            }
                        }
                    }

                    // Tooltips with encounter name for each column
                    for (i, encounter) in wing.encounters().iter().enumerate() {
                        if ui.table_get_column_flags_with_column(i as i32).contains(TableColumnFlags::IS_HOVERED) {
                            ui.tooltip(|| {
                                if settings.short_names() {
                                    utils::centered_text(&ui, &tr.encounter_short_name_im_string(encounter));
                                } else {
                                    utils::centered_text(&ui, &ImString::new(encounter_english_name(encounter)));
                                }
                            });
                        }
                    }
                    ui.end_table();
                }
                ui.end_table();
            }
        }
    } else {
        ui.text(tr.im_string("clears-no-public-data-yet"));
        ui.text("");

        let time = *bg_workers.api_worker_next_wakeup().lock().unwrap();
        let until_wakeup = time.saturating_duration_since(Instant::now());
        utils::centered_text(
            &ui,
            &im_str!("{}{}{}", tr.im_string("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.im_string("next-refresh-secs-suffix")),
        );
    }
}

fn friends(ui: &Ui, tr: &Translation) {
    ui.text(tr.im_string("not-implemented-yet"))
}

fn settings(ui: &Ui, ui_state: &mut UiState, settings: &mut Settings, tr: &Translation) {
    if CollapsingHeader::new(&tr.im_string("settings-section-updates"))
        .default_open(true)
        .build(&ui) {
        ui.checkbox(
            &tr.im_string("setting-check-updates"),
            &mut settings.check_updates,
        );
        ui.same_line(0.0);
        utils::help_marker(ui, tr.im_string("setting-check-updates-description"));
    }

    if CollapsingHeader::new(&tr.im_string("settings-section-style"))
        .default_open(true)
        .build(&ui) {
        /* Clear table styles */
        let table_styles = [
            ClearsStyle::WingRows,
            ClearsStyle::WingColumns,
            ClearsStyle::SingleRow,
        ];

        let mut table_style_index = table_styles.iter().position(|x| *x == settings.clears_style).unwrap_or_default();

        if ComboBox::new(&tr.im_string("setting-clears-style"))
            .build_simple(&ui, &mut table_style_index, &table_styles, &|style|
                Cow::from(match style {
                    ClearsStyle::WingRows => tr.im_string("setting-clears-style-option-rows"),
                    ClearsStyle::WingColumns => tr.im_string("setting-clears-style-option-columns"),
                    ClearsStyle::SingleRow => tr.im_string("setting-clears-style-option-single-row"),
                }),
            ) {
            settings.clears_style = table_styles[table_style_index];
        }
        ui.same_line(0.0);
        ui.align_text_to_frame_padding();
        utils::help_marker(ui, tr.im_string("setting-clears-style-description"));

        /* Short encounter names */
        ui.checkbox(
            &tr.im_string("setting-short-encounter-names"),
            &mut settings.short_names,
        );
        ui.same_line(0.0);
        utils::help_marker(
            ui,
            tr.im_string("setting-short-encounter-names-description"),
        );

        /* Account header styles */
        // We currently only have two account header styles, so we use a checkbox.
        // In the future, this may be changed into a combo box.
        let mut show_account_headers = match settings.account_header_style {
            AccountHeaderStyle::None => false,
            AccountHeaderStyle::CenteredText => true
        };

        if ui.checkbox(
            &tr.im_string("setting-clears-header-style"),
            &mut show_account_headers,
        ) {
            settings.account_header_style = match show_account_headers {
                true => AccountHeaderStyle::CenteredText,
                false => AccountHeaderStyle::None
            };
        }
        ui.same_line(0.0);
        ui.align_text_to_frame_padding();
        utils::help_marker(ui, tr.im_string("setting-clears-header-style-description"));

        /* Colors */
        ColorEdit::new(&tr.im_string("setting-finished-clear-color"), &mut settings.finished_clear_color)
            .flags(ColorEditFlags::NO_INPUTS | ColorEditFlags::ALPHA_PREVIEW_HALF)
            .build(&ui);
        ui.same_line(0.0);
        ui.align_text_to_frame_padding();
        utils::help_marker(ui, tr.im_string("setting-finished-clear-color-description"));

        ColorEdit::new(&tr.im_string("setting-unfinished-clear-color"), &mut settings.unfinished_clear_color)
            .flags(ColorEditFlags::NO_INPUTS | ColorEditFlags::ALPHA_PREVIEW_HALF)
            .build(&ui);
        ui.same_line(0.0);
        ui.align_text_to_frame_padding();
        utils::help_marker(ui, tr.im_string("setting-unfinished-clear-color-description"));
    }

    if CollapsingHeader::new(&tr.im_string("settings-section-keybinds"))
        .default_open(true)
        .build(&ui) {

        /* Keybind: Main window */
        utils::keybind_input(&ui, im_str!("##MainWindowKeybindInput"), &mut settings.main_window_keybind, tr);
        ui.same_line(0.0);
        ui.align_text_to_frame_padding();
        ui.text(tr.im_string("setting-keybind-window-clears"));
        ui.same_line(0.0);
        ui.align_text_to_frame_padding();
        utils::help_marker(ui, tr.im_string("setting-keybind-window-clears-description"));

        /* Close on escape */
        ui.checkbox(
            &tr.im_string("setting-close-window-with-escape"),
            &mut settings.close_window_with_escape,
        );
        ui.same_line(0.0);
        utils::help_marker(
            ui,
            tr.im_string("setting-close-window-with-escape-description"),
        );
    }

    if ui.button(&tr.im_string("setting-button-manage-api-keys"), [ui.current_column_width(), 0.0]) {
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

    pub fn keybind_input(ui: &Ui, label: &ImStr, keybind: &mut Option<usize>, tr: &Translation) {
        let mut keybind_buffer = match keybind {
            None => ImString::new(""),
            Some(key) => ImString::from(key.to_string())
        };
        let width_token = ui.push_item_width(ui.current_font_size() * 3.0);
        if InputText::new(&ui, label, &mut keybind_buffer)
            .chars_decimal(true)
            .build() {
            if let Ok(new_keybind) = keybind_buffer.to_str().parse() {
                *keybind = Some(new_keybind);
            } else {
                *keybind = None;
            }
        }
        width_token.pop(&ui);

        let original_style = ui.clone_style();
        let spacing_token = ui.push_style_var(StyleVar::ItemSpacing([1.0, original_style.item_spacing[1]]));
        ui.same_line(0.0);
        let mut preview_buffer = if let Some(keybind) = keybind {
            if let Some(name) = crate::input::get_key_name(*keybind) {
                ImString::new(name)
            } else {
                tr.im_string("input-keybind-unknown")
            }
        } else {
            tr.im_string("input-keybind-disabled")
        };
        let width_token = ui.push_item_width(ui.calc_text_size(&preview_buffer, true, 0.0)[0] + original_style.frame_padding[0] * 2.0);
        let alpha_token = ui.push_style_var(StyleVar::Alpha(0.5));
        InputText::new(&ui, &im_str!("{}##preview", label), &mut preview_buffer)
            .read_only(true)
            .build();
        alpha_token.pop(&ui);
        width_token.pop(&ui);
        spacing_token.pop(&ui);
    }
}
