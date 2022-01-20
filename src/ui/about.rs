use arcdps::imgui::Window;
use crate::imgui::Ui;
use crate::{Translation, UiState, urls};
use crate::ui::utils;

pub fn about_window(ui: &Ui, ui_state: &mut UiState, tr: &Translation) {
    if ui_state.about_window.shown {
        let mut shown = ui_state.about_window.shown;
        Window::new(&tr.translate("about-window-title"))
            .always_auto_resize(true)
            .focus_on_appearing(true)
            .no_nav()
            .collapsible(false)
            .opened(&mut shown)
            .build(ui, || {
                about(ui, tr)
            });

        ui_state.about_window.shown = shown;
    }
}

fn about(ui: &Ui, tr: &Translation) {
    ui.dummy([ui.text_line_height() * 20.0, ui.text_line_height() * 1.5]);
    utils::centered_text(ui, format!("{} {}", tr.translate("about-name"), env!("CARGO_PKG_VERSION")));
    ui.new_line();
    utils::centered_text(ui, tr.translate("about-made-by"));
    ui.spacing();
    ui.dummy([ui.text_line_height() * 20.0, ui.text_line_height() * 2.0]);

    utils::url_button(ui, tr.translate("about-guide-button"), urls::guide::GUIDE, tr);
    ui.same_line();
    utils::url_button(ui, tr.translate("about-source-button"), urls::SOURCE_CODE, tr);
    ui.same_line();
    utils::url_button(ui, tr.translate("about-website-button"), urls::WEBSITE, tr);
    ui.same_line();
    utils::url_button(ui, tr.translate("about-discord-button"), urls::DISCORD, tr);
}