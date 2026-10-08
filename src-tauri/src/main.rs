// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! The Keyflex app: a tray icon around the core, plus a window that exists
//! only while it is open, so the background cost stays that of the core alone.

mod startup;

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use keyflex_core::engine::{Engine, Mood, Policy, Status, TipView};
use keyflex_core::platform::runtime::{self, Handle, Options};
use keyflex_core::rules;
use keyflex_core::state::{Settings, State};
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, Wry};
use tauri_plugin_autostart::MacosLauncher;
use windows::Win32::Globalization::GetUserDefaultUILanguage;

/// The primary-language part of a Windows language id, and its value for English.
const PRIMARY_LANGUAGE_MASK: u16 = 0x3ff;
const LANG_ENGLISH: u16 = 0x09;

/// The tray face can change without any event (pride wears off), so it is rechecked this often.
const TRAY_REFRESH: Duration = Duration::from_secs(10 * 60);

struct PauseItem(CheckMenuItem<Wry>);

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

// ── Commands for the window ──────────────────────────────────────────────

#[tauri::command]
fn get_tips(core: tauri::State<Handle>) -> Vec<TipView> {
    core.with(|engine| engine.tips())
}

#[tauri::command]
fn set_tip_muted(core: tauri::State<Handle>, id: String, muted: bool) {
    core.with(|engine| engine.set_muted(&id, muted));
}

#[tauri::command]
fn get_settings(core: tauri::State<Handle>) -> Settings {
    core.with(|engine| engine.state().settings.clone())
}

#[tauri::command]
fn set_settings(core: tauri::State<Handle>, settings: Settings) {
    core.with(|engine| engine.set_settings(settings));
}

/// Every app that has tips, sorted.
#[tauri::command]
fn get_apps(core: tauri::State<Handle>) -> Vec<String> {
    let mut apps: Vec<String> = core.with(|engine| rules::apps(engine.rules())).into_iter().collect();
    apps.sort();
    apps
}

// `async` runs these off the main thread: the Store's startup API blocks while it asks Windows.
#[tauri::command(async)]
fn get_autostart(app: AppHandle) -> bool {
    startup::is_enabled(&app)
}

#[tauri::command(async)]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    startup::set_enabled(&app, enabled)
}

/// Tips match the English names of buttons and menus, so they only work when
/// Windows itself is displayed in English.
#[tauri::command]
fn windows_is_english() -> bool {
    unsafe { GetUserDefaultUILanguage() & PRIMARY_LANGUAGE_MASK == LANG_ENGLISH }
}

/// Show what a tip looks like, beside the cursor.
#[tauri::command]
fn preview_tip(core: tauri::State<Handle>) {
    core.show_tip("", "Ctrl + J", "Psst. This is where I pop up: right beside what you just did.", Mood::Wink);
}

/// How the character is feeling, for the sidebar.
#[tauri::command]
fn get_status(core: tauri::State<Handle>) -> Status {
    core.with(|engine| engine.status(now()))
}

// ── Window and tray ──────────────────────────────────────────────────────

fn open_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.set_focus();
        return;
    }
    // KEYFLEX_SCREEN=tips (or settings, intro) opens straight on that screen, for checking the UI.
    let page = match std::env::var("KEYFLEX_SCREEN") {
        Ok(screen) if screen.chars().all(|c| c.is_ascii_lowercase()) => format!("index.html#{screen}"),
        _ => "index.html".to_string(),
    };
    let built = WebviewWindowBuilder::new(app, "main", WebviewUrl::App(page.into()))
        .title("Keyflex")
        .inner_size(1000.0, 680.0)
        .min_inner_size(930.0, 600.0)
        // The UI draws its own title bar; keep the system shadow and rounded corners.
        .decorations(false)
        .shadow(true)
        .center()
        .build();
    if let Err(error) = built {
        eprintln!("could not open the Keyflex window: {error}");
    }
}

