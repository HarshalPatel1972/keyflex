//! A key or key combination, e.g. `Ctrl+Shift+N`, `F2`, `Delete`.

use serde::Deserialize;

pub const CTRL: u8 = 1;
pub const ALT: u8 = 2;
pub const SHIFT: u8 = 4;
pub const WIN: u8 = 8;

const MODIFIERS: &[(&str, u8)] = &[("Ctrl", CTRL), ("Alt", ALT), ("Shift", SHIFT), ("Win", WIN)];

/// Named keys and their Windows virtual-key codes.
const NAMED_KEYS: &[(&str, u16)] = &[
    ("Backspace", 0x08),
    ("Tab", 0x09),
    ("Enter", 0x0D),
    ("Esc", 0x1B),
    ("Space", 0x20),
    ("PgUp", 0x21),
    ("PgDn", 0x22),
    ("End", 0x23),
    ("Home", 0x24),
    ("Left", 0x25),
    ("Up", 0x26),
    ("Right", 0x27),
    ("Down", 0x28),
    ("Insert", 0x2D),
    ("Delete", 0x2E),
    // Punctuation, by its position on a US keyboard.
    (";", 0xBA),
    ("=", 0xBB),
    (",", 0xBC),
    ("-", 0xBD),
    (".", 0xBE),
    ("/", 0xBF),
    ("`", 0xC0),
    ("[", 0xDB),
    ("\\", 0xDC),
    ("]", 0xDD),
    ("'", 0xDE),
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Combo {
    pub mods: u8,
    pub vk: u16,
}

impl Combo {
    /// Parse `"Ctrl+Shift+N"`. Modifiers first, one key last, case-insensitive.
    pub fn parse(text: &str) -> Option<Combo> {
        let mut parts = text.split('+').map(str::trim).peekable();
        let mut mods = 0;
        while let Some(part) = parts.next() {
            if parts.peek().is_none() {
                return Some(Combo { mods, vk: key_code(part)? });
            }
            let (_, bit) = MODIFIERS.iter().find(|(name, _)| name.eq_ignore_ascii_case(part))?;
            mods |= bit;
        }
        None
    }

    /// Single number identifying the combo, for cheap lookup inside the hook.
    pub fn code(self) -> u32 {
        (self.mods as u32) << 16 | self.vk as u32
    }

    /// How the combo is shown to the user: `Ctrl + Shift + N`.
    pub fn label(self) -> String {
        let mut parts: Vec<String> = MODIFIERS
            .iter()
            .filter(|(_, bit)| self.mods & bit != 0)
            .map(|(name, _)| name.to_string())
            .collect();
        parts.push(key_name(self.vk));
        parts.join(" + ")
    }
}

impl TryFrom<String> for Combo {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Combo::parse(&text).ok_or_else(|| format!("unknown key combination \"{text}\""))
    }
}

fn key_code(name: &str) -> Option<u16> {
    if let Some((_, vk)) = NAMED_KEYS.iter().find(|(known, _)| known.eq_ignore_ascii_case(name)) {
        return Some(*vk);
    }
    let upper = name.to_ascii_uppercase();
    match upper.as_bytes() {
        // Letters and digits share their ASCII value with the virtual-key code.
        [c @ (b'A'..=b'Z' | b'0'..=b'9')] => Some(*c as u16),
        [b'F', digits @ ..] => {
            let n: u16 = std::str::from_utf8(digits).ok()?.parse().ok()?;
            (1..=24).contains(&n).then(|| 0x6F + n)
        }
        _ => None,
    }
}

fn key_name(vk: u16) -> String {
    if let Some((name, _)) = NAMED_KEYS.iter().find(|(_, known)| *known == vk) {
        return name.to_string();
    }
    match vk {
        0x30..=0x39 | 0x41..=0x5A => (vk as u8 as char).to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        _ => "?".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modifiers_and_key() {
        assert_eq!(Combo::parse("Ctrl+J"), Some(Combo { mods: CTRL, vk: 0x4A }));
        assert_eq!(Combo::parse("ctrl + shift + n"), Some(Combo { mods: CTRL | SHIFT, vk: 0x4E }));
        assert_eq!(Combo::parse("F2"), Some(Combo { mods: 0, vk: 0x71 }));
        assert_eq!(Combo::parse("Alt+Enter"), Some(Combo { mods: ALT, vk: 0x0D }));
        assert_eq!(Combo::parse("Shift+Delete"), Some(Combo { mods: SHIFT, vk: 0x2E }));
        assert_eq!(Combo::parse("Ctrl+`"), Some(Combo { mods: CTRL, vk: 0xC0 }));
        assert_eq!(Combo::parse("Alt+="), Some(Combo { mods: ALT, vk: 0xBB }));
    }

    #[test]
    fn rejects_unknown_input() {
        assert_eq!(Combo::parse(""), None);
        assert_eq!(Combo::parse("Ctrl+"), None);
        assert_eq!(Combo::parse("Hyper+J"), None);
        assert_eq!(Combo::parse("F25"), None);
        assert_eq!(Combo::parse("Ctrl+Banana"), None);
    }

    #[test]
    fn labels_round_trip() {
        for text in ["Ctrl + J", "Ctrl + Shift + N", "F2", "Alt + Enter", "Shift + Delete", "Win + 1"] {
            assert_eq!(Combo::parse(text).unwrap().label(), text);
        }
    }
}
