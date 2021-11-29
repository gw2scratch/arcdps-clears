use std::borrow::Cow;

use arcdps::imgui::{CollapsingHeader, ColorEdit, ColorEditFlags, PopupModal, TableFlags, Ui};

use crate::settings::{AccountHeaderStyle, ClearsStyle, ClearsTableStyle, Settings};
use crate::translations::Translation;
use crate::ui::{UiState, utils};

pub fn settings(ui: &Ui, ui_state: &mut UiState, settings: &mut Settings, tr: &Translation) {
    if CollapsingHeader::new(&tr.translate("settings-section-behavior"))
        .build(&ui) {
        /* Hide in loading screens */
        ui.checkbox(
            &tr.translate("setting-hide-in-loading-screens"),
            &mut settings.hide_in_loading_screens,
        );
        ui.same_line();
        utils::help_marker(ui, tr.translate("setting-hide-in-loading-screens-description"));

        /* Close on escape */
        ui.checkbox(
            format!("{}##behavior", tr.translate("setting-close-window-with-escape")),
            &mut settings.close_window_with_escape,
        );
        ui.same_line();
        utils::help_marker(ui, tr.translate("setting-close-window-with-escape-description"));
    }

    if CollapsingHeader::new(&tr.translate("settings-section-keybinds"))
        .build(&ui) {
        /* Keybind: Main window */
        utils::keybind_input(
            &ui,
            "##MainWindowKeybindInput",
            &mut settings.main_window_keybind,
            tr,
        );
        ui.same_line();
        ui.align_text_to_frame_padding();
        ui.text(tr.translate("setting-keybind-window-clears"));
        ui.same_line();
        ui.align_text_to_frame_padding();
        utils::help_marker(ui, tr.translate("setting-keybind-window-clears-description"));

        /* Keybind: API key window */
        utils::keybind_input(
            &ui,
            "##APIKeyWindowKeybindInput",
            &mut settings.api_window_keybind,
            tr,
        );
        ui.same_line();
        ui.align_text_to_frame_padding();
        ui.text(tr.translate("setting-keybind-window-apikeys"));
        ui.same_line();
        ui.align_text_to_frame_padding();
        utils::help_marker(ui, tr.translate("setting-keybind-window-apikeys-description"));

        ui.separator();
        /* Close on escape */
        ui.checkbox(
            format!("{}##keybinds", tr.translate("setting-close-window-with-escape")),
            &mut settings.close_window_with_escape,
        );
        ui.same_line();
        utils::help_marker(ui, tr.translate("setting-close-window-with-escape-description"));
    }

    if CollapsingHeader::new(&tr.translate("settings-section-common-style"))
        .build(&ui) {
        common_style_section(ui, settings, tr);
    }

    if CollapsingHeader::new(&tr.translate("settings-section-my-clears-style"))
        .build(&ui) {
        style_section(ui, "myclears-style", &mut settings.my_clears_style, tr);
    }

    if CollapsingHeader::new(&tr.translate("settings-section-friends-clears-style"))
        .build(&ui) {
        style_section(ui, "friends-style", &mut settings.friends_clears_style, tr);
    }

    if CollapsingHeader::new(&tr.translate("settings-section-updates"))
        .build(&ui) {
        ui.checkbox(
            &tr.translate("setting-check-updates"),
            &mut settings.check_updates,
        );
        ui.same_line();
        utils::help_marker(ui, tr.translate("setting-check-updates-description"));
    }

    if ui.button_with_size(&tr.translate("setting-button-manage-api-keys"), [ui.content_region_avail()[0], 0.0]) {
        ui_state.api_key_window.shown = true;
    }
}

pub fn common_style_section(ui: &Ui, settings: &mut Settings, tr: &Translation) {
    /* Short encounter names */
    ui.checkbox(
        &tr.translate("setting-short-encounter-names"),
        &mut settings.short_names,
    );
    ui.same_line();
    utils::help_marker(
        ui,
        tr.translate("setting-short-encounter-names-description"),
    );

    /* Show main window title */
    ui.checkbox(
        &tr.translate("setting-main-window-show-title"),
        &mut settings.main_window_show_title,
    );
    ui.same_line();
    utils::help_marker(
        ui,
        tr.translate("setting-main-window-show-title-description"),
    );

    /* Show main window background */
    ui.checkbox(
        &tr.translate("setting-main-window-show-bg"),
        &mut settings.main_window_show_bg,
    );
    ui.same_line();
    utils::help_marker(
        ui,
        tr.translate("setting-main-window-show-bg-description"),
    );

    let reset_modal_label = tr.translate("setting-reset-style-modal-title");
    if ui.button(&tr.translate("setting-reset-style-button")) {
        ui.open_popup(&reset_modal_label);
    }
    PopupModal::new(&reset_modal_label)
        .save_settings(false)
        .build(ui, || {
            ui.text(tr.translate("setting-reset-style-modal-question"));
            ui.separator();
            if let Some(_t) = ui.begin_table_with_flags("ResetConfirmationPopupTable", 2, TableFlags::SIZING_STRETCH_SAME) {
                ui.table_next_row();
                ui.table_next_column();
                if ui.button_with_size(&tr.translate("setting-reset-style-modal-confirm"), [ui.current_column_width(), 0.0]) {
                    settings.reset_style();
                    ui.close_current_popup();
                }
                ui.set_item_default_focus();
                ui.table_next_column();
                if ui.button_with_size(&tr.translate("setting-reset-style-modal-cancel"), [ui.current_column_width(), 0.0]) {
                    ui.close_current_popup();
                }
            }
        });
}

