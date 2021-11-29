use crate::settings::*;

const FINISHED_COLOR: [f32; 4] = [38. / 255., 199. / 255., 29. / 255., 177. / 255.];
const UNFINISHED_COLOR: [f32; 4] = [192. / 255., 24. / 255., 30. / 255., 136. / 255.];

pub fn my_clears_style() -> ClearsStyle {
    ClearsStyle {
        table_style: ClearsTableStyle::WingRows,
        account_header_style: AccountHeaderStyle::CenteredText,
        show_clears_table_headers: true,
        show_clears_table_row_names: true,
        finished_clear_color: FINISHED_COLOR,
        unfinished_clear_color: UNFINISHED_COLOR,
    }
}

pub fn friends_clears_style() -> ClearsStyle {
    ClearsStyle {
        table_style: ClearsTableStyle::SingleRow,
        account_header_style: AccountHeaderStyle::CenteredText,
        show_clears_table_headers: true,
        show_clears_table_row_names: true,
        finished_clear_color: FINISHED_COLOR,
        unfinished_clear_color: UNFINISHED_COLOR,
    }
}

pub fn friends_api_url() -> String {
    "https://clears.gw2scratch.com/".to_string()
}

pub fn check_updates() -> bool {
    true
}

pub fn api_keys() -> Vec<ApiKey> {
    Vec::new()
}

pub fn friends() -> Vec<Friend> {
    Vec::new()
}

pub fn friend_default_show_state() -> bool {
    true
}

pub fn short_names() -> bool {
    true
}

pub fn api_window_keybind() -> Option<usize> {
    None
}

pub fn main_window_keybind() -> Option<usize> {
    // C (for clears)
    // Note that arcdps uses this as a default keybind, but it seems to be an uncommonly used one.
    // As we eat the input, there should be no issue and the behavior should be obvious. The user
    // can change the conflicting keybind in our plugin or in arcdps itself if needed.
    Some(67)
}

pub fn close_window_with_escape() -> bool {
    true
}

pub fn hide_in_loading_screens() -> bool {
    false
}

pub fn main_window_show_bg() -> bool {
    true
}

pub fn main_window_show_title() -> bool {
    true
}

pub fn show_key_in_clears() -> bool {
    true
}

pub fn expanded_in_clears() -> bool {
    true
}
