#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::*;
#[cfg(windows)]
use windows::Win32::Foundation::*;

use once_cell::sync::OnceCell;
use std::sync::mpsc::Sender;

static HOOK_SENDER: OnceCell<Sender<String>> = OnceCell::new();

// ── Modifier VK codes to watch ──────────────────────────────────────────
#[cfg(windows)]
const MODIFIERS: &[u16] = &[
    VK_CONTROL.0, VK_LCONTROL.0, VK_RCONTROL.0,
    VK_MENU.0,    VK_LMENU.0,    VK_RMENU.0,
    VK_SHIFT.0,   VK_LSHIFT.0,   VK_RSHIFT.0,
    VK_LWIN.0,    VK_RWIN.0,
];

// ── Hook callback ────────────────────────────────────────────────────────
#[cfg(windows)]
unsafe extern "system" fn keyboard_proc(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if ncode >= 0 && (wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN) {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let vk = kb.vkCode as u16;

        // Skip if this key IS a modifier
        if !MODIFIERS.contains(&vk) {
            let ctrl  = is_mod_down(VK_CONTROL);
            let alt   = is_mod_down(VK_MENU);
            let shift = is_mod_down(VK_SHIFT);
            let win   = is_mod_down(VK_LWIN) || is_mod_down(VK_RWIN);

            // Only record if at least one modifier is held
            // Shift alone only counts for special keys (F-keys, arrows, etc.)
            if ctrl || alt || win || (shift && is_special_key(vk)) {
                let shortcut = build_shortcut_string(ctrl, alt, shift, win, vk);
                let _ = HOOK_SENDER.get().map(|tx| tx.send(shortcut));
            }
        }
    }
    CallNextHookEx(None, ncode, wparam, lparam)
}

#[cfg(windows)]
fn is_mod_down(vk: VIRTUAL_KEY) -> bool {
    unsafe { (GetAsyncKeyState(vk.0 as i32) & 0x8000u16 as i16) != 0 }
}

/// Shift alone counts only for function keys F1–F24,
/// arrow keys, PgUp/Dn, Home, End, Insert, Delete — not for letters/numbers
fn is_special_key(vk: u16) -> bool {
    matches!(vk,
        0x21..=0x2F |   // PgUp, PgDn, End, Home, arrows, Insert, Delete
        0x70..=0x87     // F1–F24
    )
}

/// Normalize to "Ctrl+Alt+Shift+Win+Key" format
fn build_shortcut_string(ctrl: bool, alt: bool, shift: bool, win: bool, vk: u16) -> String {
    let mut parts = Vec::new();
    if ctrl  { parts.push("Ctrl"); }
    if alt   { parts.push("Alt"); }
    if shift { parts.push("Shift"); }
    if win   { parts.push("Win"); }
    parts.push(vk_to_name(vk));
    parts.join("+")
}

/// Map VK code to a human-readable key name.
fn vk_to_name(vk: u16) -> &'static str {
    match vk {
        0x08 => "Backspace",
        0x09 => "Tab",
        0x0D => "Enter",
        0x1B => "Esc",
        0x20 => "Space",
        0x21 => "PgUp",
        0x22 => "PgDn",
        0x23 => "End",
        0x24 => "Home",
        0x25 => "\u{2190}", // ←
        0x26 => "\u{2191}", // ↑
        0x27 => "\u{2192}", // →
        0x28 => "\u{2193}", // ↓
        0x2D => "Insert",
        0x2E => "Delete",
        // 0-9
        0x30 => "0", 0x31 => "1", 0x32 => "2", 0x33 => "3", 0x34 => "4",
        0x35 => "5", 0x36 => "6", 0x37 => "7", 0x38 => "8", 0x39 => "9",
        // A-Z
        0x41 => "A", 0x42 => "B", 0x43 => "C", 0x44 => "D", 0x45 => "E",
        0x46 => "F", 0x47 => "G", 0x48 => "H", 0x49 => "I", 0x4A => "J",
        0x4B => "K", 0x4C => "L", 0x4D => "M", 0x4E => "N", 0x4F => "O",
        0x50 => "P", 0x51 => "Q", 0x52 => "R", 0x53 => "S", 0x54 => "T",
        0x55 => "U", 0x56 => "V", 0x57 => "W", 0x58 => "X", 0x59 => "Y",
        0x5A => "Z",
        // Numpad
        0x60 => "Num0", 0x61 => "Num1", 0x62 => "Num2", 0x63 => "Num3",
        0x64 => "Num4", 0x65 => "Num5", 0x66 => "Num6", 0x67 => "Num7",
        0x68 => "Num8", 0x69 => "Num9",
        0x6A => "Num*", 0x6B => "Num+", 0x6D => "Num-", 0x6E => "Num.",
        0x6F => "Num/",
        // Function keys
        0x70 => "F1",  0x71 => "F2",  0x72 => "F3",  0x73 => "F4",
        0x74 => "F5",  0x75 => "F6",  0x76 => "F7",  0x77 => "F8",
        0x78 => "F9",  0x79 => "F10", 0x7A => "F11", 0x7B => "F12",
        0x7C => "F13", 0x7D => "F14", 0x7E => "F15", 0x7F => "F16",
        0x80 => "F17", 0x81 => "F18", 0x82 => "F19", 0x83 => "F20",
        0x84 => "F21", 0x85 => "F22", 0x86 => "F23", 0x87 => "F24",
        // OEM keys
        0xBA => ";",  0xBB => "=",  0xBC => ",",  0xBD => "-",
        0xBE => ".",  0xBF => "/",  0xC0 => "`",  0xDB => "[",
        0xDC => "\\", 0xDD => "]",  0xDE => "'",
        // Fallback — we use a static leak to return &'static str
        _ => "?",
    }
}

/// Start the keyboard hook on a dedicated OS thread.
#[cfg(windows)]
pub fn start_hook(sender: Sender<String>) {
    HOOK_SENDER.set(sender).ok();

    std::thread::spawn(|| {
        unsafe {
            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_proc),
                None,
                0,
            );

            if hook.is_err() {
                eprintln!("[keyflex] Failed to set keyboard hook");
                return;
            }

            // Message pump — required to keep the hook active
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });
}

#[cfg(not(windows))]
pub fn start_hook(_sender: Sender<String>) {
    eprintln!("[keyflex] Keyboard hook is only supported on Windows");
}
