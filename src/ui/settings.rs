use crate::ui::{utils, UiState};
use arcdps::imgui::{im_str, CollapsingHeader, ColorEditFlags, ColorEdit, ComboBox, Ui};
use crate::settings::{AccountHeaderStyle, ClearsStyle, Settings};
use std::borrow::Cow;
use crate::translations::Translation;

pub fn settings(ui: &Ui, ui_state: &mut UiState, settings: &mut Settings, tr: &Translation) {
    if CollapsingHeader::new(&tr.im_string("settings-section-behavior"))
        .build(&ui) {

        /* Hide in loading screens */
        ui.checkbox(
            &tr.im_string("setting-hide-in-loading-screens"),
            &mut settings.hide_in_loading_screens,
        );
        ui.same_line(0.0);
        utils::help_marker(ui, tr.im_string("setting-hide-in-loading-screens-description"));

        /* Close on escape */
        ui.checkbox(
            &im_str!("{}##behavior", tr.im_string("setting-close-window-with-escape")),
            &mut settings.close_window_with_escape,
        );
        ui.same_line(0.0);
        utils::help_marker(ui, tr.im_string("setting-close-window-with-escape-description"));
    }

    if CollapsingHeader::new(&tr.im_string("settings-section-keybinds"))
        .build(&ui) {
        /* Keybind: Main window */
        utils::keybind_input(
            &ui,
            im_str!("##MainWindowKeybindInput"),
            &mut settings.main_window_keybind,
            tr
        );
        ui.same_line(0.0);
        ui.align_text_to_frame_padding();
        ui.text(tr.im_string("setting-keybind-window-clears"));
        ui.same_line(0.0);
        ui.align_text_to_frame_padding();
        utils::help_marker(ui, tr.im_string("setting-keybind-window-clears-description"));

        /* Keybind: API key window */
        utils::keybind_input(
            &ui,
            im_str!("##APIKeyWindowKeybindInput"),
            &mut settings.api_window_keybind,
            tr
        );
        ui.same_line(0.0);
        ui.align_text_to_frame_padding();
        ui.text(tr.im_string("setting-keybind-window-apikeys"));
        ui.same_line(0.0);
        ui.align_text_to_frame_padding();
        utils::help_marker(ui, tr.im_string("setting-keybind-window-apikeys-description"));

        ui.separator();
        /* Close on escape */
        ui.checkbox(
            &im_str!("{}##keybinds", tr.im_string("setting-close-window-with-escape")),
            &mut settings.close_window_with_escape,
        );
        ui.same_line(0.0);
        utils::help_marker(ui, tr.im_string("setting-close-window-with-escape-description"));
    }

    if CollapsingHeader::new(&tr.im_string("settings-section-style"))
        .build(&ui) {
        style_section(ui, settings, tr);
    }

    if CollapsingHeader::new(&tr.im_string("settings-section-updates"))
        .build(&ui) {
        ui.checkbox(
            &tr.im_string("setting-check-updates"),
            &mut settings.check_updates,
        );
        ui.same_line(0.0);
        utils::help_marker(ui, tr.im_string("setting-check-updates-description"));
    }

    if ui.button(&tr.im_string("setting-button-manage-api-keys"), [ui.current_column_width(), 0.0]) {
        ui_state.api_key_window.shown = true;
    }
}

pub fn style_section(ui: &Ui, settings: &mut Settings, tr: &Translation) {
    /* Clear table styles */
    let table_styles = [
        ClearsStyle::WingRows,
        ClearsStyle::WingColumns,
        ClearsStyle::SingleRow,
    ];

    let mut table_style_index = table_styles.iter().position(|x| *x == settings.clears_style).unwrap_or_default();

    if ComboBox::new(&tr.im_string("setting-clears-style"))
        .build_simple(&ui, &mut table_style_index, &table_styles, &|style|
            Cow::from(match style {
                ClearsStyle::WingRows => tr.im_string("setting-clears-style-option-rows"),
                ClearsStyle::WingColumns => tr.im_string("setting-clears-style-option-columns"),
                ClearsStyle::SingleRow => tr.im_string("setting-clears-style-option-single-row"),
            }),
        ) {
        settings.clears_style = table_styles[table_style_index];
    }
    ui.same_line(0.0);
    ui.align_text_to_frame_padding();
    utils::help_marker(ui, tr.im_string("setting-clears-style-description"));

    /* Account header styles */
    let account_header_styles = [
        AccountHeaderStyle::None,
        AccountHeaderStyle::CenteredText,
        AccountHeaderStyle::Collapsible
    ];

    let mut account_style_index = account_header_styles.iter()
        .position(|x| *x == settings.account_header_style)
        .unwrap_or_default();

    if ComboBox::new(&tr.im_string("setting-clears-header-style"))
        .build_simple(&ui, &mut account_style_index, &account_header_styles, &|style|
            Cow::from(match style {
                AccountHeaderStyle::None => tr.im_string("setting-clears-header-style-none"),
                AccountHeaderStyle::CenteredText => tr.im_string("setting-clears-header-style-centered"),
                AccountHeaderStyle::Collapsible => tr.im_string("setting-clears-header-style-collapsible"),
            }),
        ) {
        settings.account_header_style = account_header_styles[account_style_index]
    }
    ui.same_line(0.0);
    ui.align_text_to_frame_padding();
    utils::help_marker(ui, tr.im_string("setting-clears-header-style-description"));

    /* Short encounter names */
    ui.checkbox(
        &tr.im_string("setting-short-encounter-names"),
        &mut settings.short_names,
    );
    ui.same_line(0.0);
    utils::help_marker(
        ui,
        tr.im_string("setting-short-encounter-names-description"),
    );

    /* Show table headers */
    ui.checkbox(
        &tr.im_string("setting-clears-show-table-headers"),
        &mut settings.show_clears_table_headers,
    );
    ui.same_line(0.0);
    utils::help_marker(
        ui,
        tr.im_string("setting-clears-show-table-headers-description"),
    );

    /* Show table headers */
    ui.checkbox(
        &tr.im_string("setting-clears-show-table-row-names"),
        &mut settings.show_clears_table_row_names,
    );
    ui.same_line(0.0);
    utils::help_marker(
        ui,
        tr.im_string("setting-clears-show-table-row-names-description"),
    );

    /* Colors */
    ColorEdit::new(&tr.im_string("setting-finished-clear-color"), &mut settings.finished_clear_color)
        .flags(ColorEditFlags::NO_INPUTS | ColorEditFlags::ALPHA_PREVIEW_HALF | ColorEditFlags::ALPHA_BAR)
        .build(&ui);
    ui.same_line(0.0);
    ui.align_text_to_frame_padding();
    utils::help_marker(ui, tr.im_string("setting-finished-clear-color-description"));

    ColorEdit::new(&tr.im_string("setting-unfinished-clear-color"), &mut settings.unfinished_clear_color)
        .flags(ColorEditFlags::NO_INPUTS | ColorEditFlags::ALPHA_PREVIEW_HALF | ColorEditFlags::ALPHA_BAR)
        .build(&ui);
    ui.same_line(0.0);
    ui.align_text_to_frame_padding();
    utils::help_marker(ui, tr.im_string("setting-unfinished-clear-color-description"));

}