use arcdps::imgui::{ChildWindow, Condition, PopupModal, Selectable, StyleVar, TabBar, TabItem, TableFlags, Ui, Window};
use chrono::Utc;
use log::warn;

use crate::{Data, friends};
use crate::friends::KeyUsability;
use crate::settings::{ApiKey, Settings, TokenType};
use crate::translations::Translation;
use crate::ui::{get_api_key_name, SelectedApiKey, UiState, utils};
use crate::ui::friends::refresh_button;
use crate::ui::style::WARNING_RED;
use crate::workers::{ApiJob, BackgroundWorkers};

pub fn api_keys_window(
    ui: &Ui,
    ui_state: &mut UiState,
    data: &Data,
    bg_workers: &BackgroundWorkers,
    settings: &mut Settings,
    tr: &Translation,
) {
    if ui_state.api_key_window.shown {
        let mut shown = ui_state.api_key_window.shown;
        Window::new(tr.translate("api-key-window-title"))
            .size([ui.current_font_size() * 40.0, ui.current_font_size() * 25.0], Condition::FirstUseEver)
            .resizable(true)
            .focus_on_appearing(false)
            .no_nav()
            .collapsible(false)
            .opened(&mut shown)
            .build(ui, || {
                // We copy this here as we will be doing borrows later on and settings won't be
                // available when this value is needed.
                let friends_enabled = settings.friends.enabled;

                let max_name_width = settings.api_keys.iter()
                    .map(|api_key| {
                        ui.calc_text_size(&get_api_key_name(api_key, tr))[0]
                    })
                    .fold(0.0, f32::max);
                let left_pane_width = max_name_width.max(ui.current_font_size() * 10.0);

                if let _t = ui.begin_group() {
                    ChildWindow::new("ApiLeftPane")
                        // The -frame_height is to make space for buttons at the bottom.
                        .size([left_pane_width, -ui.frame_height_with_spacing()])
                        .build(ui, || {
                            for api_key in settings.api_keys.iter() {
                                let name = get_api_key_name(api_key, tr);
                                if Selectable::new(format!("{}##{}", name, api_key.id().to_string()))
                                    .selected(ui_state.api_key_window.is_key_selected(api_key))
                                    .build(ui) {
                                    ui_state.api_key_window.selected_key = SelectedApiKey::Id(*api_key.id());
                                }
                            }
                        });
                    if ui.button(&tr.translate("api-key-add-new-button")) {
                        let new_key = ApiKey::new("");
                        ui_state.api_key_window.selected_key = SelectedApiKey::Id(*new_key.id());
                        settings.api_keys.push(new_key);
                    }
                }
                ui.same_line();
                if let _t = ui.begin_group() {
                    ChildWindow::new("ApiRightPane")
                        // The -frame_height is to make space for buttons at the bottom.
                        .size([0.0, -ui.frame_height_with_spacing()])
                        .build(ui, || {
                            let selected_key = match ui_state.api_key_window.selected_key {
                                SelectedApiKey::None => None,
                                SelectedApiKey::Id(id) => settings.get_key_mut(&id)
                            };
                            if let Some(key) = selected_key {
                                let mut key_text = key.key().to_string();
                                let mut key_changed = false;

                                // This has to be a password box to protect streamers and users
                                // who may have others watching them.
                                if ui.input_text(&tr.translate("api-key-key-label"), &mut key_text)
                                    .password(true)
                                    .build() {
                                    key_changed = true;
                                }
                                if key.data().account_data().is_none() && key.data().token_info().is_none() {
                                    if ui.button(&tr.translate("api-key-check-api-key-button")) {
                                        let sender = bg_workers.api_sender();
                                        if let Err(_) = sender.send(ApiJob::UpdateAccountData(*key.id())) {
                                            warn!("Failed to send request to API worker");
                                        }
                                        if let Err(_) = sender.send(ApiJob::UpdateTokenInfo(*key.id())) {
                                            warn!("Failed to send request to API worker");
                                        }
                                        if let Err(_) = sender.send(ApiJob::UpdateClears(*key.id())) {
                                            warn!("Failed to send request to API worker");
                                        }
                                    }
                                    ui.separator();
                                    ui.text_wrapped(&tr.translate("api-key-guide-step1-prefix"));
                                    ui.same_line();
                                    utils::small_url_button(ui, tr.translate("api-key-guide-step1-open"), "https://account.arena.net/applications/create", tr);
                                    ui.same_line();
                                    ui.text(tr.translate("api-key-guide-step1-url"));
                                    ui.text_wrapped(&tr.translate("api-key-guide-step2"));
                                    ui.text_wrapped(&tr.translate("api-key-guide-step3"));
                                    ui.text_wrapped(&tr.translate("api-key-guide-step4"));
                                    ui.text_wrapped(&tr.translate("api-key-guide-step5"));
                                }

                                ui.separator();
                                TabBar::new("api_key_tabs").build(ui, || {
                                    TabItem::new(&tr.translate("api-key-details-tab-details"))
                                        .build(ui, || {
                                            if key.data().account_data().is_some() || key.data().token_info().is_some() {
                                                if let Some(_t) = ui.begin_table_with_flags("ApiKeyData", 2, TableFlags::SIZING_FIXED_FIT) {
                                                    ui.table_next_row();
                                                    ui.table_next_column();
                                                    ui.text(tr.translate("api-key-details-account-name"));
                                                    ui.table_next_column();
                                                    if let Some(name) = key.data().account_data().as_ref().map(|x| x.name()) {
                                                        ui.text(name);
                                                    } else {
                                                        ui.text(tr.translate("api-key-details-unknown-value"));
                                                    }
                                                    ui.table_next_row();
                                                    ui.table_next_column();
                                                    ui.text(tr.translate("api-key-details-key-name"));
                                                    ui.table_next_column();
                                                    if let Some(name) = key.data().token_info().as_ref().map(|x| x.name()) {
                                                        ui.text(name);
                                                    } else {
                                                        ui.text(tr.translate("api-key-details-unknown-value"));
                                                    }
                                                    ui.table_next_row();
                                                    ui.table_next_column();
                                                    ui.text(tr.translate("api-key-details-permissions"));
                                                    ui.table_next_column();
                                                    if let Some(token_info) = key.data().token_info() {
                                                        let mut account = token_info.has_permission("account");
                                                        let mut progression = token_info.has_permission("progression");

                                                        let extra_perms: Vec<_> = token_info.permissions().iter()
                                                            .filter(|x| *x != "account" && *x != "progression")
                                                            .collect();

                                                        // Account permission checkmark
                                                        if let _disabled = ui.push_style_var(StyleVar::Alpha(0.75)) {
                                                            if let _width = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0])) {
                                                                ui.checkbox("##Account", &mut account);
                                                            }
                                                        }
                                                        if ui.is_item_hovered() {
                                                            ui.tooltip(|| {
                                                                ui.text(tr.translate("api-key-details-permission-account"))
                                                            });
                                                        }
                                                        ui.same_line();

                                                        // Progression permission checkmark
                                                        if let _disabled = ui.push_style_var(StyleVar::Alpha(0.75)) {
                                                            if let _width = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0])) {
                                                                ui.checkbox("##Progression", &mut progression);
                                                            }
                                                        }
                                                        if ui.is_item_hovered() {
                                                            ui.tooltip(|| {
                                                                ui.text(tr.translate("api-key-details-permission-progression"))
                                                            });
                                                        }

                                                        // Extra permission indicator
                                                        if !extra_perms.is_empty() {
                                                            ui.same_line();
                                                            ui.text(format!("{}{}{}",
                                                                            tr.translate("api-key-details-permissions-extra-prefix"),
                                                                            extra_perms.len(),
                                                                            tr.translate("api-key-details-permissions-extra-suffix")));
                                                            if ui.is_item_hovered() {
                                                                ui.tooltip(|| {
                                                                    for extra_perm in extra_perms {
                                                                        ui.text(extra_perm);
                                                                    }
                                                                });
                                                            }
                                                        }
                                                    } else {
                                                        ui.text(tr.translate("api-key-details-unknown-value"));
                                                    }
                                                    ui.table_next_row();
                                                    ui.table_next_column();
                                                    ui.text(tr.translate("api-key-details-key-type"));
                                                    ui.table_next_column();
                                                    let key_type = key.data().token_info().as_ref().map(|x| x.token_type());
                                                    if let Some(TokenType::ApiKey) = key_type {
                                                        ui.text(tr.translate("api-key-details-key-type-apikey"));
                                                    } else if let Some(TokenType::Subtoken { expires_at, issued_at, .. }) = key_type {
                                                        ui.text(tr.translate("api-key-details-key-type-subtoken"));
                                                        ui.table_next_row();
                                                        ui.table_next_column();
                                                        ui.text(tr.translate("api-key-details-subtoken-issued-at"));
                                                        ui.table_next_column();
                                                        ui.text(issued_at.to_rfc2822());
                                                        ui.table_next_row();
                                                        ui.table_next_column();
                                                        ui.text(tr.translate("api-key-details-subtoken-expires-at"));
                                                        ui.table_next_column();
                                                        ui.text(expires_at.to_rfc2822());
                                                    } else {
                                                        ui.text(tr.translate("api-key-details-unknown-value"));
                                                    }
                                                }
                                            }

                                            // Missing permission/access warnings
                                            if let Some(token_info) = key.data().token_info() {
                                                if !token_info.has_permission("account") {
                                                    ui.text_colored(WARNING_RED, tr.translate("api-key-warning-permission-account-missing"));
                                                }
                                                if !token_info.has_permission("progression") {
                                                    ui.text_colored(WARNING_RED, tr.translate("api-key-warning-permission-progression-missing"));
                                                }

                                                if let TokenType::Subtoken { urls, expires_at, .. } = token_info.token_type() {
                                                    if *expires_at < Utc::now() {
                                                        ui.text_colored(WARNING_RED, tr.translate("api-key-warning-subtoken-expired"));
                                                    }
                                                    if let Some(urls) = urls {
                                                        let account_access = urls.iter().any(|x| x == "/v2/account");
                                                        let clears_access = urls.iter().any(|x| x == "/v2/account/raids");
                                                        let masteries_access = urls.iter().any(|x| x == "/v2/account/masteries");
                                                        if !account_access {
                                                            ui.text_colored(WARNING_RED, tr.translate("api-key-warning-subtoken-url-missing-account"));
                                                        }
                                                        if !clears_access {
                                                            ui.text_colored(WARNING_RED, tr.translate("api-key-warning-subtoken-url-missing-account-raids"));
                                                        }
                                                        if !masteries_access {
                                                            ui.text_colored(WARNING_RED, tr.translate("api-key-warning-subtoken-url-missing-account-masteries"));
                                                        }
                                                    }
                                                }
                                            }

                                            ui.separator();
                                            if ui.checkbox(&tr.translate("api-key-show-in-my-clears-checkbox"), key.show_key_in_clears_mut()) {
                                                if key.show_key_in_clears() {
                                                    if let Err(_) = bg_workers.api_sender().send(ApiJob::UpdateClears(*key.id())) {
                                                        warn!("Failed to send request to API worker");
                                                    }
                                                }
                                            }

                                            if key_changed {
                                                key.change_key(&key_text);
                                            }
                                        });
                                    TabItem::new(&tr.translate("api-key-details-tab-friends"))
                                        .build(ui, || {
                                            // Check enabled
                                            if !friends_enabled {
                                                let _wrap = ui.push_text_wrap_pos();
                                                ui.text_colored(WARNING_RED, tr.translate("api-key-friends-disabled"));
                                            } else {
                                                let key_usable = match friends::get_key_usability(key) {
                                                    KeyUsability::NoTokenInfo => {
                                                        ui.text_colored(WARNING_RED, tr.translate("api-key-friends-warning-no-token-info"));
                                                        false
                                                    }
                                                    KeyUsability::Usable => true,
                                                    KeyUsability::InsufficientPermissions => {
                                                        ui.text_colored(WARNING_RED, tr.translate("api-key-friends-warning-no-permissions"));
                                                        false
                                                    }
                                                    KeyUsability::InsufficientSubtokenUrls(urls) => {
                                                        ui.text_colored(WARNING_RED, tr.translate("api-key-friends-warning-subtoken-missing-urls"));
                                                        for url in urls {
                                                            ui.text_colored(WARNING_RED, format!("\t{}", url));
                                                        }
                                                        false
                                                    }
                                                    KeyUsability::SubtokenExpired => {
                                                        ui.text_colored(WARNING_RED, tr.translate("api-key-friends-warning-subtoken-expired"));
                                                        false
                                                    }
                                                };

                                                if !key_usable {
                                                    return;
                                                }

                                                if data.friends.api_state().is_none() {
                                                    ui.text_colored(WARNING_RED, tr.translate("friends-no-connection-to-server"));
                                                    refresh_button(ui, ui_state, bg_workers, tr);
                                                }

                                                if let Some(state) = data.friends.api_state().and_then(|x| x.key_state(key)) {
                                                    ui.text_wrapped(&tr.translate("api-key-friends-intro"));

                                                    let original_public = state.public();
                                                    let mut public = state.public();
                                                    if let _padding = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0])) {
                                                        ui.radio_button(tr.translate("api-key-friends-share-public"), &mut public, true);
                                                        ui.same_line();
                                                        utils::help_marker(ui, tr.translate("api-key-friends-share-public-description"));
                                                        ui.radio_button(tr.translate("api-key-friends-share-friends"), &mut public, false);
                                                        ui.same_line();
                                                        utils::help_marker(ui, tr.translate("api-key-friends-share-friends-description"));
                                                    }

                                                    if public != original_public {
                                                        if let Err(_) = bg_workers.api_sender().send(ApiJob::SetKeyPublicFriend {
                                                            key_uuid: *key.id(),
                                                            public,
                                                        }) {
                                                            warn!("Failed to send request to API worker");
                                                        }
                                                    }


                                                    if !state.public() {
                                                        if let Some(_t) = ui.begin_table_with_flags("ApiKeyFriendsTable", 2, TableFlags::SIZING_FIXED_FIT | TableFlags::SCROLL_Y) {
                                                            for share in state.shared_to() {
                                                                ui.table_next_row();
                                                                ui.table_next_column();
                                                                ui.text(share.account());
                                                                if !share.account_available() {
                                                                    utils::warning_marker(ui, tr.translate("api-key-friends-warning-unknown-user"));
                                                                }
                                                                ui.table_next_column();
                                                                if ui.small_button(format!("{}##{}", tr.translate("api-key-friends-unshare-button"), share.account())) {
                                                                    if let Err(_) = bg_workers.api_sender().send(ApiJob::UnshareKeyWithFriend {
                                                                        key_uuid: *key.id(),
                                                                        friend_account_name: share.account().to_string(),
                                                                    }) {
                                                                        warn!("Failed to send request to API worker");
                                                                    }
                                                                }
                                                            }

                                                            ui.table_next_row();
                                                            ui.table_next_column();

                                                            let width = ui.push_item_width(ui.current_font_size() * 20.0);
                                                            let padding = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
                                                            let mut add = ui.input_text("##add_new_friend_name", &mut ui_state.api_key_window.new_friend_name)
                                                                .hint("Account Name.1234") // TODO: Test ingame to see if it's visible, might need a tooltip
                                                                .enter_returns_true(true)
                                                                .build();
                                                            width.pop(ui);
                                                            padding.pop();

                                                            ui.table_next_column();
                                                            add = add || ui.small_button(&tr.translate("api-key-friends-share-button"));

                                                            if add {
                                                                if let Err(_) = bg_workers.api_sender().send(ApiJob::ShareKeyWithFriend {
                                                                    key_uuid: *key.id(),
                                                                    friend_account_name: ui_state.api_key_window.new_friend_name.to_string(),
                                                                }) {
                                                                    warn!("Failed to send request to API worker");
                                                                }
                                                                ui_state.api_key_window.new_friend_name.clear();
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                });
                            } else {
                                if settings.api_keys.is_empty() {
                                    ui.text_wrapped(&tr.translate("api-key-window-intro-first-key"));
                                } else {
                                    ui.text_wrapped(&tr.translate("api-key-window-intro"));
                                }
                            }
                        });
                    if let SelectedApiKey::Id(uuid) = ui_state.api_key_window.selected_key {
                        let popup_label = tr.translate("api-key-remove-modal-title");
                        if ui.button(&tr.translate("api-key-remove-key-button")) {
                            ui.open_popup(&popup_label);
                        }
                        PopupModal::new(&popup_label)
                            .save_settings(false)
                            .build(ui, || {
                                ui.text(tr.translate("api-key-remove-modal-warning"));
                                ui.separator();
                                if let Some(_t) = ui.begin_table_with_flags("DeleteConfirmationPopupTable", 2, TableFlags::SIZING_STRETCH_SAME) {
                                    ui.table_next_row();
                                    ui.table_next_column();
                                    if ui.button_with_size(&tr.translate("api-key-remove-modal-confirm"), [ui.current_column_width(), 0.0]) {
                                        settings.remove_key(&uuid);
                                        ui.close_current_popup();
                                    }
                                    ui.set_item_default_focus();
                                    ui.table_next_column();
                                    if ui.button_with_size(&tr.translate("api-key-remove-modal-cancel"), [ui.current_column_width(), 0.0]) {
                                        ui.close_current_popup();
                                    }
                                }
                            });
                    }
                }
            });

        ui_state.api_key_window.shown = shown;
    }
}