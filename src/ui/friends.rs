use arcdps::imgui::{Ui, CollapsingHeader, StyleColor};
use crate::translations::Translation;

pub fn friends(ui: &Ui, tr: &Translation) {
    let wrap_pos = ui.push_text_wrap_pos(ui.text_line_height() * 25.0);
    ui.text_wrapped(&tr.im_string("friends-not-implemented-yet"));
    ui.text("");
    ui.text("");
    ui.text_wrapped(&tr.im_string("friends-api-key-sharing-warning"));
    if CollapsingHeader::new(&tr.im_string("friends-api-key-sharing-section")).build(&ui) {
        ui.text_wrapped(&tr.im_string("friends-api-key-sharing-data-warning"));
        ui.text("");
        ui.text_wrapped(&tr.im_string("friends-api-key-sharing-impersonation-warning"));
        let warning_color = ui.push_style_color(StyleColor::Text, [1., 0., 0., 1.]);
        ui.text_wrapped(&tr.im_string("friends-api-key-sharing-not-recommended"));
        warning_color.pop(&ui);
        ui.text("");
        ui.text_wrapped(&tr.im_string("friends-api-key-sharing-feature-will-be-safer"));
        ui.text("");
        ui.text_wrapped(&tr.im_string("friends-api-key-sharing-rules"));
        ui.bullet();
        ui.text_wrapped(&tr.im_string("friends-api-key-sharing-rule-new-key"));
        ui.bullet();
        ui.text_wrapped(&tr.im_string("friends-api-key-sharing-rule-extra-perms"));
        ui.bullet();
        ui.text_wrapped(&tr.im_string("friends-api-key-sharing-rule-trust"));
        ui.text("");
        if CollapsingHeader::new(&tr.im_string("friends-api-key-sharing-subtoken-section")).build(&ui) {
            ui.text_wrapped(&tr.im_string("friends-api-key-sharing-subtoken-intro"));
            ui.text_wrapped(&tr.im_string("friends-api-key-sharing-subtoken-endpoints"));
            ui.bullet_text(&tr.im_string("friends-api-key-sharing-subtoken-tokeninfo"));
            ui.bullet_text(&tr.im_string("friends-api-key-sharing-subtoken-account"));
            ui.bullet_text(&tr.im_string("friends-api-key-sharing-subtoken-raids"));
            ui.text_wrapped(&tr.im_string("friends-api-key-sharing-subtoken-perms"));
            ui.bullet_text(&tr.im_string("api-key-details-permission-account"));
            ui.bullet_text(&tr.im_string("api-key-details-permission-progression"));
            ui.text_wrapped(&tr.im_string("friends-api-key-sharing-subtoken-expiration-date"));
        }
    }
    wrap_pos.pop(&ui);
}
