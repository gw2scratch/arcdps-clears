use arcdps::imgui::{im_str, Window, TableFlags, Ui};
use crate::ui::UiState;
use crate::translations::Translation;

pub fn update_window(ui: &Ui, ui_state: &mut UiState, tr: &Translation) {
    if ui_state.update_window.shown {
        let release = &ui_state.update_window.release;
        let mut shown = ui_state.update_window.shown;
        Window::new(&tr.im_string("update-window-title"))
            .always_auto_resize(true)
            .focus_on_appearing(false)
            .no_nav()
            .collapsible(false)
            .opened(&mut shown)
            .build(&ui, || {
                if let Some(release) = release {
                    ui.text(tr.im_string("update-available"));
                    ui.separator();
                    if ui.begin_table_with_flags(im_str!("UpdateVersionColumns"), 2, TableFlags::SIZING_FIXED_FIT) {
                        ui.table_next_row();
                        ui.table_next_column();
                        ui.text(tr.im_string("update-current-version-prefix"));
                        ui.table_next_column();
                        ui.text(env!("CARGO_PKG_VERSION"));
                        ui.table_next_row();
                        ui.table_next_column();
                        ui.text(tr.im_string("update-new-version-prefix"));
                        ui.table_next_column();
                        ui.text(release.version());
                        ui.end_table();
                    }
                    ui.separator();
                    if ui.button(&tr.im_string("update-button-changelog"), [0.0, 0.0]) {
                        open::that(release.changelog_url());
                    }
                    ui.same_line(0.0);
                    if ui.button(&tr.im_string("update-button-download"), [0.0, 0.0]) {
                        open::that(release.tool_site_url());
                    }
                } else {
                    ui.text(tr.im_string("update-not-available"))
                }
            });

        ui_state.update_window.shown = shown;
    }
}