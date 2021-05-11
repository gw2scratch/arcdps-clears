use arcdps::imgui;
use crate::translations::Translation;
use crate::settings::{ApiKey, Settings};
use arcdps::imgui::{im_str, ImString, TabItem, TableBgTarget, TableFlags, TabBar, Window};
use crate::{Data};

pub struct UiState {
    pub ui_shown: bool,
}

pub fn draw_main_window(ui: &imgui::Ui, ui_state: &mut UiState, data: &mut Data, settings: &mut Settings, tr: &Translation) {
    if !ui_state.ui_shown {
        return;
    }

    Window::new(&tr.im_string("window-title"))
        .always_auto_resize(true)
        .focus_on_appearing(false)
        .no_nav()
        .collapsible(false)
        .opened(&mut ui_state.ui_shown)
        .build(&ui, || {
            TabBar::new(im_str!("main_tabs"))
                .build(&ui, || {
                    TabItem::new(&tr.im_string("clears-tab-title"))
                        .build(&ui, || {
                            if let Some(raids) = data.clears.raids() {
                                if let Some(clears) = data.clears.state() {
                                    let max_bosses = raids.wings().iter().map(|x| x.encounters().len()).max().unwrap_or_default();
                                    ui.begin_table_with_flags(im_str!("ClearsTable"), (max_bosses + 1) as i32, TableFlags::BORDERS);
                                    ui.table_setup_column(&im_str!(""));
                                    for boss in 0..max_bosses {
                                        ui.table_setup_column(&im_str!("{} {}", tr.im_string("clears-header-boss"), boss + 1));
                                    }
                                    ui.table_headers_row();
                                    for (wing_index, wing) in raids.wings().iter().enumerate() {
                                        ui.table_next_row();
                                        ui.table_next_column();
                                        ui.text(im_str!("{}{}", tr.im_string("clears-wing-prefix"), wing_index + 1));
                                        for column in 0..max_bosses {
                                            ui.table_next_column();
                                            if let Some(encounter) = wing.encounters().get(column) {
                                                let finished = clears.is_finished(&encounter);

                                                let bg_color = if finished {
                                                    [8. / 255., 148. / 255., 0. / 255., 1.]
                                                } else {
                                                    [157. / 255., 0. / 255., 6. / 255., 1.]
                                                };

                                                let text = ImString::new(encounter.english_name());

                                                // Center the text
                                                let current_x = ui.cursor_pos()[0];
                                                let text_width = ui.calc_text_size(&text, false, -1.0)[0];
                                                let column_width = ui.current_column_width();
                                                let new_x = (current_x + column_width / 2. - text_width / 2.).max(current_x);
                                                ui.set_cursor_pos([new_x, ui.cursor_pos()[1]]);
                                                ui.text(text);

                                                ui.table_set_bg_color(TableBgTarget::CELL_BG, bg_color);
                                            }
                                            ui.next_column()
                                        }
                                    }
                                    ui.end_table();
                                }
                            }
                        });
                    TabItem::new(&tr.im_string("friends-tab-title"))
                        .build(&ui, || {
                            ui.text(tr.im_string("not-implemented-yet"))
                        });
                    TabItem::new(&tr.im_string("settings-tab-title"))
                        .build(&ui, || {
                            let mut api_key = settings.main_api_key().as_ref()
                                .map(|x| ImString::new(x.key())).unwrap_or_default();

                            if ui.input_text(&tr.im_string("setting-gw2-api-key"), &mut api_key)
                                .resize_buffer(true)
                                .build() {
                                settings.set_main_api_key(Some(ApiKey::new(api_key.to_str())));
                            }
                        });
                });
        });
}