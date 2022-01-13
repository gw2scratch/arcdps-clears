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

pub fn check_updates() -> bool {
    true
}

pub fn api_keys() -> Vec<ApiKey> {
    Vec::new()
}

pub fn short_names() -> bool {
    true
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

pub fn clears_check_interval_minutes() -> u32 {
    3
}

pub fn last_run_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn keybinds() -> Keybinds {
    Keybinds {
        // C (for clears)
        // Note that arcdps uses this as a default keybind, but it seems to be an uncommonly used one.
        // As we eat the input, there should be no issue and the behavior should be obvious. The user
        // can change the conflicting keybind in our plugin or in arcdps itself if needed.
        main_window: Some(67),
        api_window: None,
    }
}

pub mod friends {
    use super::*;

    pub fn settings() -> FriendSettings {
        FriendSettings {
            enabled: enabled(),
            list: friend_list(),
            friends_api_url: api_url(),
        }
    }

    pub fn enabled() -> bool {
        false
    }

    pub fn api_url() -> String {
        "https://clears.gw2scratch.com/".to_string()
    }

    pub fn friend_list() -> FriendList {
        FriendList {
            friends: Vec::new()
        }
    }
}

pub mod feature_ads {
    use crate::settings::FeatureAdverts;

    pub fn ads() -> FeatureAdverts {
        FeatureAdverts {
            friends_shown: friends_shown()
        }
    }

    pub fn friends_shown() -> bool {
        // We default to true, as we only really want to advertise this to users who are not new.
        true
    }
}