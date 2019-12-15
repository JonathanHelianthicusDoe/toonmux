use gdk::{
    self,
    enums::key::{self, Key},
};
use glib::GString;

pub enum KeyName {
    Static(&'static str),
    GString(GString),
}

impl KeyName {
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::GString(g) => g.as_str(),
        }
    }
}

/// Assumes that `key` is already canonical.
pub fn key_name(key: Key) -> KeyName {
    use KeyName::*;

    match key {
        0 => Static(""),
        key::uparrow => Static("\u{2191}"),
        key::downarrow => Static("\u{2193}"),
        key::leftarrow => Static("\u{2190}"),
        key::rightarrow => Static("\u{2192}"),
        key::BackSpace => Static("\u{232b}"),
        key::Home => Static("\u{1f3e0}"),
        key::Num_Lock => Static("\u{21ed}"),
        key::Pause => Static("\u{23f8}"),
        key::Return => Static("\u{21b5}"),
        key::Scroll_Lock => Static("\u{2913}"),
        key::Shift_L => Static("\u{21e7}"),
        key::space => Static("\u{2423}"),
        key::apostrophe => Static("' \""),
        key::backslash => Static("\\ |"),
        key::bracketleft => Static("[ {"),
        key::bracketright => Static("] }"),
        key::comma => Static(", <"),
        key::equal => Static("= +"),
        key::grave => Static("` ~"),
        key::minus => Static("- _"),
        key::period => Static(". >"),
        key::semicolon => Static("; :"),
        key::slash => Static("/ ?"),
        key::F1 => Static("F1"),
        key::F2 => Static("F2"),
        key::F3 => Static("F3"),
        key::F4 => Static("F4"),
        key::F5 => Static("F5"),
        key::F6 => Static("F6"),
        key::F7 => Static("F7"),
        key::F8 => Static("F8"),
        key::F9 => Static("F9"),
        key::F10 => Static("F10"),
        key::F11 => Static("F11"),
        key::F12 => Static("F12"),
        key::Alt_L => Static("Alt"),
        key::Caps_Lock => Static("Caps"),
        key::Control_L => Static("Ctrl"),
        key::Delete => Static("Del"),
        key::Escape => Static("Esc"),
        key::Insert => Static("Ins"),
        key::Page_Down => Static("PgDn"),
        key::Page_Up => Static("PgUp"),
        key::Super_L => Static("Sup"),
        key::Tab => Static("Tab"),
        _ => GString(gdk::keyval_name(key).expect("invalid `Key`")),
    }
}

/// Gets the "canonical" version of a `Key`; the only purpose is to consider
/// keys like Ctrl that have a "left" and a "right" version to be the same, by
/// making both of them into the "left" version.
///
/// Something something, two left feet.
#[inline]
pub fn canonicalize_key(key: Key) -> Key {
    match key {
        key::Alt_R => key::Alt_L,
        key::Control_R => key::Control_L,
        key::Hyper_R => key::Hyper_L,
        key::Shift_R => key::Shift_L,
        key::Super_R => key::Super_L,
        _ => key,
    }
}
