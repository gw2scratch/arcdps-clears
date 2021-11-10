use std::borrow::Cow;

use arcdps::imgui::{ChildWindow, CollapsingHeader, ColorEdit, ColorEditFlags, ComboBox, im_str, PopupModal, TableFlags, Ui};

use crate::settings::{AccountHeaderStyle, ClearsStyle, ClearsTableStyle, Settings};
use crate::translations::Translation;
use crate::ui::{UiState, utils};

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
            tr,
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
            tr,
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

    if CollapsingHeader::new(&tr.im_string("settings-section-common-style"))
        .build(&ui) {
        common_style_section(ui, settings, tr);
    }

    if CollapsingHeader::new(&tr.im_string("settings-section-my-clears-style"))
        .build(&ui) {
        style_section(ui, "myclears-style", &mut settings.my_clears_style, tr);
    }

    if CollapsingHeader::new(&tr.im_string("settings-section-friends-clears-style"))
        .build(&ui) {
        style_section(ui, "friends-style", &mut settings.friends_clears_style, tr);
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

    if ui.button(&tr.im_string("setting-button-manage-api-keys"), [ui.content_region_avail()[0], 0.0]) {
        ui_state.api_key_window.shown = true;
    }
}

pub fn common_style_section(ui: &Ui, settings: &mut Settings, tr: &Translation) {
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

    /* Show main window title */
    ui.checkbox(
        &tr.im_string("setting-main-window-show-title"),
        &mut settings.main_window_show_title,
    );
    ui.same_line(0.0);
    utils::help_marker(
        ui,
        tr.im_string("setting-main-window-show-title-description"),
    );

    /* Show main window background */
    ui.checkbox(
        &tr.im_string("setting-main-window-show-bg"),
        &mut settings.main_window_show_bg,
    );
    ui.same_line(0.0);
    utils::help_marker(
        ui,
        tr.im_string("setting-main-window-show-bg-description"),
    );

    let reset_modal_label = tr.im_string("setting-reset-style-modal-title");
    if ui.button(&tr.im_string("setting-reset-style-button"), [0.0, 0.0]) {
        ui.open_popup(&reset_modal_label);
    }
    PopupModal::new(&ui, &reset_modal_label)
        .save_settings(false)
        .build(|| {
            ui.text(tr.im_string("setting-reset-style-modal-question"));
            ui.separator();
            if ui.begin_table_with_flags(im_str!("ResetConfirmationPopupTable"), 2, TableFlags::SIZING_STRETCH_SAME) {
                ui.table_next_row();
                ui.table_next_column();
                if ui.button(&tr.im_string("setting-reset-style-modal-confirm"), [ui.current_column_width(), 0.0]) {
                    settings.reset_style();
                    ui.close_current_popup();
                }
                ui.set_item_default_focus();
                ui.table_next_column();
                if ui.button(&tr.im_string("setting-reset-style-modal-cancel"), [ui.current_column_width(), 0.0]) {
                    ui.close_current_popup();
                }
                ui.end_table();
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

    if ComboBox::new(&im_str!("{}##{}", tr.im_string("setting-clears-style"), imgui_id_label))
        .build_simple(&ui, &mut table_style_index, &table_styles, &|style|
            Cow::from(match style {
                ClearsTableStyle::WingRows => tr.im_string("setting-clears-style-option-rows"),
                ClearsTableStyle::WingColumns => tr.im_string("setting-clears-style-option-columns"),
                ClearsTableStyle::SingleRow => tr.im_string("setting-clears-style-option-single-row"),
            }),
        ) {
        style.table_style = table_styles[table_style_index];
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
        .position(|x| *x == style.account_header_style)
        .unwrap_or_default();

    if ComboBox::new(&im_str!("{}##{}", tr.im_string("setting-clears-header-style"), imgui_id_label))
        .build_simple(&ui, &mut account_style_index, &account_header_styles, &|style|
            Cow::from(match style {
                AccountHeaderStyle::None => tr.im_string("setting-clears-header-style-none"),
                AccountHeaderStyle::CenteredText => tr.im_string("setting-clears-header-style-centered"),
                AccountHeaderStyle::Collapsible => tr.im_string("setting-clears-header-style-collapsible"),
            }),
        ) {
        style.account_header_style = account_header_styles[account_style_index]
    }
    ui.same_line(0.0);
    ui.align_text_to_frame_padding();
    utils::help_marker(ui, tr.im_string("setting-clears-header-style-description"));

    /* Show table headers */
    ui.checkbox(
        &im_str!("{}##{}", tr.im_string("setting-clears-show-table-headers"), imgui_id_label),
        &mut style.show_clears_table_headers,
    );
    ui.same_line(0.0);
    utils::help_marker(
        ui,
        tr.im_string("setting-clears-show-table-headers-description"),
    );

    /* Show table headers */
    ui.checkbox(
        &im_str!("{}##{}", tr.im_string("setting-clears-show-table-row-names"), imgui_id_label),
        &mut style.show_clears_table_row_names,
    );
    ui.same_line(0.0);
    utils::help_marker(
        ui,
        tr.im_string("setting-clears-show-table-row-names-description"),
    );

    /* Colors */
    ColorEdit::new(&im_str!("{}##{}", tr.im_string("setting-finished-clear-color"), imgui_id_label),
                   &mut style.finished_clear_color)
        .flags(ColorEditFlags::NO_INPUTS | ColorEditFlags::ALPHA_PREVIEW_HALF | ColorEditFlags::ALPHA_BAR)
        .build(&ui);
    ui.same_line(0.0);
    ui.align_text_to_frame_padding();
    utils::help_marker(ui, tr.im_string("setting-finished-clear-color-description"));

    ColorEdit::new(&im_str!("{}##{}", tr.im_string("setting-unfinished-clear-color"), imgui_id_label),
                   &mut style.unfinished_clear_color)
        .flags(ColorEditFlags::NO_INPUTS | ColorEditFlags::ALPHA_PREVIEW_HALF | ColorEditFlags::ALPHA_BAR)
        .build(&ui);
    ui.same_line(0.0);
    ui.align_text_to_frame_padding();
    utils::help_marker(ui, tr.im_string("setting-unfinished-clear-color-description"));
}