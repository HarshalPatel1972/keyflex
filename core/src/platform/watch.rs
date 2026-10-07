//! Watching what the user does, as cheaply and narrowly as possible.
//!
//!   * The low-level hooks do no work: they pass a point or an already-filtered
//!     key combination to a worker thread and return.
//!   * A click is only inspected in apps the user has tips switched on for, and
//!     then only the names of buttons and menu items in the app's own interface
//!     are read. Text, tabs, list items, links and anything inside a document
//!     or web page are dropped without their names ever being requested.
//!   * The only keys that ever leave the keyboard hook are the combinations
//!     named in the tip rules. Other keys only update a "was typing" timestamp.
//!   * Window switches are noticed so that "opened with the mouse" can be told
//!     apart from "opened with the keyboard"; only the app's name is looked up.
//!   * No global UI Automation event subscriptions: those slow down every app.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::OnceLock;

use windows::core::{Result, PWSTR, VARIANT};
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::Accessibility::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::combo::{Combo, ALT, CTRL, SHIFT, WIN};
use crate::rules::{self, Control, WEB_UI_APPS};

/// What the hooks hand to the worker thread.
pub enum Raw {
    Click(POINT),
    Key(Combo),
    /// A window came to the front. `by_mouse`: the last thing the user did was
    /// click, with no typing or shortcut since.
    Foreground { hwnd: isize, by_mouse: bool },
}

/// A window that appears this long after a click was opened by that click.
const OPENED_BY_CLICK_MS: u64 = 6000;
/// A click this soon after typing means the hands just left the keyboard.
const TYPING_MS: u64 = 8000;

struct Hooks {
    tx: Sender<Raw>,
    combos: HashSet<u32>,
}

static HOOKS: OnceLock<Hooks> = OnceLock::new();
/// The key currently held down, so auto-repeat counts as one press.
static HELD_KEY: AtomicU32 = AtomicU32::new(0);
/// When the user last clicked, typed a plain key, or pressed any key at all
/// (system uptime in milliseconds). Times only: which key is never kept.
static LAST_CLICK: AtomicU64 = AtomicU64::new(0);
static LAST_TYPED: AtomicU64 = AtomicU64::new(0);
static LAST_KEY: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    unsafe { GetTickCount64() }
}

/// Whether the user was typing moments ago.
pub fn was_typing() -> bool {
    let typed = LAST_TYPED.load(Ordering::Relaxed);
    typed != 0 && now_ms().saturating_sub(typed) < TYPING_MS
}

// ── Hooks ────────────────────────────────────────────────────────────────

/// Hook coordinates are physical pixels; UI Automation must see the same space.
pub fn make_dpi_aware() {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

/// Install the hooks on the calling thread, which must then run [`pump`].
/// `combos` are the only key combinations that will be reported.
pub unsafe fn install(tx: Sender<Raw>, combos: HashSet<u32>) -> Result<()> {
    HOOKS.set(Hooks { tx, combos }).ok();
    SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0)?;
    SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)?;
    // Delivered to this thread's message loop; nothing is injected into other apps.
    SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND,
        EVENT_SYSTEM_FOREGROUND,
        None,
        Some(foreground_proc),
        0,
        0,
        WINEVENT_OUTOFCONTEXT,
    );
    Ok(())
}

unsafe extern "system" fn foreground_proc(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _object: i32,
    _child: i32,
    _thread: u32,
    _time: u32,
) {
    let Some(hooks) = HOOKS.get() else { return };
    let click = LAST_CLICK.load(Ordering::Relaxed);
    let by_mouse = click != 0
        && now_ms().saturating_sub(click) < OPENED_BY_CLICK_MS
        && LAST_KEY.load(Ordering::Relaxed) < click;
    let _ = hooks.tx.send(Raw::Foreground { hwnd: hwnd.0 as isize, by_mouse });
}

/// Every visible top-level window right now, so they are not mistaken for new ones later.
pub fn open_windows() -> Vec<isize> {
    unsafe extern "system" fn collect(hwnd: HWND, list: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd).as_bool() {
            (*(list.0 as *mut Vec<isize>)).push(hwnd.0 as isize);
        }
        TRUE
    }
    let mut windows = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut windows as *mut Vec<isize> as isize));
    }
    windows
}

/// The Windows class name of a window: what kind of window it is.
pub fn window_class(hwnd: isize) -> String {
    let mut buf = [0u16; 128];
    let len = unsafe { GetClassNameW(HWND(hwnd as *mut _), &mut buf) }.max(0) as usize;
    String::from_utf16_lossy(&buf[..len])
}

/// Lowercase exe name of the app a top-level window belongs to. Store apps are
/// hosted by a shared frame process, so for those the app inside is reported.
pub fn app_of_window(hwnd: isize) -> String {
    unsafe {
        let hwnd = HWND(hwnd as *mut _);
        let app = window_exe(hwnd);
        if app == "applicationframehost.exe" {
            if let Ok(inner) = FindWindowExW(hwnd, None, windows::core::w!("Windows.UI.Core.CoreWindow"), None) {
                return window_exe(inner);
            }
        }
        app
    }
}

pub unsafe fn pump() {
    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).as_bool() {
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

unsafe extern "system" fn mouse_proc(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // Button-down, not up: a menu item is gone by the time the button is released.
    if ncode >= 0 && wparam.0 as u32 == WM_LBUTTONDOWN {
        LAST_CLICK.store(now_ms(), Ordering::Relaxed);
        if let Some(hooks) = HOOKS.get() {
            let mouse = &*(lparam.0 as *const MSLLHOOKSTRUCT);
            let _ = hooks.tx.send(Raw::Click(mouse.pt));
        }
    }
    CallNextHookEx(None, ncode, wparam, lparam)
}

unsafe extern "system" fn keyboard_proc(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode >= 0 {
        let vk = (*(lparam.0 as *const KBDLLHOOKSTRUCT)).vkCode;
        match wparam.0 as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                if HELD_KEY.swap(vk, Ordering::Relaxed) != vk {
                    report_key(vk as u16);
                }
            }
            WM_KEYUP | WM_SYSKEYUP => HELD_KEY.store(0, Ordering::Relaxed),
            _ => {}
        }
    }
    CallNextHookEx(None, ncode, wparam, lparam)
}

