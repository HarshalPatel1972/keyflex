//! Runs the core in the background: one thread for the input hooks and the
//! popup window, one for looking at clicks and asking the engine.

use std::collections::HashSet;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::engine::{Engine, Mood};
use crate::platform::popup;
use crate::platform::watch::{self, Inspector, Raw};
use crate::rules::{self, Action, Event};
use crate::state::Theme;

#[derive(Default)]
pub struct Options {
    /// Where tip history and settings are saved. `None` keeps them in memory only.
    pub state_path: Option<PathBuf>,
    /// Print each inspected click and each tip.
    pub verbose: bool,
    /// Also append those lines to this file. For checking tips against real
    /// apps: it records the names of the buttons and menu items clicked, so it
    /// is never on unless someone asks for it.
    pub log_path: Option<PathBuf>,
    /// Called after anything the UI displays has changed.
    pub on_change: Option<Box<dyn Fn() + Send + Sync>>,
}

struct Shared {
    engine: Mutex<Engine>,
    options: Options,
}

impl Options {
    fn note(&self, line: impl FnOnce() -> String) {
        if !self.verbose && self.log_path.is_none() {
            return;
        }
        let line = line();
        if self.verbose {
            println!("{line}");
        }
        if let Some(path) = &self.log_path {
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(file, "{line}");
            }
        }
    }
}

/// Access to the running engine from other threads, e.g. the settings UI.
#[derive(Clone)]
pub struct Handle(Arc<Shared>);

impl Handle {
    /// Run `f` on the engine. Anything it changed is saved and announced.
    pub fn with<R>(&self, f: impl FnOnce(&mut Engine) -> R) -> R {
        let mut engine = self.0.engine.lock().unwrap();
        let result = f(&mut engine);
        let changed = engine.take_dirty();
        if changed {
            if let Some(path) = &self.0.options.state_path {
                if let Err(error) = engine.state().save(path) {
                    eprintln!("could not save tip history: {error}");
                }
            }
        }
        drop(engine);
        if changed {
            if let Some(on_change) = &self.0.options.on_change {
                on_change();
            }
        }
        result
    }

    /// Put a tip on screen in the user's theme. An empty `id` cannot be muted.
    pub fn show_tip(&self, id: &str, keys: &str, body: &str, mood: Mood) {
        let light = match self.with(|engine| engine.state().settings.theme) {
            Theme::Light => true,
            Theme::Dark => false,
            Theme::System => popup::system_uses_light_theme(),
        };
        popup::show(id, keys, body, mood, light);
    }
}

pub fn start(engine: Engine, options: Options) -> Handle {
    let combos = rules::combos(engine.rules());
    let held = rules::held_keys(engine.rules());
    let handle = Handle(Arc::new(Shared { engine: Mutex::new(engine), options }));

    let muter = handle.clone();
    popup::on_mute(move |id| muter.with(|engine| engine.set_muted(id, true)));

    let (tx, rx) = channel();
    let worker = handle.clone();
    std::thread::spawn(move || work(rx, worker));
    let notes = handle.clone();
    std::thread::spawn(move || unsafe {
        // Keep going with whatever works: the window and settings are still useful.
        if let Err(error) = popup::create() {
            notes.0.options.note(|| format!("tips cannot be shown: {}", error.message()));
        }
        if let Err(error) = watch::install(tx, combos, held) {
            notes.0.options.note(|| format!("cannot watch for input: {}", error.message()));
            return;
        }
        watch::pump();
    });
    handle
}

/// Turn raw input into events, ask the engine, show the tip.
fn work(rx: Receiver<Raw>, handle: Handle) {
    let options = &handle.0.options;
    // Without UI Automation, clicks cannot be read; key and window tips still work.
    let inspector = match Inspector::new() {
        Ok(inspector) => Some(inspector),
        Err(error) => {
            options.note(|| format!("clicks cannot be inspected: {}", error.message()));
            None
        }
    };
    // Windows already seen at the front; anything else that appears is new.
    let mut seen: HashSet<isize> = watch::open_windows().into_iter().collect();
    // Whether the user's latest click landed on the taskbar.
    let mut clicked_taskbar = false;

    for raw in rx {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());

        let shown = match raw {
            Raw::Click(point) => {
                clicked_taskbar = watch::on_taskbar(point);
                let typing = watch::was_typing();
                let watching = |app: &str| handle.with(|engine| engine.is_watching(app));
                inspector.as_ref().and_then(|inspector| inspector.inspect(point, watching)).and_then(|clicked| {
                    let (_, window) = watch::foreground();
                    options.note(|| format!("[{}] {:?} \"{}\"", clicked.app, clicked.control, clicked.name));
                    let action = Action::Click { name: &clicked.name, control: clicked.control };
                    let event = Event { app: &clicked.app, window: &window, typing, class: "", action };
                    handle.with(|engine| engine.handle(&event, now))
                })
            }
            Raw::Key(combo) => {
                let (app, window) = watch::foreground();
                let event =
                    Event { app: &app, window: &window, typing: false, class: "", action: Action::Key(combo) };
                handle.with(|engine| engine.handle(&event, now))
            }
            Raw::Held(combo) => {
                let (app, window) = watch::foreground();
                let typing = watch::was_typing();
                options.note(|| format!("[{app}] held {}", combo.label()));
                let event = Event { app: &app, window: &window, typing, class: "", action: Action::Held(combo) };
                handle.with(|engine| engine.handle(&event, now))
            }
            Raw::Minimized { hwnd } => {
                let app = watch::app_of_window(hwnd);
                options.note(|| format!("[{app}] minimised with the mouse"));
                let event = Event { app: &app, window: "", typing: false, class: "", action: Action::Minimized };
                handle.with(|engine| engine.handle(&event, now))
            }
            Raw::Foreground { hwnd, by_mouse } => {
                // A window never seen before counts as opened, if the mouse did it.
                if !seen.insert(hwnd) {
                    // An existing window, brought forward from the taskbar. Clicking the
                    // taskbar also brings the taskbar itself forward for a moment; that
                    // and Windows' other shell surfaces are not app windows.
                    let app = watch::app_of_window(hwnd);
                    let shell = app == "explorer.exe" && watch::window_class(hwnd) != "CabinetWClass";
                    if by_mouse && clicked_taskbar && !shell {
                        options.note(|| format!("[{app}] switched to from the taskbar"));
                        let event = Event { app: &app, window: "", typing: false, class: "", action: Action::Switched };
                        handle.with(|engine| engine.handle(&event, now))
                    } else {
                        None
                    }
                } else if by_mouse {
                    let app = watch::app_of_window(hwnd);
                    let class = watch::window_class(hwnd);
                    options.note(|| format!("[{app}] opened with the mouse ({class})"));
                    let event = Event { app: &app, window: "", typing: false, class: &class, action: Action::Opened };
                    handle.with(|engine| engine.handle(&event, now))
                } else {
                    None
                }
            }
        };

        if let Some(tip) = shown {
            options.note(|| format!("    >>> {} [{}] {}", tip.id, tip.keys, tip.line));
            let id = if tip.can_mute { tip.id.as_str() } else { "" };
            handle.show_tip(id, &tip.keys, &tip.line, tip.mood);
        }
    }
}
