use std::fmt::Display;
use arcdps::imgui::{InputText, StyleVar, Ui};
use crate::translations::Translation;

pub fn centered_text<T: AsRef<str>>(ui: &Ui, text: T) {
    let current_x = ui.cursor_pos()[0];
    let text_width = ui.calc_text_size(&text)[0];
    let column_width = ui.current_column_width();
    let new_x = (current_x + column_width / 2. - text_width / 2.).max(current_x);
    ui.set_cursor_pos([new_x, ui.cursor_pos()[1]]);
    ui.text(text);
}

pub fn help_marker<T: AsRef<str>>(ui: &Ui, text: T) {
    if let _alpha = ui.push_style_var(StyleVar::Alpha(0.5)) {
        ui.text("(?)");
    }

    if ui.is_item_hovered() {
        ui.tooltip(|| {
            let wrap = ui.push_text_wrap_pos_with_pos(ui.current_font_size() * 35.0);
            ui.text(text);
            wrap.pop(&ui);
        });
    }
}

pub fn warning_marker<T: AsRef<str>>(ui: &Ui, text: T) {
    if let _alpha = ui.push_style_var(StyleVar::Alpha(0.5)) {
        ui.text("(!)");
    }

    if ui.is_item_hovered() {
        ui.tooltip(|| {
            let wrap = ui.push_text_wrap_pos_with_pos(ui.current_font_size() * 35.0);
            ui.text(text);
            wrap.pop(&ui);
        });
    }
}

pub fn keybind_input<T: AsRef<str> + Display + Copy>(ui: &Ui, label: T, keybind: &mut Option<usize>, tr: &Translation) {
    let mut keybind_buffer = match keybind {
        None => String::new(),
        Some(key) => key.to_string(),
    };
    let width_token = ui.push_item_width(ui.current_font_size() * 3.0);
    if InputText::new(&ui, label, &mut keybind_buffer)
        .chars_decimal(true)
        .build() {
        if let Ok(new_keybind) = keybind_buffer.parse() {
            *keybind = Some(new_keybind);
        } else {
            *keybind = None;
        }
    }
    width_token.pop(&ui);

    let original_style = ui.clone_style();
    if let _spacing = ui.push_style_var(StyleVar::ItemSpacing([1.0, original_style.item_spacing[1]])) {
        ui.same_line();
        let mut preview_buffer = if let Some(keybind) = keybind {
            if let Some(name) = crate::input::get_key_name(*keybind) {
                name.to_string()
            } else {
                tr.translate("input-keybind-unknown")
            }
        } else {
            tr.translate("input-keybind-disabled")
        };
        if let _width = ui.push_item_width(ui.calc_text_size(&preview_buffer)[0] + original_style.frame_padding[0] * 2.0) {
            if let _alpha = ui.push_style_var(StyleVar::Alpha(0.5)) {
                InputText::new(&ui, &format!("{}##preview", label), &mut preview_buffer)
                    .read_only(true)
                    .build();
            }
        }
    }
}
