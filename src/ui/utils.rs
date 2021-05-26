use arcdps::imgui::{im_str, ImStr, InputText, StyleVar, ImString, Ui};
use crate::translations::Translation;

pub fn centered_text(ui: &Ui, text: &ImStr) {
    let current_x = ui.cursor_pos()[0];
    let text_width = ui.calc_text_size(&text, false, -1.0)[0];
    let column_width = ui.current_column_width();
    let new_x = (current_x + column_width / 2. - text_width / 2.).max(current_x);
    ui.set_cursor_pos([new_x, ui.cursor_pos()[1]]);
    ui.text(text);
}

pub fn help_marker<T: AsRef<str>>(ui: &Ui, text: T) {
    let alpha = ui.push_style_var(StyleVar::Alpha(0.5));
    ui.text("(?)");
    alpha.pop(&ui);
    if ui.is_item_hovered() {
        ui.tooltip(|| {
            let wrap = ui.push_text_wrap_pos(ui.current_font_size() * 35.0);
            ui.text(text);
            wrap.pop(&ui);
        });
    }
}

pub fn keybind_input(ui: &Ui, label: &ImStr, keybind: &mut Option<usize>, tr: &Translation) {
    let mut keybind_buffer = match keybind {
        None => ImString::new(""),
        Some(key) => ImString::from(key.to_string())
    };
    let width_token = ui.push_item_width(ui.current_font_size() * 3.0);
    if InputText::new(&ui, label, &mut keybind_buffer)
        .chars_decimal(true)
        .build() {
        if let Ok(new_keybind) = keybind_buffer.to_str().parse() {
            *keybind = Some(new_keybind);
        } else {
            *keybind = None;
        }
    }
    width_token.pop(&ui);

    let original_style = ui.clone_style();
    let spacing_token = ui.push_style_var(StyleVar::ItemSpacing([1.0, original_style.item_spacing[1]]));
    ui.same_line(0.0);
    let mut preview_buffer = if let Some(keybind) = keybind {
        if let Some(name) = crate::input::get_key_name(*keybind) {
            ImString::new(name)
        } else {
            tr.im_string("input-keybind-unknown")
        }
    } else {
        tr.im_string("input-keybind-disabled")
    };
    let width_token = ui.push_item_width(ui.calc_text_size(&preview_buffer, true, 0.0)[0] + original_style.frame_padding[0] * 2.0);
    let alpha_token = ui.push_style_var(StyleVar::Alpha(0.5));
    InputText::new(&ui, &im_str!("{}##preview", label), &mut preview_buffer)
        .read_only(true)
        .build();
    alpha_token.pop(&ui);
    width_token.pop(&ui);
    spacing_token.pop(&ui);
}
