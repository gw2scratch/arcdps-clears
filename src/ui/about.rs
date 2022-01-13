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
    if ui.button(tr.translate("about-guide-button")) {
        let _ = open::that(urls::guide::GUIDE);
    }
    ui.same_line();
    if ui.is_item_hovered() {
        ui.tooltip_text(tr.translate("tooltip-opens-in-a-browser"));
    }
    if ui.button(tr.translate("about-source-button")) {
        let _ = open::that(urls::SOURCE_CODE);
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(tr.translate("tooltip-opens-in-a-browser"));
    }
    ui.same_line();
    if ui.button(tr.translate("about-website-button")) {
        let _ = open::that(urls::WEBSITE);
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(tr.translate("tooltip-opens-in-a-browser"));
    }
    ui.same_line();
    if ui.button(tr.translate("about-discord-button")) {
        let _ = open::that(urls::DISCORD);
    }
    if ui.is_item_hovered() {
        ui.tooltip_text(tr.translate("tooltip-opens-in-a-browser"));
    }
}