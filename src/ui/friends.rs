use arcdps::imgui::{im_str, Ui, CollapsingHeader, StyleColor};
use crate::translations::Translation;
use crate::ui::UiState;
use crate::Data;
use crate::workers::BackgroundWorkers;
use crate::settings::Settings;

pub fn friends(
    ui: &Ui,
    ui_state: &mut UiState,
    data: &Data,
    bg_workers: &BackgroundWorkers,
    settings: &mut Settings,
    tr: &Translation,
) {
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
            ui.text(im_str!("friend acc {}", friend.account()));
        }

    }
    ui.separator();

    for (account, clears) in data.friends.clears_by_account() {
        ui.text(account);
        ui.same_line(0.0);
        ui.text(im_str!("finished encounters {}", clears.finished_encounter_ids().join(",")));
    }
}
