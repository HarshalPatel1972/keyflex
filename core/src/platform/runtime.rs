//! Runs the core in the background: one thread for the input hooks and the
//! popup window, one for looking at clicks and asking the engine.

use std::collections::HashSet;
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
    /// Called after anything the UI displays has changed.
    pub on_change: Option<Box<dyn Fn() + Send + Sync>>,
}

struct Shared {
    engine: Mutex<Engine>,
    options: Options,
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
    let handle = Handle(Arc::new(Shared { engine: Mutex::new(engine), options }));

    let muter = handle.clone();
    popup::on_mute(move |id| muter.with(|engine| engine.set_muted(id, true)));

    let (tx, rx) = channel();
    let worker = handle.clone();
    std::thread::spawn(move || work(rx, worker));
    std::thread::spawn(move || unsafe {
        popup::create().expect("failed to create tip popup");
        watch::install(tx, combos).expect("failed to install input hooks");
        watch::pump();
    });
    handle
}

/// Turn raw input into events, ask the engine, show the tip.
fn work(rx: Receiver<Raw>, handle: Handle) {
    let inspector = Inspector::new().expect("UI Automation is unavailable");
    let verbose = handle.0.options.verbose;
    // Windows already seen at the front; anything else that appears is new.
    let mut seen: HashSet<isize> = watch::open_windows().into_iter().collect();

    for raw in rx {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());

        let shown = match raw {
            Raw::Click(point) => {
                let typing = watch::was_typing();
                let watching = |app: &str| handle.with(|engine| engine.is_watching(app));
                inspector.inspect(point, watching).and_then(|clicked| {
                    let (_, window) = watch::foreground();
                    if verbose {
                        println!("[{}] {:?} \"{}\"", clicked.app, clicked.control, clicked.name);
                    }
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
            Raw::Foreground { hwnd, by_mouse } => {
                // Only a window never seen before counts as opened, and only if the mouse did it.
                if seen.insert(hwnd) && by_mouse {
                    let app = watch::app_of_window(hwnd);
                    let class = watch::window_class(hwnd);
                    if verbose {
                        println!("[{app}] opened with the mouse ({class})");
                    }
                    let event = Event { app: &app, window: "", typing: false, class: &class, action: Action::Opened };
                    handle.with(|engine| engine.handle(&event, now))
                } else {
                    None
                }
            }
        };

        if let Some(tip) = shown {
            if verbose {
                println!("    >>> {} [{}] {}", tip.id, tip.keys, tip.line);
            }
            let id = if tip.can_mute { tip.id.as_str() } else { "" };
            handle.show_tip(id, &tip.keys, &tip.line, tip.mood);
        }
    }
}
