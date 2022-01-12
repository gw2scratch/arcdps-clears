use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

use arcdps::imgui::{Direction, MenuItem, MouseButton, StyleVar, TableFlags, Ui, Window};
use log::warn;


use crate::{Data, guide};
use crate::settings::{Friend, Settings};
use crate::translations::Translation;
use crate::ui::{settings, UiState, utils};
use crate::ui::clears::{clears_table, ClearTableEntry};
use crate::ui::style::WARNING_RED;
use crate::workers::{ApiJob, BackgroundWorkers};

pub fn friends(
    ui: &Ui,
    ui_state: &mut UiState,
    data: &Data,
    bg_workers: &BackgroundWorkers,
    settings: &mut Settings,
    tr: &Translation,
) {
    if !settings.friends.enabled {
        let publicize_text = match settings.api_keys.len() {
            0 => None,
            1 => Some(tr.translate("friends-enable-set-keys-public-single")),
            _ => Some(tr.translate("friends-enable-set-keys-public-multiple")),
        };

        let enable_button_width = ui.text_line_height() * 25.0;

        if ui.button_with_size(tr.translate("friends-enable-button"), [enable_button_width, ui.text_line_height() * 2.0]) {
            if ui_state.friends_window.enable_set_all_keys_public {
                let sender = bg_workers.api_sender();
                thread::spawn(move || {
                    sleep(Duration::from_secs(60));
                    // We are really hoping that all subtokens are uploaded
                    // already so everything can be set properly.
                    // We are also hoping the game is not closed before this finishes.

                    // There currently isn't any good way to enqueue this so that it
                    // only happens *after* the submission of all subtokens.

                    sender.send(ApiJob::SetAllKeysPublicFriend { public: true });
                });
            }

            settings.friends.enabled = true;
            bg_workers.api_sender().send(ApiJob::UpdateFriendState);
        }

        if let Some(text) = publicize_text {
            let _padding = ui.push_style_var(StyleVar::FramePadding([1.0, 1.0]));
            ui.checkbox(text, &mut ui_state.friends_window.enable_set_all_keys_public);
            ui.same_line();
            utils::help_marker(ui, tr.translate("friends-enable-set-keys-public-description"));
        }

        ui.spacing();

        // Right-aligned button with reduced padding
        const X_FRAME_PADDING: f32 = 1.0;
        const Y_FRAME_PADDING: f32 = 2.0;
        let privacy_text = tr.translate("friends-privacy-button");
        let x_offset = enable_button_width - ui.calc_text_size(&privacy_text)[0] - 2.0 * X_FRAME_PADDING;

        let _padding = ui.push_style_var(StyleVar::FramePadding([X_FRAME_PADDING, Y_FRAME_PADDING]));
        let _spacing = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));
        ui.dummy([x_offset, 0.0]);
        ui.same_line();
        if ui.button(privacy_text) {
            let _ = open::that(guide::FRIEND_PRIVACY_URL);
        }
        if ui.is_item_hovered() {
            ui.tooltip_text(tr.translate("tooltip-opens-in-a-browser"));
        }
    } else {
        if data.friends.api_state().is_none() {
            ui.text_colored(WARNING_RED, tr.translate("friends-no-connection-to-server"));
            refresh_button(ui, ui_state, bg_workers, tr);
        } else {
            let mut entries: Vec<_> = settings.friends.list.friends_mut().iter_mut()
                .filter(|friend| friend.show_in_friends())
                .filter(|friend| data.friends.state_available(friend.account_name()))
                .map(|friend| ClearTableEntry {
                    account_name: friend.account_name().to_string(),
                    state: data.friends.finished_encounters(friend.account_name()),
                    expanded: friend.expanded_in_friends_mut(),
                })
                .collect();

            if let Some(raids) = data.clears.raids() {
                if entries.is_empty() {
                    let wrap = ui.push_text_wrap_pos_with_pos(ui.current_font_size() * 25.0);
                    ui.text_wrapped(&tr.translate("friends-intro"));
                    ui.text("");
                    wrap.pop(ui);
                } else {
                    clears_table(ui, raids, &mut entries, &settings.friends_clears_style, settings.short_names, tr, || {
                        utils::centered_text(&ui, &tr.translate("friends-no-data-available"));
                        ui.text("");

                        let time = *bg_workers.api_refresher_next_wakeup().lock().unwrap();
                        let until_wakeup = time.saturating_duration_since(Instant::now());
                        utils::centered_text(
                            &ui,
                            format!("{}{}{}", tr.translate("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.translate("next-refresh-secs-suffix")),
                        );
                    });
                }
            } else {
                // TODO: Deduplicate
                ui.text(tr.translate("clears-no-public-data-yet"));
                ui.text("");

                let time = *bg_workers.api_refresher_next_wakeup().lock().unwrap();
                let until_wakeup = time.saturating_duration_since(Instant::now());
                utils::centered_text(
                    &ui,
                    format!("{}{}{}", tr.translate("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.translate("next-refresh-secs-suffix")),
                );
            }

            let friends_window_button_text = if entries.is_empty() {
                tr.translate("friends-friendlist-button-no-friends")
            } else {
                tr.translate("friends-friendlist-button")
            };

            if ui.button(friends_window_button_text) {
                ui_state.friends_window.shown = true;
            }
            ui.same_line();
            if ui.button(tr.translate("friends-share-button")) {
                ui_state.api_key_window.shown = true;
            }
        }

        if ui.is_mouse_released(MouseButton::Right) && ui.is_window_hovered() {
            ui.open_popup("##RightClickMenuFriendsClears");
        }

        ui.popup("##RightClickMenuFriendsClears", || {
            if let _small_frame_padding = ui.push_style_var(StyleVar::FramePadding([1.0, 1.0])) {
                ui.menu(&tr.translate("friends-contextmenu-friend-list"), || {
                    let entries: Vec<_> = settings.friends.list.friends_mut().iter_mut()
                        .filter(|friend| data.friends.state_available(friend.account_name()))
                        .collect();

                    for friend in entries {
                        if MenuItem::new(format!("{}##FriendsContextCheckbox", friend.account_name()))
                            .selected(friend.show_in_friends())
                            .build(ui) {
                            *friend.show_in_friends_mut() = !friend.show_in_friends();
                        }
                    }
                });
                ui.separator();
                settings::style_section(ui, "friends-style-tooltip", &mut settings.friends_clears_style, tr);
            }
        })
    }
}

pub fn friends_window(
    ui: &Ui,
    ui_state: &mut UiState,
    data: &Data,
    bg_workers: &BackgroundWorkers,
    settings: &mut Settings,
    tr: &Translation,
) {
    if ui_state.friends_window.shown && settings.friends.enabled {
        let mut shown = ui_state.friends_window.shown;
        Window::new(&tr.translate("friends-window-title"))
            .always_auto_resize(true)
            .focus_on_appearing(false)
            .no_nav()
            .collapsible(false)
            .opened(&mut shown)
            .build(ui, || {
                if settings.friends.list.friends().iter().any(|friend| data.friends.state_available(friend.account_name())) {
                    if let Some(_t) = ui.begin_table_with_flags("FriendsTable", 5, TableFlags::BORDERS) {
                        ui.table_setup_column("##updown");
                        ui.table_setup_column(&tr.translate("friends-friendlist-account-name"));
                        ui.table_setup_column(&tr.translate("friends-friendlist-shown"));
                        ui.table_setup_column("##remove");
                        ui.table_setup_column("##status");
                        ui.table_headers_row();

                        let mut swap = None;
                        let mut removal = None;

                        let shown_indices: Vec<_> = settings.friends.list.friends().iter()
                            .map(|friend| data.friends.state_available(friend.account_name()))
                            .collect();

                        for (i, friend) in settings.friends.list.friends_mut().iter_mut().enumerate() {
                            // Hide currently unavailable friends, but do not remove them
                            if !shown_indices[i] {
                                continue;
                            }

                            ui.table_next_row();
                            ui.table_next_column();

                            // To implement moving, we need to find previous non-hidden friend and
                            // its index in the non-filtered friend list.
                            let prev = (0..i).rev().filter(|prev_i| shown_indices[*prev_i]).next();
                            let next = (i + 1..shown_indices.len()).filter(|next_i| shown_indices[*next_i]).next();

                            if let _padding = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0])) {
                                if let Some(prev_i) = prev {
                                    if ui.arrow_button(format!("##friend_up_{}", friend.account_name()), Direction::Up) {
                                        swap = Some((prev_i, i));
                                    };
                                } else {
                                    ui.invisible_button(format!("##friend_up_{}", friend.account_name()), [ui.frame_height(), ui.frame_height()]);
                                }
                                ui.same_line();
                                if let Some(next_i) = next {
                                    if ui.arrow_button(format!("##friend_down_{}", friend.account_name()), Direction::Down) {
                                        swap = Some((next_i, i));
                                    }
                                } else {
                                    ui.invisible_button(format!("##friend_down_{}", friend.account_name()), [ui.frame_height(), ui.frame_height()]);
                                }
                            }

                            ui.table_next_column();
                            ui.text(friend.account_name());

                            ui.table_next_column();
                            if let _padding = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0])) {
                                let current_x = ui.cursor_pos()[0];
                                let checkbox_width = ui.frame_height();
                                let column_width = ui.current_column_width();
                                let new_x = (current_x + column_width / 2. - checkbox_width / 2.).max(current_x);

                                ui.set_cursor_pos([new_x, ui.cursor_pos()[1]]);
                                ui.checkbox(&format!("##friend_show_{}", friend.account_name()), friend.show_in_friends_mut());

                                ui.table_next_column();
                                if ui.button(format!("{}##friend_{}", tr.translate("friends-friendlist-remove"), friend.account_name())) {
                                    removal = Some(i);
                                }
                            }
                            ui.table_next_column();

                            if !data.friends.state_available(friend.account_name()) {
                                ui.text(tr.translate("friends-friendlist-error-no-data"))
                            }
                            if let Some(known) = data.friends.known(friend.account_name()) {
                                if !known {
                                    ui.text(tr.translate("friends-friendlist-error-not-known"))
                                }
                            }
                        }

                        if let Some((index1, index2)) = swap {
                            settings.friends.list.friends_mut().swap(index1, index2);
                        }

                        if let Some(i) = removal {
                            settings.friends.list.friends_mut().remove(i);
                        }
                    }
                } else {
                    let wrap = ui.push_text_wrap_pos_with_pos(ui.current_font_size() * 30.0);
                    ui.text_wrapped(&tr.translate("friends-friendlist-intro"));
                    wrap.pop(ui);
                    ui.spacing();
                }

                refresh_button(ui, ui_state, bg_workers, tr);
                ui.same_line();

                ui.popup("add-friend-popup", || {
                    let mut add = ui.input_text(tr.translate("friends-friendlist-add-account-name"), &mut ui_state.friends_window.new_friend_name)
                        .enter_returns_true(true)
                        .build();

                    add = add || ui.button(tr.translate("friends-friendlist-add-add"));

                    if add {
                        // Make sure that duplicates are not added.
                        if settings.friends.list.get(&ui_state.friends_window.new_friend_name).is_none() {
                            settings.friends.list.add(Friend::new(ui_state.friends_window.new_friend_name.clone(), true));
                            if let Err(_) = bg_workers.api_sender().send(ApiJob::UpdateFriendState) {
                                warn!("Failed to send request to API worker");
                            }
                        }
                        ui_state.friends_window.new_friend_name.clear();
                        ui.close_current_popup();
                    }
                    ui.same_line();
                    if ui.button(tr.translate("friends-friendlist-add-close")) {
                        ui.close_current_popup();
                    }
                });

                if ui.button(tr.translate("friends-friendlist-button-add")) {
                    ui.open_popup("add-friend-popup")
                }
            });

        ui_state.friends_window.shown = shown;
    }
}

pub fn refresh_button(ui: &Ui, ui_state: &mut UiState, bg_workers: &BackgroundWorkers, tr: &Translation) {
    // We have a cooldown here to avoid spamming the request too much
    // and to make it feel like the button is doing something.
    if Instant::now().saturating_duration_since(ui_state.friends_window.last_refresh_use) > Duration::from_secs(2) {
        if ui.button(&tr.translate("friends-refresh-button")) {
            if let Err(_) = bg_workers.api_sender().send(ApiJob::UpdateFriendState) {
                warn!("Failed to send request to API worker");
            }
            ui_state.friends_window.last_refresh_use = Instant::now();
        }
    } else {
        if let _disabled = ui.push_style_var(StyleVar::Alpha(0.6)) {
            ui.button(&tr.translate("friends-refresh-button"));
        }
    }
}
