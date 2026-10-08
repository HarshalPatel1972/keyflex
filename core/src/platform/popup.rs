//! The tip popup: a small always-on-top window drawn by us, so it does not
//! depend on Windows notification settings or Do Not Disturb.
//!
//! It shows the character's face (whose expression carries the mood), the
//! shortcut as keycaps, and one line. It pops in beside the cursor, where the
//! user is already looking, never takes focus, hides itself after a few
//! seconds, and closes on click. "Don't show again" also mutes the tip.

use std::sync::atomic::{AtomicI32, AtomicIsize, Ordering};
use std::sync::{Mutex, OnceLock};

use windows::core::w;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Dwm::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::engine::Mood;

const WM_SHOW_TIP: u32 = WM_APP + 1;
const HIDE_TIMER: usize = 1;
const POP_TIMER: usize = 2;
const VISIBLE_MS: u32 = 8000;
const MUTE_LABEL: &str = "Don't show again";

/// The pop-in animation: frames, milliseconds per frame, how far it rises.
const POP_FRAMES: i32 = 14;
const POP_FRAME_MS: u32 = 15;
const POP_RISE: i32 = 14;
const OPACITY: f32 = 248.0;

// Sizes at 100% scaling.
const WIDTH: i32 = 384;
const PAD: i32 = 16;
const FACE: i32 = 64;
const FACE_GAP: i32 = 14;
const KEY_HEIGHT: i32 = 26;
const KEY_PAD: i32 = 9;
const KEY_GAP: i32 = 6;
const BODY_GAP: i32 = 10;
/// Height of the "Don't show again" row.
const FOOTER: i32 = 24;
/// Distance from the cursor, so the popup never sits under the pointer.
const CURSOR_GAP: i32 = 24;

/// Colours match the window's themes in `src/styles.css`. COLORREF is 0x00BBGGRR.
struct Palette {
    background: COLORREF,
    border: COLORREF,
    key: COLORREF,
    accent: COLORREF,
    text: COLORREF,
    muted: COLORREF,
}

const DARK: Palette = Palette {
    background: COLORREF(0x001C_2118),
    border: COLORREF(0x0032_3B2C),
    key: COLORREF(0x0026_2D21),
    accent: COLORREF(0x0084_DC3D),
    text: COLORREF(0x00E6_EDE4),
    muted: COLORREF(0x0098_A38F),
};

const LIGHT: Palette = Palette {
    background: COLORREF(0x00F1_F8FB),
    border: COLORREF(0x00C0_D3DC),
    key: COLORREF(0x00D6_E5EC),
    accent: COLORREF(0x005A_8E1E),
    text: COLORREF(0x0028_2F2A),
    muted: COLORREF(0x0066_746F),
};

struct Tip {
    /// Empty for messages that cannot be muted.
    id: String,
    /// "Ctrl + J"
    keys: String,
    body: String,
    mood: Mood,
    light: bool,
    /// The face, already scaled for the screen: premultiplied BGRA, `face_size` square.
    face: Vec<u8>,
    face_size: i32,
}

static POPUP: AtomicIsize = AtomicIsize::new(0);
static TIP: Mutex<Tip> = Mutex::new(Tip {
    id: String::new(),
    keys: String::new(),
    body: String::new(),
    mood: Mood::Knowing,
    light: false,
    face: Vec::new(),
    face_size: 0,
});
static ON_MUTE: OnceLock<Box<dyn Fn(&str) + Send + Sync>> = OnceLock::new();

/// Where the popup settles, and how far through the pop-in it is.
static TARGET_X: AtomicI32 = AtomicI32::new(0);
static TARGET_Y: AtomicI32 = AtomicI32::new(0);
static POP_FRAME: AtomicI32 = AtomicI32::new(0);

