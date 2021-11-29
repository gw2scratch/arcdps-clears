use std::time::Instant;

use arcdps::imgui::{CollapsingHeader, MenuItem, MouseButton, StyleColor, StyleVar, TableBgTarget, TableColumnFlags, TableColumnSetup, TableFlags, TableRowFlags, Ui};

use crate::clears::{RaidClearState, RaidWings};
use crate::Data;

use crate::settings::{AccountHeaderStyle, ClearsStyle, ClearsTableStyle, Settings};
use crate::translations::{encounter_english_name, Translation};
use crate::ui::{get_api_key_name, settings, UiState, utils};
use crate::workers::BackgroundWorkers;

pub fn my_clears(
    ui: &Ui,
    ui_state: &mut UiState,
    data: &Data,
    bg_workers: &BackgroundWorkers,
    settings: &mut Settings,
    tr: &Translation,
) {
    if let Some(raids) = data.clears.raids() {
        if settings.api_keys.len() == 0 {
            utils::centered_text(ui, &tr.translate("clears-intro-welcome"));
            ui.text("");
            ui.text(tr.translate("clears-intro-get-started-prefix"));
            ui.same_line();
            if ui.small_button(&tr.translate("clears-intro-get-started-button")) {
                ui_state.api_key_window.shown = true;
            }

            // We remove spacing here to remove space before the period to make the small button
            // look like just another word in the sentence.
            if let _no_spacing = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0])) {
                ui.same_line();
                ui.text(tr.translate("clears-intro-get-started-postfix"));
            }
        } else if settings.api_keys.iter().filter(|x| x.show_key_in_clears()).count() == 0 {
            if let _wrap = ui.push_text_wrap_pos_with_pos(ui.current_font_size() * 25.0) {
                ui.text_wrapped(&tr.translate("clears-all-accounts-hidden"));
            }
        }

        let mut entries: Vec<_> = settings.api_keys.iter_mut()
            .filter(|key| key.show_key_in_clears())
            .map(|key| ClearTableEntry {
                account_name: get_api_key_name(key, tr),
                state: data.clears.state(key),
                expanded: key.expanded_in_clears_mut()
            })
            .collect();

        clears_table(ui, raids, &mut entries, &settings.my_clears_style, settings.short_names, tr, || {
            utils::centered_text(&ui, &tr.translate("clears-no-clears-data-yet"));
            ui.text("");
            // TODO: Custom prompt for missing perms

            let time = *bg_workers.api_refresher_next_wakeup().lock().unwrap();
            let until_wakeup = time.saturating_duration_since(Instant::now());
            utils::centered_text(
                &ui,
                format!("{}{}{}", tr.translate("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.translate("next-refresh-secs-suffix")),
            );
        });
    } else {
        ui.text(tr.translate("clears-no-public-data-yet"));
        ui.text("");

        let time = *bg_workers.api_refresher_next_wakeup().lock().unwrap();
        let until_wakeup = time.saturating_duration_since(Instant::now());
        utils::centered_text(
            &ui,
            format!("{}{}{}", tr.translate("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.translate("next-refresh-secs-suffix")),
        );
    }

    if ui.is_mouse_released(MouseButton::Right) && ui.is_window_hovered() {
        ui.open_popup("##RightClickMenuMyClears");
    }

    ui.popup("##RightClickMenuMyClears", || {
        if let _small_frame_padding = ui.push_style_var(StyleVar::FramePadding([1.0, 1.0])) {
            ui.menu(&tr.translate("clears-contextmenu-account-list"), || {
                let entries: Vec<_> = settings.api_keys.iter_mut()
                    .map(|key| (get_api_key_name(key, tr), key.show_key_in_clears_mut()))
                    .collect();

                for (name, shown) in entries {
                    if MenuItem::new(format!("{}##ClearsContextCheckbox", name))
                        .selected(*shown)
                        .build(ui) {
                        *shown = !*shown;
                    }
                }
            });
            ui.separator();
            settings::style_section(&ui, "my-clears-style-tooltip", &mut settings.my_clears_style, tr);
        }
    })
}

pub struct ClearTableEntry<'a> {
    pub account_name: String,
    pub state: Option<&'a RaidClearState>,
    pub expanded: &'a mut bool,
}

pub fn clears_table<F: Fn()>(
    ui: &Ui,
    raids: &RaidWings,
    data: &mut [ClearTableEntry],
    style: &ClearsStyle,
    short_names: bool,
    tr: &Translation,
    no_data_available: F
) {
    let max_bosses = raids
        .wings()
        .iter()
        .map(|x| x.encounters().len())
        .max()
        .unwrap_or_default();

    match style.table_style {
        ClearsTableStyle::WingRows => {
            let mut first_key = true;
            for (i, item) in data.iter_mut().enumerate() {
                if !first_key {
                    account_separator(&ui, style.account_header_style);
                }
                first_key = false;

                if account_header(ui, &item.account_name, item.expanded, style.account_header_style) {
                    if let Some(clears) = item.state {
                        let column_count = if style.show_clears_table_row_names {
                            max_bosses + 1
                        } else {
                            max_bosses
                        };
                        if let Some(_t) = ui.begin_table_with_flags(
                            format!("ClearsTableRows##{}", i),
                            column_count,
                            TableFlags::BORDERS | TableFlags::NO_HOST_EXTEND_X,
                        ) {
                            if style.show_clears_table_row_names {
                                ui.table_setup_column("");
                            }
                            for boss in 0..max_bosses {
                                ui.table_setup_column(format!("{} {}", tr.translate("clears-header-boss"), boss + 1 ));
                            }
                            if style.show_clears_table_headers {
                                ui.table_headers_row();
                            }
                            for (wing_index, wing) in raids.wings().iter().enumerate() {
                                ui.table_next_row();
                                if style.show_clears_table_row_names {
                                    ui.table_next_column();
                                    ui.text(format!("{}{}", tr.translate("clears-wing-prefix"), wing_index + 1));
                                }
                                for column in 0..max_bosses {
                                    ui.table_next_column();
                                    if let Some(encounter) = wing.encounters().get(column) {
                                        let finished = clears.is_finished(&encounter);

                                        let bg_color = if finished {
                                            style.finished_clear_color
                                        } else {
                                            style.unfinished_clear_color
                                        };

                                        if short_names {
                                            utils::centered_text(
                                                ui,
                                                tr.encounter_short_name_im_string(encounter),
                                            );
                                        } else {
                                            utils::centered_text(
                                                ui,
                                                encounter_english_name(encounter),
                                            );
                                        }

                                        ui.table_set_bg_color(TableBgTarget::CELL_BG, bg_color);
                                    }
                                    ui.next_column()
                                }
                            }
                        }
                    } else {
                        no_data_available();
                    }
                }
            }
        }
        ClearsTableStyle::WingColumns => {
            let mut first_key = true;
            for (i, item) in data.iter_mut().enumerate() {

                if !first_key {
                    account_separator(&ui, style.account_header_style);
                }
                first_key = false;

                if account_header(ui, &item.account_name, item.expanded, style.account_header_style) {
                    if let Some(clears) = item.state {
                        let column_count = if style.show_clears_table_row_names {
                            raids.wings().len() + 1
                        } else {
                            raids.wings().len()
                        };
                        if let Some(_t) = ui.begin_table_with_flags(
                            format!("ClearsTableColumns##{}", i),
                            column_count,
                            TableFlags::BORDERS | TableFlags::NO_HOST_EXTEND_X,
                        ) {
                            if style.show_clears_table_row_names {
                                ui.table_setup_column("");
                            }
                            for (wing_index, _wing) in raids.wings().iter().enumerate() {
                                ui.table_setup_column(format!("{} {}", tr.translate("clears-wing-prefix-full"), wing_index + 1));
                            }
                            if style.show_clears_table_headers {
                                ui.table_headers_row();
                            }
                            for boss in 0..max_bosses {
                                ui.table_next_row();
                                if style.show_clears_table_row_names {
                                    ui.table_next_column();
                                    ui.text(format!("{} {}", tr.translate("clears-header-boss"), boss + 1));
                                }
                                for wing in raids.wings() {
                                    ui.table_next_column();
                                    if let Some(encounter) = wing.encounters().get(boss) {
                                        let finished = clears.is_finished(&encounter);

                                        let bg_color = if finished {
                                            style.finished_clear_color
                                        } else {
                                            style.unfinished_clear_color
                                        };

                                        if short_names {
                                            utils::centered_text(&ui, tr.encounter_short_name_im_string(encounter));
                                        } else {
                                            utils::centered_text(&ui, encounter_english_name(encounter));
                                        }

                                        ui.table_set_bg_color(TableBgTarget::CELL_BG, bg_color);
                                    }
                                    ui.next_column()
                                }
                            }
                        }
                    } else {
                        no_data_available();
                    }
                }
            }
        }
        ClearsTableStyle::SingleRow => {
            /*
            Beware, this is significantly cursed, attempts to simplify this are likely to fail.

            Our aim is to have one header per wing, and multiple columns under each header.
            We also want to have wider borders between each wing.

            The imgui API doesn't quite offer that, and instead we have an an outer table
            containing an inner table per wing, and an extra table with rows for account names.

            There are a couple problems with this approach that we work around:

            Dynamic sizing does not work properly without specifying fixed column sizes on
            the outer table. Without that, the window had to be shaken for the inner tables
            to properly unfold to full size. Do not ask me why.

            Instead, we compute the width of inner tables, accounting for borders and use
            fixed width columns.

            Another issue is that we had some extra cell padding added due to the extra layer
            of tables, which looks ugly with horizontal borders between rows enabled - horizontal
            borders add a border on the top as well, which only looks fine if there is
            no gap between it and the header. We reduce this padding to zero and reintroduce it
            manually in the headers.

            Also note that getting good rendering of borders requires a very specific
            configuration of flags that we found by experimenting, and in some cases vertical
            borders are affected by horizontal ones.
            */

            let max_name_width = data.iter()
                .map(|item| ui.calc_text_size(&item.account_name)[0])
                .fold(0.0, f32::max);

            let cell_padding = ui.clone_style().cell_padding;
            // We need to remove cell padding of the outer table, but keep it in inner tables.
            // The cell padding is saved into the table on construction, which means we cannot
            // selectively apply it only to relevant parts.
            let outer_cell_padding = ui.push_style_var(StyleVar::CellPadding([0.0, 0.0]));
            if let Some(_outer_table) = ui.begin_table_with_flags(
                "ClearsTableCompactOuter",
                raids.wings().len() + 1,
                TableFlags::BORDERS_OUTER | TableFlags::BORDERS_INNER_V | TableFlags::NO_HOST_EXTEND_X | TableFlags::SIZING_FIXED_FIT | TableFlags::NO_PAD_INNER_X,
            ) {
                outer_cell_padding.pop();

                let mut table_headers_names = Vec::new();
                table_headers_names.push(tr.translate("clears-account-column-header"));
                ui.table_setup_column_with(TableColumnSetup {
                    name: table_headers_names.last().unwrap(),
                    flags: TableColumnFlags::WIDTH_FIXED,
                    // We add 10.0 to the account name width as right padding
                    init_width_or_weight: max_name_width + 10.0,
                    user_id: Default::default()
                });

                for (wing_index, wing) in raids.wings().iter().enumerate() {
                    let inner_width = (ui.current_font_size() * 1.5).ceil() * wing.encounters().len() as f32
                        + (wing.encounters().len() - 1) as f32 // Inner borders
                        + 2.0; // Outer borders
                    table_headers_names.push(format!("{} {}", tr.translate("clears-wing-prefix-full"), wing_index + 1));
                    ui.table_setup_column_with(TableColumnSetup {
                        name: table_headers_names.last().unwrap(),
                        flags: TableColumnFlags::WIDTH_FIXED,
                        init_width_or_weight: inner_width,
                        user_id: Default::default()
                    });
                }

                // We construct headers manually to add missing padding instead of using table_headers_row()
                if style.show_clears_table_headers {
                    ui.table_next_row_with_flags(TableRowFlags::HEADERS);
                    for i in 0..ui.table_column_count() {
                        if !ui.table_set_column_index(i) {
                            continue;
                        }
                        ui.dummy([0.0, ui.current_font_size() + cell_padding[1]]);
                        ui.same_line_with_spacing(0.0, 0.0);
                        ui.set_cursor_pos([ui.cursor_pos()[0] + cell_padding[0], ui.cursor_pos()[1] + cell_padding[1]]);
                        ui.table_header(&table_headers_names[i as usize]);
                    }
                }

                ui.table_next_column();

                // Account table
                if let Some(_account_table) = ui.begin_table_with_flags(
                    "ClearsTableCompactAccounts",
                    1,
                    TableFlags::BORDERS_INNER_H | TableFlags::PAD_OUTER_X,
                ) {
                    for item in data.iter() {
                        ui.table_next_column();
                        ui.text(&item.account_name);
                    }
                }

                // Wing tables
                for (wing_index, wing) in raids.wings().iter().enumerate() {
                    ui.table_next_column();
                    if let Some(_wing_table) = ui.begin_table_with_flags(
                        format!("ClearsTableCompactWing{}", wing_index),
                        wing.encounters().len(),
                        TableFlags::NO_PAD_OUTER_X | TableFlags::NO_PAD_INNER_X | TableFlags::BORDERS_INNER | TableFlags::BORDERS_OUTER_V | TableFlags::BORDERS_OUTER_H,
                    ) {
                        for (encounter_index, _encounter) in wing.encounters().iter().enumerate() {
                            ui.table_setup_column_with(TableColumnSetup {
                                name: format!("W{}B{}", wing_index, encounter_index),
                                flags: TableColumnFlags::WIDTH_FIXED,
                                init_width_or_weight: (ui.current_font_size() * 1.5).ceil(),
                                user_id: Default::default()
                            });
                        }

                        for item in data.iter() {
                            for encounter in wing.encounters() {
                                ui.table_next_column();
                                if let Some(clears) = item.state {
                                    let finished = clears.is_finished(&encounter);

                                    let bg_color = if finished {
                                        style.finished_clear_color
                                    } else {
                                        style.unfinished_clear_color
                                    };

                                    ui.table_set_bg_color(TableBgTarget::CELL_BG, bg_color);

                                    // A centered checkbox with transparent background
                                    let padding_style = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
                                    let standard_style = ui.push_style_color(StyleColor::FrameBg, [0.0, 0.0, 0.0, 0.0]);
                                    let hover_style = ui.push_style_color(StyleColor::FrameBgHovered, ui.style_color(StyleColor::FrameBg));
                                    let active_style = ui.push_style_color(StyleColor::FrameBgActive, ui.style_color(StyleColor::FrameBg));

                                    let current_x = ui.cursor_pos()[0];
                                    let checkbox_width = ui.frame_height();
                                    let column_width = ui.current_column_width();
                                    let new_x = (current_x + column_width / 2. - checkbox_width / 2.).max(current_x);
                                    ui.set_cursor_pos([new_x, ui.cursor_pos()[1]]);

                                    let mut finished_checkbox_copy = finished;
                                    ui.checkbox("", &mut finished_checkbox_copy);
                                    active_style.pop();
                                    hover_style.pop();
                                    standard_style.pop();
                                    padding_style.pop();
                                } else {
                                    utils::centered_text(&ui, &tr.translate("clears-compressed-layout-short-unknown"));
                                }
                            }
                        }

                        // Tooltips with encounter name for each column
                        for (i, encounter) in wing.encounters().iter().enumerate() {
                            if ui.table_column_flags_with_column(i).contains(TableColumnFlags::IS_HOVERED) {
                                ui.tooltip(|| {
                                    if short_names {
                                        utils::centered_text(&ui, tr.encounter_short_name_im_string(encounter));
                                    } else {
                                        utils::centered_text(&ui, encounter_english_name(encounter));
                                    }
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn account_separator(ui: &Ui, style: AccountHeaderStyle) {
    match style {
        AccountHeaderStyle::None => ui.separator(),
        AccountHeaderStyle::CenteredText => ui.separator(),
        AccountHeaderStyle::Collapsible => {}
    }
}

pub fn account_header<T: AsRef<str>>(ui: &Ui, name: T, expanded: &mut bool, style: AccountHeaderStyle) -> bool {
    let mut shown = true;
    match style {
        AccountHeaderStyle::None => {}
        AccountHeaderStyle::CenteredText => utils::centered_text(ui, name),
        AccountHeaderStyle::Collapsible => {
            *expanded = CollapsingHeader::new(name).default_open(*expanded).build(&ui);
            shown = *expanded;
        }
    };

    shown
}