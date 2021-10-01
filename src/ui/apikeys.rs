use arcdps::imgui::{im_str, Window, Condition, ChildWindow, StyleVar, TableFlags, ImString, Selectable, Ui, PopupModal};
use crate::ui::{get_api_key_name, SelectedApiKey, UiState};
use crate::workers::{ApiJob, BackgroundWorkers};
use crate::settings::{TokenType, ApiKey, Settings};
use crate::translations::Translation;
use crate::friends::KeyUsability;
use crate::friends;

pub fn api_keys_window(ui: &Ui, ui_state: &mut UiState, bg_workers: &BackgroundWorkers, settings: &mut Settings, tr: &Translation) {
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
                        for api_key in settings.api_keys.iter() {
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
                        let selected_key = match ui_state.api_key_window.selected_key {
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

                                // TODO: Move this close to friends share buttons etc.
                                match friends::get_key_usability(key) {
                                    KeyUsability::NoTokenInfo => {}
                                    KeyUsability::Usable => {}
                                    KeyUsability::InsufficientPermissions => {
                                        ui.text_colored(WARNING_RED, "Cannot be shared with friends, missing permissions!"); // TODO: Translate
                                    }
                                    KeyUsability::InsufficientSubtokenUrls => {
                                        // TODO: Add list of missing urls
                                        ui.text_colored(WARNING_RED, "Cannot be shared with friends, missing subtoken urls!"); // TODO: Translate
                                    }
                                    KeyUsability::SubtokenExpired => {
                                        ui.text_colored(WARNING_RED, "Cannot be shared with friends, subtoken is expired!"); // TODO: Translate
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
                    let popup_label = tr.im_string("api-key-remove-modal-title");
                    if ui.button(&tr.im_string("api-key-remove-key-button"), [0., 0.]) {
                        ui.open_popup(&popup_label);
                    }
                    PopupModal::new(&ui, &popup_label)
                        .save_settings(false)
                        .build(|| {
                            ui.text(tr.im_string("api-key-remove-modal-warning"));
                            ui.separator();
                            if ui.begin_table_with_flags(im_str!("DeleteConfirmationPopupTable"), 2, TableFlags::SIZING_STRETCH_SAME) {
                                ui.table_next_row();
                                ui.table_next_column();
                                if ui.button(&tr.im_string("api-key-remove-modal-confirm"), [ui.current_column_width(), 0.0]) {
                                    settings.remove_key(&uuid);
                                    ui.close_current_popup();
                                }
                                ui.set_item_default_focus();
                                ui.table_next_column();
                                if ui.button(&tr.im_string("api-key-remove-modal-cancel"), [ui.current_column_width(), 0.0]) {
                                    ui.close_current_popup();
                                }
                                ui.end_table();
                            }
                        });
                }
                group.end(&ui);
            });

        ui_state.api_key_window.shown = shown;
    }
}