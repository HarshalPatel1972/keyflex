//! Headless runner for the core: watches, decides, shows the popup.
//!
//!   keyflex-core                 normal limits, tip history saved to disk
//!   keyflex-core --demo          no limits and nothing saved, for trying rules out
//!   keyflex-core --verbose       print each inspected click and each tip
//!   keyflex-core --log FILE      also append those lines to FILE
//!   keyflex-core --preview MOOD  show one sample popup with that face
//!                                (wink, cheeky, pleading, proud) and wait

use keyflex_core::engine::{Engine, Mood, Policy};
use keyflex_core::platform::runtime::{self, Options};
use keyflex_core::platform::watch;
use keyflex_core::rules;
use keyflex_core::state::State;

/// (face, tip id or "" for a celebration, line) for `--preview`.
fn sample(mood: &str) -> Option<(Mood, &'static str, &'static str)> {
    Some(match mood {
        "wink" => (Mood::Wink, "browser.downloads", "Psst. Two clicks to reach Downloads? Ctrl+J just walks in the front door."),
        "cheeky" => (Mood::Cheeky, "browser.downloads", "The menu again? Ctrl+J is right there. I am not mad. I am just... watching."),
        "pleading" => (Mood::Pleading, "browser.downloads", "Last time I will say it: Ctrl+J opens Downloads. After this, I suffer in silence."),
        "proud" => (Mood::Proud, "", "Ctrl+J! You actually did it. I am not crying, you are crying."),
        _ => return None,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let demo = args.iter().any(|arg| arg == "--demo");
    let verbose = args.iter().any(|arg| arg == "--verbose");
    let value_of = |flag: &str| args.iter().position(|arg| arg == flag).and_then(|at| args.get(at + 1));
    let preview = value_of("--preview");
    let log_path = value_of("--log").map(std::path::PathBuf::from);

    let state_path = if demo || preview.is_some() { None } else { State::default_path() };
    let state = state_path.as_deref().map(State::load).unwrap_or_default();
    let policy = if demo { Policy::demo() } else { Policy::default() };
    let engine = Engine::new(rules::builtin(), state, policy);

    watch::make_dpi_aware();
    let handle = runtime::start(engine, Options { state_path, verbose, log_path, on_change: None });
    if verbose {
        println!("keyflex-core running{}. Ctrl+C to stop.", if demo { " (demo)" } else { "" });
    }
    if let Some((mood, id, line)) = preview.and_then(|mood| sample(mood)) {
        // Give the popup thread a moment to create its window.
        std::thread::sleep(std::time::Duration::from_millis(400));
        handle.show_tip(id, "Ctrl + J", line, mood);
    }
    loop {
        std::thread::park();
    }
}
