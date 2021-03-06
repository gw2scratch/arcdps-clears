use arcdps::imgui::{Window, TableFlags, Ui};
use crate::ui::{UiState, utils};
use crate::translations::Translation;

pub fn update_window(ui: &Ui, ui_state: &mut UiState, tr: &Translation) {
    if ui_state.update_window.shown {
        let release = &ui_state.update_window.release;
        let mut shown = ui_state.update_window.shown;
        Window::new(&tr.translate("update-window-title"))
            .always_auto_resize(true)
            .focus_on_appearing(false)
            .no_nav()
            .collapsible(false)
            .opened(&mut shown)
            .build(ui, || {
                if let Some(release) = release {
                    ui.text(tr.translate("update-available"));
                    ui.separator();
                    if let Some(_t) = ui.begin_table_with_flags("UpdateVersionColumns", 2, TableFlags::SIZING_FIXED_FIT) {
                        ui.table_next_row();
                        ui.table_next_column();
                        ui.text(tr.translate("update-current-version-prefix"));
                        ui.table_next_column();
                        ui.text(env!("CARGO_PKG_VERSION"));
                        ui.table_next_row();
                        ui.table_next_column();
                        ui.text(tr.translate("update-new-version-prefix"));
                        ui.table_next_column();
                        ui.text(release.version());
                    }
                    ui.separator();
                    utils::url_button(ui, tr.translate("update-button-changelog"), release.changelog_url(), tr);
                    ui.same_line();
                    utils::url_button(ui, tr.translate("update-button-download"), release.tool_site_url(), tr);
                } else {
                    ui.text(tr.translate("update-not-available"))
                }
            });

        ui_state.update_window.shown = shown;
    }
}