/// Create the hidden popup window. Must run on a thread that pumps messages.
pub unsafe fn create() -> windows::core::Result<()> {
    let class = w!("KeyflexTip");
    let wc = WNDCLASSW {
        lpfnWndProc: Some(wnd_proc),
        lpszClassName: class,
        hCursor: LoadCursorW(None, IDC_HAND)?,
        ..Default::default()
    };
    RegisterClassW(&wc);

    let hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
        class,
        w!(""),
        WS_POPUP,
        0,
        0,
        WIDTH,
        FACE,
        None,
        None,
        None,
        None,
    )?;
    SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA)?;
    // Smooth rounded corners and a shadow where Windows offers them (11 and later).
    let round = DWMWCP_ROUND;
    let _ = DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &round as *const _ as *const _, 4);
    POPUP.store(hwnd.0 as isize, Ordering::Relaxed);
    Ok(())
}

/// Called with the tip id when the user clicks "Don't show again".
pub fn on_mute(callback: impl Fn(&str) + Send + Sync + 'static) {
    ON_MUTE.set(Box::new(callback)).ok();
}

/// Whether Windows is set to light mode for apps.
pub fn system_uses_light_theme() -> bool {
    let mut value = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    let read = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut _),
            Some(&mut size),
        )
    };
    read.is_ok() && value != 0
}

/// Show a tip. Safe to call from any thread. An empty `id` hides the mute row.
pub fn show(id: &str, keys: &str, body: &str, mood: Mood, light: bool) {
    {
        let mut tip = TIP.lock().unwrap();
        tip.id = id.to_owned();
        tip.keys = keys.to_owned();
        tip.body = body.to_owned();
        tip.mood = mood;
        tip.light = light;
    }
    let hwnd = HWND(POPUP.load(Ordering::Relaxed) as *mut _);
    unsafe {
        let _ = PostMessageW(hwnd, WM_SHOW_TIP, WPARAM(0), LPARAM(0));
    }
}

fn scale(hwnd: HWND, value: i32) -> i32 {
    value * unsafe { GetDpiForWindow(hwnd) } as i32 / 96
}

fn can_mute(tip: &Tip) -> bool {
    !tip.id.is_empty() && ON_MUTE.get().is_some()
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_SHOW_TIP => {
            pop_in(hwnd);
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == POP_TIMER => {
            pop_frame(hwnd);
            LRESULT(0)
        }
        WM_TIMER => hide(hwnd),
        WM_LBUTTONDOWN => {
            let y = (lparam.0 >> 16) as i16 as i32;
            let mut client = RECT::default();
            let _ = GetClientRect(hwnd, &mut client);
            // Copy the id out first: the callback may show another popup.
            let muted_id = {
                let tip = TIP.lock().unwrap();
                (can_mute(&tip) && y >= client.bottom - scale(hwnd, FOOTER + PAD)).then(|| tip.id.clone())
            };
            if let (Some(id), Some(callback)) = (muted_id, ON_MUTE.get()) {
                callback(&id);
            }
            hide(hwnd)
        }
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_PAINT => {
            paint(hwnd);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn hide(hwnd: HWND) -> LRESULT {
    let _ = KillTimer(hwnd, HIDE_TIMER);
    let _ = KillTimer(hwnd, POP_TIMER);
    let _ = ShowWindow(hwnd, SW_HIDE);
    LRESULT(0)
}

// ── Placement and the pop-in ─────────────────────────────────────────────

/// Size the popup for its text, place it beside the cursor (flipped to the
/// other side when it would leave the screen) and start the pop-in.
unsafe fn pop_in(hwnd: HWND) {
    let mut cursor = POINT::default();
    let _ = GetCursorPos(&mut cursor);
    let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
    if !GetMonitorInfoW(monitor, &mut info).as_bool() {
        return;
    }
    let work = info.rcWork;
    let s = |v| scale(hwnd, v);

    let (w, h, border) = {
        let mut tip = TIP.lock().unwrap();
        tip.face_size = s(FACE);
        tip.face = face_pixels(tip.mood, tip.face_size);

        let text_width = s(WIDTH - PAD - FACE - FACE_GAP - PAD);
        let text_bottom = s(PAD + KEY_HEIGHT + BODY_GAP) + body_height(hwnd, &tip.body, text_width);
        let footer = if can_mute(&tip) { s(FOOTER) } else { 0 };
        let h = text_bottom.max(s(PAD + FACE)) + footer + s(PAD);
        (s(WIDTH), h, if tip.light { LIGHT.border } else { DARK.border })
    };
    let _ = DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &border as *const _ as *const _, 4);

    let gap = s(CURSOR_GAP);
    let mut x = cursor.x + gap;
    if x + w > work.right {
        x = cursor.x - gap - w;
    }
    let mut y = cursor.y + gap;
    if y + h > work.bottom {
        y = cursor.y - gap - h;
    }
    TARGET_X.store(x.max(work.left), Ordering::Relaxed);
    TARGET_Y.store(y.max(work.top), Ordering::Relaxed);
    POP_FRAME.store(0, Ordering::Relaxed);

    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA);
    let _ = SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        TARGET_X.load(Ordering::Relaxed),
        TARGET_Y.load(Ordering::Relaxed) + s(POP_RISE),
        w,
        h,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    );
    let _ = InvalidateRect(hwnd, None, true);
    SetTimer(hwnd, POP_TIMER, POP_FRAME_MS, None);
    SetTimer(hwnd, HIDE_TIMER, VISIBLE_MS, None);
}