pub fn style_section(ui: &Ui, imgui_id_label: &str, style: &mut ClearsStyle, tr: &Translation) {
    /* Clear table styles */
    let table_styles = [
        ClearsTableStyle::WingRows,
        ClearsTableStyle::WingColumns,
        ClearsTableStyle::SingleRow,
    ];

    let mut table_style_index = table_styles.iter().position(|x| *x == style.table_style).unwrap_or_default();

    if ui.combo(format!("{}##{}", tr.translate("setting-clears-style"), imgui_id_label), &mut table_style_index, &table_styles, |style|
        Cow::from(match style {
            ClearsTableStyle::WingRows => tr.translate("setting-clears-style-option-rows"),
            ClearsTableStyle::WingColumns => tr.translate("setting-clears-style-option-columns"),
            ClearsTableStyle::SingleRow => tr.translate("setting-clears-style-option-single-row"),
        }),
    ) {
        style.table_style = table_styles[table_style_index];
    }
    ui.same_line();
    ui.align_text_to_frame_padding();
    utils::help_marker(ui, tr.translate("setting-clears-style-description"));

    /* Account header styles */
    // Hidden for single row layout as it's not affected.
    if !matches!(style.table_style, ClearsTableStyle::SingleRow) {
        let account_header_styles = [
            AccountHeaderStyle::None,
            AccountHeaderStyle::CenteredText,
            AccountHeaderStyle::Collapsible
        ];

        let mut account_style_index = account_header_styles.iter()
            .position(|x| *x == style.account_header_style)
            .unwrap_or_default();

        if ui.combo(format!("{}##{}", tr.translate("setting-clears-header-style"), imgui_id_label), &mut account_style_index, &account_header_styles, |style|
            Cow::from(match style {
                AccountHeaderStyle::None => tr.translate("setting-clears-header-style-none"),
                AccountHeaderStyle::CenteredText => tr.translate("setting-clears-header-style-centered"),
                AccountHeaderStyle::Collapsible => tr.translate("setting-clears-header-style-collapsible"),
            }),
        ) {
            style.account_header_style = account_header_styles[account_style_index]
        }
        ui.same_line();
        ui.align_text_to_frame_padding();
        utils::help_marker(ui, tr.translate("setting-clears-header-style-description"));
    }

    /* Show table headers */
    ui.checkbox(
        format!("{}##{}", tr.translate("setting-clears-show-table-headers"), imgui_id_label),
        &mut style.show_clears_table_headers,
    );
    ui.same_line();
    utils::help_marker(
        ui,
        tr.translate("setting-clears-show-table-headers-description"),
    );

    /* Show table headers */
    // Hidden for single row layout as it's not affected.
    if !matches!(style.table_style, ClearsTableStyle::SingleRow) {
        ui.checkbox(
            format!("{}##{}", tr.translate("setting-clears-show-table-row-names"), imgui_id_label),
            &mut style.show_clears_table_row_names,
        );
        ui.same_line();
        utils::help_marker(
            ui,
            tr.translate("setting-clears-show-table-row-names-description"),
        );
    }

    /* Colors */
    ColorEdit::new(format!("{}##{}", tr.translate("setting-finished-clear-color"), imgui_id_label),
                   &mut style.finished_clear_color)
        .flags(ColorEditFlags::NO_INPUTS | ColorEditFlags::ALPHA_PREVIEW_HALF | ColorEditFlags::ALPHA_BAR)
        .build(&ui);
    ui.same_line();
    ui.align_text_to_frame_padding();
    utils::help_marker(ui, tr.translate("setting-finished-clear-color-description"));

    ColorEdit::new(format!("{}##{}", tr.translate("setting-unfinished-clear-color"), imgui_id_label),
                   &mut style.unfinished_clear_color)
        .flags(ColorEditFlags::NO_INPUTS | ColorEditFlags::ALPHA_PREVIEW_HALF | ColorEditFlags::ALPHA_BAR)
        .build(&ui);
    ui.same_line();
    ui.align_text_to_frame_padding();
    utils::help_marker(ui, tr.translate("setting-unfinished-clear-color-description"));
}