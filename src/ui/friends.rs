use arcdps::imgui::{im_str, Ui, CollapsingHeader, StyleColor, Selectable, StyleVar, Direction};
use crate::translations::Translation;
use crate::ui::UiState;
use crate::Data;
use crate::workers::{BackgroundWorkers, ApiJob};
use crate::settings::Settings;
use crate::ui::style::WARNING_RED;
use log::warn;

pub fn friends(
    ui: &Ui,
    ui_state: &mut UiState,
    data: &Data,
    bg_workers: &BackgroundWorkers,
    settings: &mut Settings,
    tr: &Translation,
) {
    if data.friends.api_state().is_none() {
        ui.text_colored(WARNING_RED, "No connection to the friend server."); // TODO: Translate
        if ui.button(im_str!("Refresh"), [0.0, 0.0]) {
            bg_workers.api_sender().send(ApiJob::UpdateFriendState);
        }
    }

    if let Some(state) = data.friends.api_state() {
        ui.text("State available");
        ui.text(im_str!("keys: {}", state.keys().len()));
        for key in state.keys() {
            ui.text(im_str!("key {}", key.key_hash()));
            ui.text(im_str!("acc {:?}", key.account()));
            ui.text(im_str!("subtoken expires at {:?}", key.subtoken_expires_at()));
        }
        ui.text(im_str!("friends: {}", state.friends().len()));
        for friend in state.friends() {
            ui.text(im_str!("friend acc {}, shared with {}", friend.account(), friend.shared_with().join(",")));
        }
    }
    ui.separator();

    for (account, clears) in data.friends.clears_by_account() {
        // TODO: Move this into friend list struct impl
        let shown = if let Some(setting) = settings.friend_list.iter().filter(|x| x.account_name() == account).next() {
            setting.show_in_friends()
        } else {
            warn!("no settings available for friend");
            false
        };

        if shown {
            ui.text(account);
            ui.same_line(0.0);
            ui.text(im_str!("finished encounters {}", clears.finished_encounter_ids().join(",")));
        }
    }

    ui.separator();
    ui.text("temporary settings");

    // TODO: drag&drop
    // TODO: make this a proper table
    for mut friend in &mut settings.friend_list {
        // Hide currently unavailable friends, but do not remove them
        if !data.friends.state_available(friend.account_name()) {
            continue;
        }

        let width = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
        if ui.arrow_button(&im_str!("##friend_up_{}", friend.account_name()), Direction::Up) {
            // TODO implement
            // beware: hidden need to be skipped
            warn!("not implemented yet")
            // TODO: tooltip
        }
        ui.same_line(0.0);
        if ui.arrow_button(&im_str!("##friend_down_{}", friend.account_name()), Direction::Down) {
            // TODO implement
            // beware: hidden need to be skipped
            warn!("not implemented yet")
            // TODO: tooltip
        }
        width.pop(&ui);
        ui.same_line(0.0);
        ui.text(friend.account_name());
        ui.same_line(0.0);
        let width = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
        ui.checkbox(&im_str!("##friend_show_{}", friend.account_name()), friend.show_in_friends_mut());
        width.pop(&ui);
    }

    // TODO: a cooldown on this button so it's not too spammable
    if ui.button(im_str!("Refresh"), [0.0, 0.0]) {
        bg_workers.api_sender().send(ApiJob::UpdateFriendState);
    }
}
