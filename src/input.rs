pub const KEY_ESCAPE: usize = 27;

pub fn get_key_name(key: usize) -> Option<&'static str> {
    match key {
        8 => Some("BACKSPACE"),
        9 => Some("TAB"),
        12 => Some("CLEAR"),
        13 => Some("ENTER"),
        19 => Some("PAUSE"),
        20 => Some("CAPS LOCK"),
        27 => Some("ESCAPE"),
        32 => Some("SPACEBAR"),
        33 => Some("PAGE UP"),
        34 => Some("PAGE DOWN"),
        35 => Some("END"),
        36 => Some("HOME"),
        37 => Some("LEFT ARROW"),
        38 => Some("UP ARROW"),
        39 => Some("RIGHT ARROW"),
        40 => Some("DOWN ARROW"),
        48 => Some("0"),
        49 => Some("1"),
        50 => Some("2"),
        51 => Some("3"),
        52 => Some("4"),
        53 => Some("5"),
        54 => Some("6"),
        55 => Some("7"),
        56 => Some("8"),
        57 => Some("9"),
        65 => Some("A"),
        66 => Some("B"),
        67 => Some("C"),
        68 => Some("D"),
        69 => Some("E"),
        70 => Some("F"),
        71 => Some("G"),
        72 => Some("H"),
        73 => Some("I"),
        74 => Some("J"),
        75 => Some("K"),
        76 => Some("L"),
        77 => Some("M"),
        78 => Some("N"),
        79 => Some("O"),
        80 => Some("P"),
        81 => Some("Q"),
        82 => Some("R"),
        83 => Some("S"),
        84 => Some("T"),
        85 => Some("U"),
        86 => Some("V"),
        87 => Some("W"),
        88 => Some("X"),
        89 => Some("Y"),
        90 => Some("Z"),
        95 => Some("SLEEP"),
        96 => Some("NUMPAD0"),
        97 => Some("NUMPAD1"),
        98 => Some("NUMPAD2"),
        99 => Some("NUMPAD3"),
        100 => Some("NUMPAD4"),
        101 => Some("NUMPAD5"),
        102 => Some("NUMPAD6"),
        103 => Some("NUMPAD7"),
        104 => Some("NUMPAD8"),
        105 => Some("NUMPAD9"),
        106 => Some("MULTIPLY"),
        107 => Some("ADD"),
        108 => Some("SEPARATOR"),
        109 => Some("SUBTRACT"),
        110 => Some("DECIMAL"),
        111 => Some("DIVIDE"),
        112 => Some("F1"),
        113 => Some("F2"),
        114 => Some("F3"),
        115 => Some("F4"),
        116 => Some("F5"),
        117 => Some("F6"),
        118 => Some("F7"),
        119 => Some("F8"),
        120 => Some("F9"),
        121 => Some("F10"),
        122 => Some("F11"),
        123 => Some("F12"),
        124 => Some("F13"),
        125 => Some("F14"),
        126 => Some("F15"),
        127 => Some("F16"),
        128 => Some("F17"),
        129 => Some("F18"),
        130 => Some("F19"),
        131 => Some("F20"),
        132 => Some("F21"),
        133 => Some("F22"),
        134 => Some("F23"),
        135 => Some("F24"),
        186 => Some(";"),
        187 => Some("="),
        188 => Some(","),
        189 => Some("-"),
        190 => Some("."),
        191 => Some("/"),
        192 => Some("`"),
        219 => Some("["),
        220 => Some("\\"),
        221 => Some("]"),
        222 => Some("'"),
        226 => Some("<"),
        _ => None
    }
}