/// One frame of the pop-in: fade up while rising, with a small overshoot.
unsafe fn pop_frame(hwnd: HWND) {
    let frame = POP_FRAME.fetch_add(1, Ordering::Relaxed) + 1;
    let t = (frame as f32 / POP_FRAMES as f32).min(1.0);
    let fade = 1.0 - (1.0 - t).powi(3);
    // "Back out" easing: goes slightly past the target, then settles.
    let overshoot = 1.0 + 2.70158 * (t - 1.0).powi(3) + 1.70158 * (t - 1.0).powi(2);

    let rise = scale(hwnd, POP_RISE) as f32 * (1.0 - overshoot);
    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), (OPACITY * fade) as u8, LWA_ALPHA);
    let _ = SetWindowPos(
        hwnd,
        None,
        TARGET_X.load(Ordering::Relaxed),
        TARGET_Y.load(Ordering::Relaxed) + rise.round() as i32,
        0,
        0,
        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
    );
    if frame >= POP_FRAMES {
        let _ = KillTimer(hwnd, POP_TIMER);
    }
}

// ── The face ─────────────────────────────────────────────────────────────

fn face_png(mood: Mood) -> &'static [u8] {
    match mood {
        Mood::Knowing => include_bytes!("../../assets/moods/knowing.png"),
        Mood::Wink => include_bytes!("../../assets/moods/wink.png"),
        Mood::Curious => include_bytes!("../../assets/moods/curious.png"),
        Mood::Excited => include_bytes!("../../assets/moods/excited.png"),
        Mood::Shocked => include_bytes!("../../assets/moods/shocked.png"),
        Mood::Cheeky => include_bytes!("../../assets/moods/cheeky.png"),
        Mood::Smug => include_bytes!("../../assets/moods/smug.png"),
        Mood::Deadpan => include_bytes!("../../assets/moods/deadpan.png"),
        Mood::Grumpy => include_bytes!("../../assets/moods/grumpy.png"),
        Mood::Pleading => include_bytes!("../../assets/moods/pleading.png"),
        Mood::Crying => include_bytes!("../../assets/moods/crying.png"),
        Mood::Dizzy => include_bytes!("../../assets/moods/dizzy.png"),
        Mood::Proud => include_bytes!("../../assets/moods/proud.png"),
        Mood::Starstruck => include_bytes!("../../assets/moods/starstruck.png"),
        Mood::Love => include_bytes!("../../assets/moods/love.png"),
        Mood::Cool => include_bytes!("../../assets/moods/cool.png"),
        Mood::Party => include_bytes!("../../assets/moods/party.png"),
        Mood::Laughing => include_bytes!("../../assets/moods/laughing.png"),
        Mood::Sleepy => include_bytes!("../../assets/moods/sleepy.png"),
    }
}

