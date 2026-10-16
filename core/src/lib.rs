//! Keyflex core: notices a manual action and decides whether to show the
//! better way.
//!
//! `combo`, `rules`, `state` and `engine` are plain Rust with no OS calls, so
//! the decision logic is unit-tested. `platform` holds the Windows side: the
//! input hooks, the UI Automation lookup, the popup window and the check for
//! moments it should stay quiet.

pub mod combo;
pub mod engine;
pub mod rules;
pub mod state;

#[cfg(windows)]
pub mod platform {
    pub mod popup;
    pub mod quiet;
    pub mod runtime;
    pub mod watch;
}
