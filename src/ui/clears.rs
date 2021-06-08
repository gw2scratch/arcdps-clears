use arcdps::imgui::{im_str, Ui, ImString, TableColumnFlags, StyleColor, StyleVar, TableBgTarget, TableFlags, TableRowFlags, CollapsingHeader};
use crate::ui::{utils, get_api_key_name, UiState};
use std::time::Instant;
use crate::translations::{encounter_english_name, Translation};
use crate::settings::{ClearsStyle, AccountHeaderStyle, Settings, ApiKey};
use crate::workers::BackgroundWorkers;
use crate::Data;
use crate::input::get_key_name;

pub fn clears(
    ui: &Ui,
    ui_state: &mut UiState,
    data: &Data,
    bg_workers: &BackgroundWorkers,
    settings: &mut Settings,
    tr: &Translation,
) {
    if let Some(raids) = data.clears.raids() {
        let max_bosses = raids
            .wings()
            .iter()
            .map(|x| x.encounters().len())
            .max()
            .unwrap_or_default();

        if settings.api_keys.len() == 0 {
            utils::centered_text(ui, &tr.im_string("clears-intro-welcome"));
            ui.text("");
            ui.text(tr.im_string("clears-intro-get-started-prefix"));
            ui.same_line(0.0);
            if ui.small_button(&tr.im_string("clears-intro-get-started-button")) {
                ui_state.api_key_window.shown = true;
            }

            // We remove spacing here to remove space before the period to make the small button
            // look like just another word in the sentence.
            let no_spacing = ui.push_style_var(StyleVar::ItemSpacing([0.0, 0.0]));
            ui.same_line(0.0);
            ui.text(tr.im_string("clears-intro-get-started-postfix"));
            no_spacing.pop(&ui);
        } else if settings.api_keys.iter().filter(|x| x.show_key_in_clears()).count() == 0 {
            let wrap = ui.push_text_wrap_pos(ui.current_font_size() * 25.0);
            ui.text_wrapped(&tr.im_string("clears-all-accounts-hidden"));
            wrap.pop(&ui);
        }

        match settings.clears_style {
            ClearsStyle::WingRows => {
                let mut first_key = true;
                for key in settings.api_keys.iter_mut() {
                    if !key.show_key_in_clears() {
                        continue;
                    }

                    if !first_key {
                        account_separator(&ui, settings.account_header_style);
                    }
                    first_key = false;

                    if account_header(ui, key, settings.account_header_style, tr) {
                        if let Some(clears) = data.clears.state(key) {
                            let column_count = if settings.show_clears_table_row_names {
                                max_bosses + 1
                            } else {
                                max_bosses
                            } as i32;
                            ui.begin_table_with_flags(
                                &im_str!("ClearsTableRows##{}", key.id()),
                                column_count,
                                TableFlags::BORDERS | TableFlags::NO_HOST_EXTEND_X
                            );
                            if settings.show_clears_table_row_names {
                                ui.table_setup_column(&im_str!(""));
                            }
                            for boss in 0..max_bosses {
                                ui.table_setup_column(&im_str!("{} {}", tr.im_string("clears-header-boss"), boss + 1 ));
                            }
                            if settings.show_clears_table_headers {
                                ui.table_headers_row();
                            }
                            for (wing_index, wing) in raids.wings().iter().enumerate() {
                                ui.table_next_row();
                                if settings.show_clears_table_row_names {
                                    ui.table_next_column();
                                    ui.text(im_str!("{}{}", tr.im_string("clears-wing-prefix"), wing_index + 1));
                                }
                                for column in 0..max_bosses {
                                    ui.table_next_column();
                                    if let Some(encounter) = wing.encounters().get(column) {
                                        let finished = clears.is_finished(&encounter);

                                        let bg_color = if finished {
                                            settings.finished_clear_color
                                        } else {
                                            settings.unfinished_clear_color
                                        };

                                        if settings.short_names {
                                            utils::centered_text(
                                                ui,
                                                &tr.encounter_short_name_im_string(encounter),
                                            );
                                        } else {
                                            utils::centered_text(
                                                ui,
                                                &ImString::new(encounter_english_name(encounter)),
                                            );
                                        }

                                        ui.table_set_bg_color(TableBgTarget::CELL_BG, bg_color);
                                    }
                                    ui.next_column()
                                }
                            }
                            ui.end_table();
                        } else {
                            // TODO: Deduplicate
                            utils::centered_text(&ui, &tr.im_string("clears-no-clears-data-yet"));
                            ui.text("");
                            // TODO: Custom prompt for missing perms

                            let time = *bg_workers.api_worker_next_wakeup().lock().unwrap();
                            let until_wakeup = time.saturating_duration_since(Instant::now());
                            utils::centered_text(
                                &ui,
                                &im_str!("{}{}{}", tr.im_string("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.im_string("next-refresh-secs-suffix")),
                            );
                        }
                    }
                }
            }
            ClearsStyle::WingColumns => {
                let mut first_key = true;
                for key in settings.api_keys.iter_mut() {
                    if !key.show_key_in_clears() {
                        continue;
                    }

                    if !first_key {
                        account_separator(&ui, settings.account_header_style);
                    }
                    first_key = false;

                    if account_header(ui, key, settings.account_header_style, tr) {
                        if let Some(clears) = data.clears.state(key) {
                            let column_count = if settings.show_clears_table_row_names {
                                raids.wings().len() + 1
                            } else {
                                raids.wings().len()
                            } as i32;
                            ui.begin_table_with_flags(
                                &im_str!("ClearsTableColumns##{}", key.id()),
                                column_count,
                                TableFlags::BORDERS | TableFlags::NO_HOST_EXTEND_X,
                            );
                            if settings.show_clears_table_row_names {
                                ui.table_setup_column(&im_str!(""));
                            }
                            for (wing_index, _wing) in raids.wings().iter().enumerate() {
                                ui.table_setup_column(&im_str!("{} {}", tr.im_string("clears-wing-prefix-full"), wing_index + 1));
                            }
                            if settings.show_clears_table_headers {
                                ui.table_headers_row();
                            }
                            for boss in 0..max_bosses {
                                ui.table_next_row();
                                if settings.show_clears_table_row_names {
                                    ui.table_next_column();
                                    ui.text(&im_str!("{} {}", tr.im_string("clears-header-boss"), boss + 1 ));
                                }
                                for wing in raids.wings() {
                                    ui.table_next_column();
                                    if let Some(encounter) = wing.encounters().get(boss) {
                                        let finished = clears.is_finished(&encounter);

                                        let bg_color = if finished {
                                            settings.finished_clear_color
                                        } else {
                                            settings.unfinished_clear_color
                                        };

                                        if settings.short_names {
                                            utils::centered_text(&ui, &tr.encounter_short_name_im_string(encounter));
                                        } else {
                                            utils::centered_text(&ui, &ImString::new(encounter_english_name(encounter)));
                                        }

                                        ui.table_set_bg_color(TableBgTarget::CELL_BG, bg_color);
                                    }
                                    ui.next_column()
                                }
                            }
                            ui.end_table();
                        } else {
                            // TODO: Deduplicate
                            utils::centered_text(&ui, &tr.im_string("clears-no-clears-data-yet"));
                            ui.text("");
                            // TODO: Custom prompt for missing perms

                            let time = *bg_workers.api_worker_next_wakeup().lock().unwrap();
                            let until_wakeup = time.saturating_duration_since(Instant::now());
                            utils::centered_text(
                                &ui,
                                &im_str!("{}{}{}", tr.im_string("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.im_string("next-refresh-secs-suffix")),
                            );
                        }
                    }
                }
            }
            ClearsStyle::SingleRow => {
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

                Instead, we compute the width of or inner tables, accounting for borders and use
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

                let max_name_width = settings.api_keys.iter()
                    .filter(|api_key| api_key.show_key_in_clears())
                    .map(|api_key| ui.calc_text_size(&get_api_key_name(api_key, tr), true, 0.0)[0])
                    .fold(0.0, f32::max);

                let cell_padding = ui.clone_style().cell_padding;
                // We need to remove cell padding of the outer table, but keep it in inner tables.
                // The cell padding is saved into the table on construction, which means we cannot
                // selectively apply it only to relevant parts.
                let outer_cell_padding = ui.push_style_var(StyleVar::CellPadding([0., 0.]));
                ui.begin_table_with_flags(
                    &im_str!("ClearsTableCompactOuter"),
                    (raids.wings().len() + 1) as i32,
                    TableFlags::BORDERS_OUTER | TableFlags::BORDERS_INNER_V | TableFlags::NO_HOST_EXTEND_X | TableFlags::SIZING_FIXED_FIT | TableFlags::NO_PAD_INNER_X,
                );
                outer_cell_padding.pop(&ui);

                let mut table_headers_names = Vec::new();
                table_headers_names.push(tr.im_string("clears-account-column-header"));
                // We add 10.0 to the account name width as right padding
                ui.table_setup_column_with_weight(table_headers_names.last().unwrap(), TableColumnFlags::WIDTH_FIXED, max_name_width + 10.0);
                for (wing_index, wing) in raids.wings().iter().enumerate() {
                    let inner_width = (ui.current_font_size() * 1.5).ceil() * wing.encounters().len() as f32
                        + (wing.encounters().len() - 1) as f32 // Inner borders
                        + 2.0; // Outer borders
                    table_headers_names.push(im_str!("{} {}", tr.im_string("clears-wing-prefix-full"), wing_index + 1));
                    ui.table_setup_column_with_weight(table_headers_names.last().unwrap(), TableColumnFlags::WIDTH_FIXED, inner_width);
                }

                // We construct headers manually to add missing padding instead of using table_headers_row()
                if settings.show_clears_table_headers {
                    ui.table_next_row_with_flags(TableRowFlags::HEADERS);
                    for i in 0..ui.table_get_column_count() {
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
                ui.begin_table_with_flags(
                    &im_str!("ClearsTableCompactAccounts"),
                    1,
                    TableFlags::BORDERS_INNER_H | TableFlags::PAD_OUTER_X,
                );
                for key in settings.api_keys() {
                    if !key.show_key_in_clears() {
                        continue;
                    }
                    ui.table_next_column();
                    ui.text(&get_api_key_name(key, tr));
                }
                ui.end_table();

                // Wing tables
                for (wing_index, wing) in raids.wings().iter().enumerate() {
                    ui.table_next_column();
                    ui.begin_table_with_flags(
                        &im_str!("ClearsTableCompactWing{}", wing_index),
                        wing.encounters().len() as i32,
                        TableFlags::NO_PAD_OUTER_X | TableFlags::NO_PAD_INNER_X | TableFlags::BORDERS_INNER | TableFlags::BORDERS_OUTER_V | TableFlags::BORDERS_OUTER_H,
                    );

                    for (encounter_index, _encounter) in wing.encounters().iter().enumerate() {
                        ui.table_setup_column_with_weight(&im_str!("W{}B{}", wing_index, encounter_index), TableColumnFlags::WIDTH_FIXED, (ui.current_font_size() * 1.5).ceil());
                    }

                    for key in settings.api_keys() {
                        if !key.show_key_in_clears() {
                            continue;
                        }

                        let state = data.clears.state(key);
                        for encounter in wing.encounters() {
                            ui.table_next_column();
                            if let Some(clears) = state {
                                let finished = clears.is_finished(&encounter);

                                let bg_color = if finished {
                                    settings.finished_clear_color
                                } else {
                                    settings.unfinished_clear_color
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
                                ui.checkbox(im_str!(""), &mut finished_checkbox_copy);
                                active_style.pop(&ui);
                                hover_style.pop(&ui);
                                standard_style.pop(&ui);
                                padding_style.pop(&ui);
                            } else {
                                utils::centered_text(&ui, &tr.im_string("clears-compressed-layout-short-unknown"));
                            }
                        }
                    }

                    // Tooltips with encounter name for each column
                    for (i, encounter) in wing.encounters().iter().enumerate() {
                        if ui.table_get_column_flags_with_column(i as i32).contains(TableColumnFlags::IS_HOVERED) {
                            ui.tooltip(|| {
                                if settings.short_names() {
                                    utils::centered_text(&ui, &tr.encounter_short_name_im_string(encounter));
                                } else {
                                    utils::centered_text(&ui, &ImString::new(encounter_english_name(encounter)));
                                }
                            });
                        }
                    }
                    ui.end_table();
                }
                ui.end_table();
            }
        }
    } else {
        ui.text(tr.im_string("clears-no-public-data-yet"));
        ui.text("");

        let time = *bg_workers.api_worker_next_wakeup().lock().unwrap();
        let until_wakeup = time.saturating_duration_since(Instant::now());
        utils::centered_text(
            &ui,
            &im_str!("{}{}{}", tr.im_string("next-refresh-secs-prefix"), until_wakeup.as_secs(), tr.im_string("next-refresh-secs-suffix")),
        );
    }
}

pub fn account_separator(ui: &Ui, style: AccountHeaderStyle) {
    match style {
        AccountHeaderStyle::None => ui.separator(),
        AccountHeaderStyle::CenteredText => ui.separator(),
        AccountHeaderStyle::Collapsible => {}
    }
}

pub fn account_header(ui: &Ui, key: &mut ApiKey, style: AccountHeaderStyle, tr: &Translation) -> bool {
    let mut shown = true;
    match style {
        AccountHeaderStyle::None => {}
        AccountHeaderStyle::CenteredText => utils::centered_text(ui, &get_api_key_name(key, tr)),
        AccountHeaderStyle::Collapsible => {
            let key_name = get_api_key_name(key, tr);
            let mut key_shown = key.expanded_in_clears_mut();
            *key_shown = CollapsingHeader::new(&key_name).default_open(*key_shown).build(&ui);
            shown = *key_shown;
        }
    };

    shown
}