fn setup_tray(app: &AppHandle, paused: bool) -> tauri::Result<()> {
    let open = MenuItemBuilder::with_id("open", "Open Keyflex").build(app)?;
    let pause = CheckMenuItemBuilder::with_id("pause", "Pause tips").checked(paused).build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app).item(&open).item(&pause).separator().item(&quit).build()?;
    app.manage(PauseItem(pause));

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Keyflex")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => open_window(app),
            "pause" => {
                let paused = app.state::<PauseItem>().0.is_checked().unwrap_or(false);
                app.state::<Handle>().with(|engine| {
                    let mut settings = engine.state().settings.clone();
                    settings.paused = paused;
                    engine.set_settings(settings);
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                open_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn tray_face(mood: Mood) -> &'static [u8] {
    match mood {
        Mood::Knowing => include_bytes!("../icons/tray/knowing.png"),
        Mood::Wink => include_bytes!("../icons/tray/wink.png"),
        Mood::Curious => include_bytes!("../icons/tray/curious.png"),
        Mood::Excited => include_bytes!("../icons/tray/excited.png"),
        Mood::Shocked => include_bytes!("../icons/tray/shocked.png"),
        Mood::Cheeky => include_bytes!("../icons/tray/cheeky.png"),
        Mood::Smug => include_bytes!("../icons/tray/smug.png"),
        Mood::Deadpan => include_bytes!("../icons/tray/deadpan.png"),
        Mood::Grumpy => include_bytes!("../icons/tray/grumpy.png"),
        Mood::Pleading => include_bytes!("../icons/tray/pleading.png"),
        Mood::Crying => include_bytes!("../icons/tray/crying.png"),
        Mood::Dizzy => include_bytes!("../icons/tray/dizzy.png"),
        Mood::Proud => include_bytes!("../icons/tray/proud.png"),
        Mood::Starstruck => include_bytes!("../icons/tray/starstruck.png"),
        Mood::Love => include_bytes!("../icons/tray/love.png"),
        Mood::Cool => include_bytes!("../icons/tray/cool.png"),
        Mood::Party => include_bytes!("../icons/tray/party.png"),
        Mood::Laughing => include_bytes!("../icons/tray/laughing.png"),
        Mood::Sleepy => include_bytes!("../icons/tray/sleepy.png"),
    }
}

/// The tray icon wears the character's current mood, with a tooltip to match.
fn refresh_tray(app: &AppHandle) {
    let (Some(core), Some(tray)) = (app.try_state::<Handle>(), app.tray_by_id("main")) else { return };
    let status = core.with(|engine| engine.status(now()));
    let tooltip = match (&status.mood, &status.waiting_on) {
        (Mood::Sleepy, _) => "Keyflex is napping (paused)".to_string(),
        (Mood::Proud, _) => "Keyflex is proud of you".to_string(),
        (Mood::Pleading, Some(keys)) => format!("Keyflex is still hoping you'll try {keys}"),
        _ => "Keyflex is keeping an eye out".to_string(),
    };
    if let Ok(icon) = Image::from_bytes(tray_face(status.mood)) {
        let _ = tray.set_icon(Some(icon));
    }
    let _ = tray.set_tooltip(Some(tooltip));
}

/// Tell the window and the tray that tips or settings changed.
fn announce_change(app: &AppHandle) {
    let _ = app.emit("changed", ());
    refresh_tray(app);
    if let (Some(core), Some(pause)) = (app.try_state::<Handle>(), app.try_state::<PauseItem>()) {
        let paused = core.with(|engine| engine.state().settings.paused);
        let _ = pause.0.set_checked(paused);
    }
}

fn main() {
    tauri::Builder::default()
        // A second copy would install a second set of hooks and show every tip twice.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| open_window(app)))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![startup::AUTOSTART_ARG])))
        .setup(|app| {
            let handle = app.handle().clone();

            let state_path = State::default_path();
            let state = state_path.as_deref().map(State::load).unwrap_or_default();
            let paused = state.settings.paused;
            // KEYFLEX_DEMO lifts every limit so each match shows a tip, for trying rules out.
            let policy = if std::env::var_os("KEYFLEX_DEMO").is_some() { Policy::demo() } else { Policy::default() };
            let engine = Engine::new(rules::builtin(), state, policy);

            let announcer = handle.clone();
            let on_change: Box<dyn Fn() + Send + Sync> = Box::new(move || announce_change(&announcer));
            // KEYFLEX_LOG=<file> records which buttons and menu items are clicked, for checking tips.
            let log_path = std::env::var_os("KEYFLEX_LOG").map(PathBuf::from);
            let options = Options { state_path, verbose: false, log_path, on_change: Some(on_change) };
            let core = runtime::start(engine, options);
            app.manage(core);

            setup_tray(&handle, paused)?;
            refresh_tray(&handle);
            let ticker = handle.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(TRAY_REFRESH);
                refresh_tray(&ticker);
            });
            if !startup::launched_at_sign_in() {
                open_window(&handle);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_tips,
            set_tip_muted,
            get_settings,
            set_settings,
            get_apps,
            get_autostart,
            set_autostart,
            preview_tip,
            get_status,
            windows_is_english,
        ])
        .build(tauri::generate_context!())
        .expect("error starting Keyflex")
        .run(|_app, event| {
            // Closing the window must not quit: Keyflex lives in the tray.
            if let RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