/// The face for `mood` as premultiplied BGRA, `size` pixels square. Empty if
/// the image cannot be read, in which case the popup simply has no face.
fn face_pixels(mood: Mood, size: i32) -> Vec<u8> {
    let mut decoder = png::Decoder::new(face_png(mood));
    decoder.set_transformations(png::Transformations::EXPAND);
    let Ok(mut reader) = decoder.read_info() else { return Vec::new() };
    let mut rgba = vec![0; reader.output_buffer_size()];
    let Ok(frame) = reader.next_frame(&mut rgba) else { return Vec::new() };
    if frame.color_type != png::ColorType::Rgba || frame.bit_depth != png::BitDepth::Eight {
        return Vec::new();
    }
    shrink(&rgba, frame.width as usize, frame.height as usize, size.max(1) as usize)
}

/// Box-filter a straight-alpha RGBA image down to `size` square, as premultiplied BGRA.
fn shrink(rgba: &[u8], width: usize, height: usize, size: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(size * size * 4);
    for dy in 0..size {
        let (y0, y1) = (dy * height / size, ((dy + 1) * height / size).max(dy * height / size + 1));
        for dx in 0..size {
            let (x0, x1) = (dx * width / size, ((dx + 1) * width / size).max(dx * width / size + 1));
            let (mut r, mut g, mut b, mut a) = (0u32, 0u32, 0u32, 0u32);
            for y in y0..y1.min(height) {
                for x in x0..x1.min(width) {
                    let px = &rgba[(y * width + x) * 4..][..4];
                    let alpha = px[3] as u32;
                    r += px[0] as u32 * alpha;
                    g += px[1] as u32 * alpha;
                    b += px[2] as u32 * alpha;
                    a += alpha;
                }
            }
            let count = ((y1.min(height) - y0) * (x1.min(width) - x0)).max(1) as u32;
            let premultiplied = |sum: u32| (sum / (count * 255)) as u8;
            out.extend([premultiplied(b), premultiplied(g), premultiplied(r), (a / count) as u8]);
        }
    }
    out
}

unsafe fn draw_face(hdc: HDC, tip: &Tip, x: i32, y: i32) {
    let size = tip.face_size;
    if size <= 0 || tip.face.len() != (size * size * 4) as usize {
        return;
    }
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size, // top-down
            biPlanes: 1,
            biBitCount: 32,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits = std::ptr::null_mut();
    let Ok(bitmap) = CreateDIBSection(hdc, &info, DIB_RGB_COLORS, &mut bits, None, 0) else { return };
    if !bits.is_null() {
        std::ptr::copy_nonoverlapping(tip.face.as_ptr(), bits as *mut u8, tip.face.len());
        let memory = CreateCompatibleDC(hdc);
        let previous = SelectObject(memory, bitmap);
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        let _ = AlphaBlend(hdc, x, y, size, size, memory, 0, 0, size, size, blend);
        SelectObject(memory, previous);
        let _ = DeleteDC(memory);
    }
    let _ = DeleteObject(bitmap);
}

// ── Text and keycaps ─────────────────────────────────────────────────────

/// Height the body text needs when wrapped to `width`.
unsafe fn body_height(hwnd: HWND, body: &str, width: i32) -> i32 {
    let hdc = GetDC(hwnd);
    let body_font = font(scale(hwnd, 15), 400);
    let previous = SelectObject(hdc, body_font);
    let mut rect = RECT { left: 0, top: 0, right: width, bottom: 0 };
    draw(hdc, body, &mut rect, DT_CALCRECT | DT_WORDBREAK | DT_NOPREFIX);
    SelectObject(hdc, previous);
    let _ = DeleteObject(body_font);
    ReleaseDC(hwnd, hdc);
    rect.bottom
}

/// `DrawTextW` reads its buffer even when the length is zero, so an empty
/// string must never reach it. Windows can ask for a paint before any tip exists.
unsafe fn draw(hdc: HDC, text: &str, rect: &mut RECT, format: DRAW_TEXT_FORMAT) {
    if text.is_empty() {
        return;
    }
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    DrawTextW(hdc, &mut wide, rect, format);
}

unsafe fn font(height: i32, weight: i32) -> HFONT {
    CreateFontW(
        -height,
        0,
        0,
        0,
        weight,
        0,
        0,
        0,
        DEFAULT_CHARSET.0 as u32,
        OUT_DEFAULT_PRECIS.0 as u32,
        CLIP_DEFAULT_PRECIS.0 as u32,
        CLEARTYPE_QUALITY.0 as u32,
        0,
        w!("Segoe UI"),
    )
}