/// Forward the key only if, with the modifiers held, it is a combination a rule names.
unsafe fn report_key(vk: u16) {
    let Some(hooks) = HOOKS.get() else { return };
    let down = |key: VIRTUAL_KEY| GetAsyncKeyState(key.0 as i32) < 0;
    let now = now_ms();
    LAST_KEY.store(now, Ordering::Relaxed);
    // A key with no Ctrl, Alt or Win held is typing (or navigating by keyboard).
    if !(down(VK_CONTROL) || down(VK_MENU) || down(VK_LWIN) || down(VK_RWIN)) {
        LAST_TYPED.store(now, Ordering::Relaxed);
    }

    // AltGr arrives as left Ctrl + right Alt and types a character; it is not a shortcut.
    if down(VK_RMENU) && down(VK_LCONTROL) {
        return;
    }
    let mut mods = 0;
    for (key, bit) in [(VK_CONTROL, CTRL), (VK_MENU, ALT), (VK_SHIFT, SHIFT), (VK_LWIN, WIN), (VK_RWIN, WIN)] {
        if down(key) {
            mods |= bit;
        }
    }
    let combo = Combo { mods, vk };
    if hooks.combos.contains(&combo.code()) {
        let _ = hooks.tx.send(Raw::Key(combo));
    }
}

// ── Looking at what was clicked ──────────────────────────────────────────

pub struct Clicked {
    /// Lowercase exe name.
    pub app: String,
    /// Element name, as `rules::normalize` returns it.
    pub name: String,
    pub control: Control,
}

#[allow(non_upper_case_globals)]
fn actionable(control_type: UIA_CONTROLTYPE_ID) -> Option<Control> {
    match control_type {
        UIA_ButtonControlTypeId => Some(Control::Button),
        UIA_MenuItemControlTypeId => Some(Control::MenuItem),
        UIA_SplitButtonControlTypeId => Some(Control::SplitButton),
        _ => None,
    }
}

pub struct Inspector {
    uia: IUIAutomation,
    cache: IUIAutomationCacheRequest,
    walker: IUIAutomationTreeWalker,
    /// Finds the document (web page, editor content) an element sits in, if any.
    documents: IUIAutomationTreeWalker,
}

impl Inspector {
    /// Must be created and used on one thread.
    pub fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
            let uia: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)?;
            // Only the element's type comes back with the lookup, never its name.
            let cache = uia.CreateCacheRequest()?;
            cache.AddProperty(UIA_ControlTypePropertyId)?;
            let walker = uia.ControlViewWalker()?;
            let is_document = uia.CreatePropertyCondition(
                UIA_ControlTypePropertyId,
                &VARIANT::from(UIA_DocumentControlTypeId.0),
            )?;
            let documents = uia.CreateTreeWalker(&is_document)?;
            Ok(Self { uia, cache, walker, documents })
        }
    }

    /// The button or menu item at `pt`, if `watching` says its app is of interest.
    pub fn inspect(&self, pt: POINT, watching: impl Fn(&str) -> bool) -> Option<Clicked> {
        unsafe {
            // Cheap checks first, so most clicks never reach UI Automation.
            let hwnd = WindowFromPoint(pt);
            if is_taskbar(GetAncestor(hwnd, GA_ROOT)) {
                return None;
            }
            let app = window_exe(hwnd);
            if !watching(&app) {
                return None;
            }

            // The click may land on a label or icon inside the button, so look a few levels up.
            let mut element = self.uia.ElementFromPointBuildCache(pt, &self.cache).ok()?;
            for _ in 0..4 {
                if let Some(control) = actionable(element.CachedControlType().ok()?) {
                    // A button inside a web page or document is content, not the app's
                    // interface, unless the whole app is drawn as a web page.
                    let web_ui = WEB_UI_APPS.contains(&app.as_str());
                    if !web_ui && self.documents.NormalizeElement(&element).is_ok() {
                        return None;
                    }
                    let name = rules::normalize(&element.CurrentName().ok()?.to_string());
                    return Some(Clicked { app, name, control });
                }
                element = self.walker.GetParentElementBuildCache(&element, &self.cache).ok()?;
            }
            None
        }
    }
}

unsafe fn is_taskbar(root: HWND) -> bool {
    let mut buf = [0u16; 64];
    let len = GetClassNameW(root, &mut buf).max(0) as usize;
    let class = String::from_utf16_lossy(&buf[..len]);
    class == "Shell_TrayWnd" || class == "Shell_SecondaryTrayWnd"
}

unsafe fn window_exe(hwnd: HWND) -> String {
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
        return String::new();
    };
    let mut buf = [0u16; 1024];
    let mut size = buf.len() as u32;
    let found = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut size);
    let _ = CloseHandle(process);
    if found.is_err() {
        return String::new();
    }
    let path = String::from_utf16_lossy(&buf[..size as usize]);
    path.rsplit('\\').next().unwrap_or("").to_lowercase()
}

/// (lowercase exe name, title) of the active window.
pub fn foreground() -> (String, String) {
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut buf).max(0) as usize;
        (window_exe(hwnd), String::from_utf16_lossy(&buf[..len]))
    }
}
