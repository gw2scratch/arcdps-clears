use arcdps::imgui::{ChildWindow, Condition, im_str, PopupModal, Selectable, StyleVar, TabBar, TabItem, TableFlags, Ui, Window};
use chrono::Utc;

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
                        ui.calc_text_size(&get_api_key_name(api_key, tr))[0]
                    })
                    .fold(0.0, f32::max);
                let left_pane_width = max_name_width.max(ui.current_font_size() * 10.0);

                if let _t = ui.begin_group() {
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
                    if ui.button(&tr.im_string("api-key-add-new-button")) {
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
                        .build(&ui, || {
                            let selected_key = match ui_state.api_key_window.selected_key {
                                SelectedApiKey::None => None,
                                SelectedApiKey::Id(id) => settings.get_key_mut(&id)
                            };
                            if let Some(key) = selected_key {
                                let mut key_text = key.key().to_string();
                                let mut key_changed = false;

                                // This has to be a password box to protect streamers and users
                                // who may have others watching them.
                                if ui.input_text(&tr.im_string("api-key-key-label"), &mut key_text)
                                    .password(true)
                                    .build() {
                                    key_changed = true;
                                }
                                if key.data().account_data().is_none() && key.data().token_info().is_none() {
                                    if ui.button(&tr.im_string("api-key-check-api-key-button")) {
                                        let sender = bg_workers.api_sender();
                                        sender.send(ApiJob::UpdateAccountData(*key.id()));
                                        sender.send(ApiJob::UpdateTokenInfo(*key.id()));
                                        sender.send(ApiJob::UpdateClears(*key.id()));
                                    }
                                    ui.separator();
                                    ui.text_wrapped(&tr.im_string("api-key-guide-step1-prefix"));
                                    ui.same_line();
                                    if ui.small_button(&tr.im_string("api-key-guide-step1-open")) {
                                        open::that("https://account.arena.net/applications/create");
                                    }
                                    ui.same_line();
                                    ui.text(tr.im_string("api-key-guide-step1-url"));
                                    ui.text_wrapped(&tr.im_string("api-key-guide-step2"));
                                    ui.text_wrapped(&tr.im_string("api-key-guide-step3"));
                                    ui.text_wrapped(&tr.im_string("api-key-guide-step4"));
                                    ui.text_wrapped(&tr.im_string("api-key-guide-step5"));
                                }

                                ui.separator();
                                TabBar::new(im_str!("api_key_tabs")).build(&ui, || {
                                    TabItem::new(&tr.im_string("api-key-details-tab-details"))
                                        .build(&ui, || {
                                            if key.data().account_data().is_some() || key.data().token_info().is_some() {
                                                if let Some(_t) = ui.begin_table_with_flags(im_str!("ApiKeyData"), 2, TableFlags::SIZING_FIXED_FIT) {
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
                                                        if let _disabled = ui.push_style_var(StyleVar::Alpha(0.75)) {
                                                            if let _width = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0])) {
                                                                ui.checkbox(im_str!("##Account"), &mut account);
                                                            }
                                                        }
                                                        if ui.is_item_hovered() {
                                                            ui.tooltip(|| {
                                                                ui.text(tr.im_string("api-key-details-permission-account"))
                                                            });
                                                        }
                                                        ui.same_line();

                                                        // Progression permission checkmark
                                                        if let _disabled = ui.push_style_var(StyleVar::Alpha(0.75)) {
                                                            if let _width = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0])) {
                                                                ui.checkbox(im_str!("##Progression"), &mut progression);
                                                            }
                                                        }
                                                        if ui.is_item_hovered() {
                                                            ui.tooltip(|| {
                                                                ui.text(tr.im_string("api-key-details-permission-progression"))
                                                            });
                                                        }

                                                        // Extra permission indicator
                                                        if extra_perms.len() > 0 {
                                                            ui.same_line();
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
                                                }
                                            }

                                            // Missing permission/access warnings
                                            if let Some(token_info) = key.data().token_info() {
                                                if !token_info.has_permission("account") {
                                                    ui.text_colored(WARNING_RED, tr.im_string("api-key-warning-permission-account-missing"));
                                                }
                                                if !token_info.has_permission("progression") {
                                                    ui.text_colored(WARNING_RED, tr.im_string("api-key-warning-permission-progression-missing"));
                                                }

                                                if let TokenType::Subtoken { urls, expires_at, .. } = token_info.token_type() {
                                                    if *expires_at < Utc::now() {
                                                        ui.text_colored(WARNING_RED, tr.im_string("api-key-warning-subtoken-expired"));
                                                    }
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
                                                key.change_key(&key_text);
                                            }
                                        });
                                    TabItem::new(&tr.im_string("api-key-details-tab-friends"))
                                        .build(&ui, || {
                                            let key_usable = match friends::get_key_usability(key) {
                                                KeyUsability::NoTokenInfo => {
                                                    ui.text_colored(WARNING_RED, tr.im_string("api-key-friends-warning-no-token-info"));
                                                    false
                                                }
                                                KeyUsability::Usable => true,
                                                KeyUsability::InsufficientPermissions => {
                                                    ui.text_colored(WARNING_RED, tr.im_string("api-key-friends-warning-no-permissions"));
                                                    false
                                                }
                                                KeyUsability::InsufficientSubtokenUrls(urls) => {
                                                    ui.text_colored(WARNING_RED, tr.im_string("api-key-friends-warning-subtoken-missing-urls"));
                                                    for url in urls {
                                                        ui.text_colored(WARNING_RED, im_str!("\t{}", url));
                                                    }
                                                    false
                                                }
                                                KeyUsability::SubtokenExpired => {
                                                    ui.text_colored(WARNING_RED, tr.im_string("api-key-friends-warning-subtoken-expired"));
                                                    false
                                                }
                                            };

                                            if !key_usable {
                                                return;
                                            }

                                            if data.friends.api_state().is_none() {
                                                ui.text_colored(WARNING_RED, tr.im_string("friends-no-connection-to-server"));
                                                refresh_button(ui, ui_state, bg_workers, tr);
                                            }

                                            if let Some(state) = data.friends.api_state().and_then(|x| x.key_state(key)) {
                                                if state.shared_to().len() == 0 {
                                                    ui.text_wrapped(&tr.im_string("api-key-friends-intro"));
                                                }

                                                let original_public = state.public();
                                                let mut public = state.public();
                                                // TODO: Translate
                                                if let _padding = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0])) {
                                                    ui.radio_button(im_str!("Public"), &mut public, true);
                                                    ui.radio_button(im_str!("Friends only"), &mut public, false);
                                                }

                                                if public != original_public {
                                                    bg_workers.api_sender().send(ApiJob::SetKeyPublicFriend {
                                                        key_uuid: *key.id(),
                                                        public
                                                    });
                                                }


                                                if !state.public() {
                                                    if let Some(_t) = ui.begin_table_with_flags(im_str!("ApiKeyFriendsTable"), 2, TableFlags::SIZING_FIXED_FIT | TableFlags::SCROLL_Y) {
                                                        for share in state.shared_to() {
                                                            ui.table_next_row();
                                                            ui.table_next_column();
                                                            ui.text(share.account());
                                                            if !share.account_available() {
                                                                utils::warning_marker(&ui, tr.im_string("api-key-friends-warning-unknown-user"));
                                                            }
                                                            ui.table_next_column();
                                                            if ui.small_button(&im_str!("Unshare##{}", share.account())) {
                                                                bg_workers.api_sender().send(ApiJob::UnshareKeyWithFriend {
                                                                    key_uuid: *key.id(),
                                                                    friend_account_name: share.account().to_string(),
                                                                });
                                                            }
                                                        }

                                                        ui.table_next_row();
                                                        ui.table_next_column();

                                                        let width = ui.push_item_width(ui.current_font_size() * 20.0);
                                                        let padding = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
                                                        let mut add = ui.input_text(im_str!("##add_new_friend_name"), &mut ui_state.api_key_window.new_friend_name)
                                                            .hint(im_str!("Account Name.1234")) // TODO: Test ingame to see if it's visible, might need a tooltip
                                                            .enter_returns_true(true)
                                                            .build();
                                                        width.pop(ui);
                                                        padding.pop();

                                                        ui.table_next_column();
                                                        add = add || ui.small_button(&tr.im_string("api-key-friends-share-button"));

                                                        if add {
                                                            bg_workers.api_sender().send(ApiJob::ShareKeyWithFriend {
                                                                key_uuid: *key.id(),
                                                                friend_account_name: ui_state.api_key_window.new_friend_name.to_string(),
                                                            });
                                                            ui_state.api_key_window.new_friend_name.clear();
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                });
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
                        if ui.button(&tr.im_string("api-key-remove-key-button")) {
                            ui.open_popup(&popup_label);
                        }
                        PopupModal::new(&popup_label)
                            .save_settings(false)
                            .build(ui, || {
                                ui.text(tr.im_string("api-key-remove-modal-warning"));
                                ui.separator();
                                if let Some(_t) = ui.begin_table_with_flags(im_str!("DeleteConfirmationPopupTable"), 2, TableFlags::SIZING_STRETCH_SAME) {
                                    ui.table_next_row();
                                    ui.table_next_column();
                                    if ui.button_with_size(&tr.im_string("api-key-remove-modal-confirm"), [ui.current_column_width(), 0.0]) {
                                        settings.remove_key(&uuid);
                                        ui.close_current_popup();
                                    }
                                    ui.set_item_default_focus();
                                    ui.table_next_column();
                                    if ui.button_with_size(&tr.im_string("api-key-remove-modal-cancel"), [ui.current_column_width(), 0.0]) {
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