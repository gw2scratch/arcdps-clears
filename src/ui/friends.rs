use std::time::{Duration, Instant};

use arcdps::imgui::{CollapsingHeader, Direction, DragDropFlags, DragDropSource, DragDropTarget, MenuItem, MouseButton, PopupModal, Selectable, StyleColor, StyleVar, TableFlags, Ui, Window, WindowFlags};
use log::warn;

use crate::Data;
use crate::settings::Settings;
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
    if data.friends.api_state().is_none() {
        ui.text_colored(WARNING_RED, tr.translate("friends-no-connection-to-server"));
        refresh_button(ui, ui_state, bg_workers, tr);
    } else {
        if let Some(raids) = data.clears.raids() {
            let mut entries: Vec<_> = settings.friend_list.iter_mut()
                .filter(|friend| friend.show_in_friends())
                .filter(|friend| data.friends.state_available(friend.account_name()))
                .map(|friend| ClearTableEntry {
                    account_name: friend.account_name().to_string(),
                    state: data.friends.clears(friend.account_name()),
                    expanded: friend.expanded_in_friends_mut(),
                })
                .collect();

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

        if ui.button(&tr.translate("friends-friendlist-button")) {
            ui_state.friends_window.shown = true;
        }
        ui.same_line();
        if ui.button(&tr.translate("friends-share-button")) {
            ui_state.api_key_window.shown = true;
        }
    }

    if ui.is_mouse_released(MouseButton::Right) && ui.is_window_hovered() {
        ui.open_popup("##RightClickMenuFriendsClears");
    }

    ui.popup("##RightClickMenuFriendsClears", || {
        if let _small_frame_padding = ui.push_style_var(StyleVar::FramePadding([1.0, 1.0])) {
            ui.menu(&tr.translate("friends-contextmenu-friend-list"), || {
                let mut entries: Vec<_> = settings.friend_list.iter_mut()
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

pub fn friends_window(
    ui: &Ui,
    ui_state: &mut UiState,
    data: &Data,
    bg_workers: &BackgroundWorkers,
    settings: &mut Settings,
    tr: &Translation,
) {
    if ui_state.friends_window.shown {
        let mut shown = ui_state.friends_window.shown;
        Window::new(&tr.translate("friends-window-title"))
            .always_auto_resize(true)
            .focus_on_appearing(false)
            .no_nav()
            .collapsible(false)
            .opened(&mut shown)
            .build(ui, || {
                if settings.friend_list.iter_mut().any(|friend| data.friends.state_available(friend.account_name())) {
                    if let Some(_t) = ui.begin_table_with_flags("FriendsTable", 4, TableFlags::BORDERS) {
                        ui.table_setup_column("##updown");
                        ui.table_setup_column(&tr.translate("friends-friendlist-account-name"));
                        ui.table_setup_column(&tr.translate("friends-friendlist-shown"));
                        ui.table_setup_column("##remove");
                        ui.table_headers_row();

                        let mut swap = None;

                        let states_available: Vec<_> = settings.friend_list.iter()
                            .map(|friend| data.friends.state_available(friend.account_name()))
                            .collect();

                        for (i, mut friend) in settings.friend_list.iter_mut().enumerate() {
                            // Hide currently unavailable friends, but do not remove them
                            if !states_available[i] {
                                continue;
                            }

                            ui.table_next_row();
                            ui.table_next_column();

                            // To implement moving, we need to find previous non-hidden friend and
                            // its index in the non-filtered friend list.
                            let prev = (0..i).rev().filter(|prev_i| states_available[*prev_i]).next();
                            let next = (i + 1..states_available.len()).filter(|next_i| states_available[*next_i]).next();

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
                                if friend.public() {
                                    ui.button("Remove"); // TODO: Translate
                                } else {
                                    utils::help_marker(ui, "This friend shared their clears with you."); // TODO: Translate
                                }
                            }
                        }

                        if let Some((index1, index2)) = swap {
                            settings.friend_list.swap(index1, index2);
                        }

                    }
                } else {
                    let wrap = ui.push_text_wrap_pos_with_pos(ui.current_font_size() * 30.0);
                    ui.text_wrapped(&tr.translate("friends-friendlist-intro"));
                    wrap.pop(ui);
                    ui.new_line();
                }

                refresh_button(ui, ui_state, bg_workers, tr);

                // TODO: Translate
                // TODO: Implement
                ui.button("Add friend");
            });

        ui_state.friends_window.shown = shown;
    }
}

pub fn refresh_button(ui: &Ui, ui_state: &mut UiState, bg_workers: &BackgroundWorkers, tr: &Translation) {
    // We have a cooldown here to avoid spamming the request too much
    // and to make it feel like the button is doing something.
    if Instant::now().saturating_duration_since(ui_state.friends_window.last_refresh_use) > Duration::from_secs(2) {
        if ui.button(&tr.translate("friends-refresh-button")) {
            bg_workers.api_sender().send(ApiJob::UpdateFriendState);
            ui_state.friends_window.last_refresh_use = Instant::now();
        }
    } else {
        if let _disabled = ui.push_style_var(StyleVar::Alpha(0.6)) {
            ui.button(&tr.translate("friends-refresh-button"));
        }
    }
}
