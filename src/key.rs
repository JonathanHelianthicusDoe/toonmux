use gdk::{
    self,
    keys::{self, Key},
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
        k if *k == 0 => Static(""),
        keys::constants::Up => Static("\u{2191}"),
        keys::constants::Down => Static("\u{2193}"),
        keys::constants::Left => Static("\u{2190}"),
        keys::constants::Right => Static("\u{2192}"),
        keys::constants::BackSpace => Static("\u{232b}"),
        keys::constants::Home => Static("\u{1f3e0}"),
        keys::constants::Num_Lock => Static("\u{21ed}"),
        keys::constants::Pause => Static("\u{23f8}"),
        keys::constants::Return => Static("\u{21b5}"),
        keys::constants::Scroll_Lock => Static("\u{2913}"),
        keys::constants::Shift_L => Static("\u{21e7}"),
        keys::constants::space => Static("\u{2423}"),
        keys::constants::apostrophe => Static("' \""),
        keys::constants::backslash => Static("\\ |"),
        keys::constants::bracketleft => Static("[ {"),
        keys::constants::bracketright => Static("] }"),
        keys::constants::comma => Static(", <"),
        keys::constants::equal => Static("= +"),
        keys::constants::grave => Static("` ~"),
        keys::constants::minus => Static("- _"),
        keys::constants::period => Static(". >"),
        keys::constants::semicolon => Static("; :"),
        keys::constants::slash => Static("/ ?"),
        keys::constants::F1 => Static("F1"),
        keys::constants::F2 => Static("F2"),
        keys::constants::F3 => Static("F3"),
        keys::constants::F4 => Static("F4"),
        keys::constants::F5 => Static("F5"),
        keys::constants::F6 => Static("F6"),
        keys::constants::F7 => Static("F7"),
        keys::constants::F8 => Static("F8"),
        keys::constants::F9 => Static("F9"),
        keys::constants::F10 => Static("F10"),
        keys::constants::F11 => Static("F11"),
        keys::constants::F12 => Static("F12"),
        keys::constants::Alt_L => Static("Alt"),
        keys::constants::Caps_Lock => Static("Caps"),
        keys::constants::Control_L => Static("Ctrl"),
        keys::constants::Delete => Static("Del"),
        keys::constants::Escape => Static("Esc"),
        keys::constants::Insert => Static("Ins"),
        keys::constants::Page_Down => Static("PgDn"),
        keys::constants::Page_Up => Static("PgUp"),
        keys::constants::Super_L => Static("Sup"),
        keys::constants::Tab => Static("Tab"),
        _ => GString(key.name().expect("invalid `Key`")),
    }
}

/// Gets the "canonical" version of a `Key`. This currently does two things:
///
/// - Fold keys like <kbd>Ctrl</kbd> that have a "left" and a "right" version
///   into the "left" version.
/// - Fold keys that have an associated lettercase (e.g. <kbd>g</kbd> ≠
///   <kbd>G</kbd>) into the "lowercase" version.
#[inline]
pub fn canonicalize_key(key: Key) -> Key {
    match key {
        keys::constants::Alt_R => keys::constants::Alt_L,
        keys::constants::Control_R => keys::constants::Control_L,
        keys::constants::Hyper_R => keys::constants::Hyper_L,
        keys::constants::Shift_R => keys::constants::Shift_L,
        keys::constants::Super_R => keys::constants::Super_L,
        _ => key.to_lower(),
    }
}