/// Draw "Ctrl + J" as a row of keycaps starting at (`x`, `y`).
unsafe fn draw_keys(hwnd: HWND, hdc: HDC, palette: &Palette, keys: &str, mut x: i32, y: i32) {
    let s = |v| scale(hwnd, v);
    let key_font = font(s(13), 600);
    let face = CreateSolidBrush(palette.key);
    let outline = CreatePen(PS_SOLID, 1, palette.accent);
    let previous_font = SelectObject(hdc, key_font);
    let previous_brush = SelectObject(hdc, face);
    let previous_pen = SelectObject(hdc, outline);
    SetTextColor(hdc, palette.text);

    for key in keys.split(" + ").filter(|key| !key.is_empty()) {
        let mut measured = RECT::default();
        draw(hdc, key, &mut measured, DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX);
        let width = (measured.right + 2 * s(KEY_PAD)).max(s(KEY_HEIGHT));
        let mut cap = RECT { left: x, top: y, right: x + width, bottom: y + s(KEY_HEIGHT) };
        let _ = RoundRect(hdc, cap.left, cap.top, cap.right, cap.bottom, s(8), s(8));
        draw(hdc, key, &mut cap, DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
        x += width + s(KEY_GAP);
    }

    SelectObject(hdc, previous_pen);
    SelectObject(hdc, previous_brush);
    SelectObject(hdc, previous_font);
    let _ = DeleteObject(outline);
    let _ = DeleteObject(face);
    let _ = DeleteObject(key_font);
}

unsafe fn paint(hwnd: HWND) {
    let s = |v| scale(hwnd, v);
    let tip = TIP.lock().unwrap();
    let palette = if tip.light { &LIGHT } else { &DARK };

    let mut ps = PAINTSTRUCT::default();
    let hdc = BeginPaint(hwnd, &mut ps);
    let mut client = RECT::default();
    let _ = GetClientRect(hwnd, &mut client);

    let background = CreateSolidBrush(palette.background);
    FillRect(hdc, &client, background);
    let _ = DeleteObject(background);
    SetBkMode(hdc, TRANSPARENT);

    draw_face(hdc, &tip, s(PAD), s(PAD));

    let (left, right) = (s(PAD + FACE + FACE_GAP), client.right - s(PAD));
    draw_keys(hwnd, hdc, palette, &tip.keys, left, s(PAD));

    let body_font = font(s(15), 400);
    let small_font = font(s(12), 400);
    let previous = SelectObject(hdc, body_font);
    SetTextColor(hdc, palette.text);
    let mut body_rect = RECT { left, top: s(PAD + KEY_HEIGHT + BODY_GAP), right, bottom: client.bottom };
    draw(hdc, &tip.body, &mut body_rect, DT_LEFT | DT_WORDBREAK | DT_NOPREFIX);

    if can_mute(&tip) {
        SelectObject(hdc, small_font);
        SetTextColor(hdc, palette.muted);
        let bottom = client.bottom - s(PAD) + s(2);
        let mut mute_rect = RECT { left, top: bottom - s(18), right, bottom };
        draw(hdc, MUTE_LABEL, &mut mute_rect, DT_RIGHT | DT_SINGLELINE | DT_NOPREFIX);
    }

    SelectObject(hdc, previous);
    let _ = DeleteObject(body_font);
    let _ = DeleteObject(small_font);
    let _ = EndPaint(hwnd, &ps);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mood_has_a_face_that_decodes_and_scales() {
        for mood in Mood::ALL {
            for size in [64, 96, 160] {
                let pixels = face_pixels(mood, size);
                assert_eq!(pixels.len(), (size * size * 4) as usize, "{mood:?} at {size}");
                // Opaque in the middle, and premultiplied throughout.
                let middle = &pixels[((size / 2 * size + size / 2) * 4) as usize..][..4];
                assert_eq!(middle[3], 255, "{mood:?} at {size}");
                assert!(pixels.chunks(4).all(|px| px[0] <= px[3] && px[1] <= px[3] && px[2] <= px[3]));
            }
        }
    }
}
