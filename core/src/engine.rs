//! Decides, for each thing the user does, whether a tip should appear now.

use std::collections::HashMap;

use serde::Serialize;

use crate::rules::{Action, Event, Rule, EVERYWHERE};
use crate::state::{Settings, State};

const DAY: u64 = 24 * 60 * 60;
/// How long the character stays visibly pleased after a celebration.
const PROUD_FOR: u64 = 6 * 60 * 60;

/// The character's expression. Each has its own colour, as emoji do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    /// Resting face: "I know a trick."
    Knowing,
    /// First showing: "psst".
    Wink,
    /// First showing: "ooh, what's this?"
    Curious,
    /// First showing: "I know this one!"
    Excited,
    /// Second showing: "you did what, the long way?"
    Shocked,
    /// Second showing: "we talked about this."
    Cheeky,
    /// Second showing: looking away, one corner of the mouth up.
    Smug,
    /// Second showing: "..."
    Deadpan,
    /// Last showing: "hmph".
    Grumpy,
    /// Last showing, and while tips keep being ignored.
    Pleading,
    /// Last showing: full waterworks.
    Crying,
    /// Last showing: "I can't watch."
    Dizzy,
    /// The user used a shortcut we taught.
    Proud,
    /// Celebration: stars in its eyes.
    Starstruck,
    /// Celebration: hearts for eyes.
    Love,
    /// Celebration: sunglasses on.
    Cool,
    /// A shortcut has stuck for good.
    Party,
    /// Celebration: tears of joy.
    Laughing,
    /// Paused.
    Sleepy,
}

impl Mood {
    pub const ALL: [Mood; 19] = [Mood::Knowing, Mood::Wink, Mood::Curious, Mood::Excited, Mood::Shocked, Mood::Cheeky, Mood::Smug, Mood::Deadpan, Mood::Grumpy, Mood::Pleading, Mood::Crying, Mood::Dizzy, Mood::Proud, Mood::Starstruck, Mood::Love, Mood::Cool, Mood::Party, Mood::Laughing, Mood::Sleepy];

    /// The mood's name, which is also the name of its image file.
    pub fn name(self) -> &'static str {
        match self {
            Mood::Knowing => "knowing",
            Mood::Wink => "wink",
            Mood::Curious => "curious",
            Mood::Excited => "excited",
            Mood::Shocked => "shocked",
            Mood::Cheeky => "cheeky",
            Mood::Smug => "smug",
            Mood::Deadpan => "deadpan",
            Mood::Grumpy => "grumpy",
            Mood::Pleading => "pleading",
            Mood::Crying => "crying",
            Mood::Dizzy => "dizzy",
            Mood::Proud => "proud",
            Mood::Starstruck => "starstruck",
            Mood::Love => "love",
            Mood::Cool => "cool",
            Mood::Party => "party",
            Mood::Laughing => "laughing",
            Mood::Sleepy => "sleepy",
        }
    }
}

// The faces a tip can wear at each showing, and a celebration at each stage.
// They are used in rotation, so the same face never appears twice running.
const FIRST_FACES: &[Mood] = &[Mood::Wink, Mood::Curious, Mood::Excited];
const SECOND_FACES: &[Mood] = &[Mood::Cheeky, Mood::Smug, Mood::Deadpan, Mood::Shocked];
const LAST_FACES: &[Mood] = &[Mood::Pleading, Mood::Crying, Mood::Grumpy, Mood::Dizzy];
const FIRST_USE_FACES: &[Mood] = &[Mood::Proud, Mood::Starstruck, Mood::Laughing, Mood::Cool];
const LEARNED_FACES: &[Mood] = &[Mood::Party, Mood::Love, Mood::Starstruck];

/// Said the first time the user presses a shortcut after being shown its tip.
const FIRST_USE_CHEERS: &[&str] = &[
    "{keys}! You actually did it. I'm not crying, you're crying.",
    "Look at you, using {keys} like you've known it all along.",
    "{keys}. Did you feel that? That was speed.",
    "I saw that {keys}. I'm telling everyone.",
];

/// Said when the shortcut has stuck and its tip retires.
const LEARNED_CHEERS: &[&str] = &[
    "{keys} is yours now. I'll never bring up {topic} again. Promise.",
    "Graduated: {keys}. My work here is done. For {topic}, anyway.",
    "That's {keys} locked in. One less thing for me to pester you about.",
];

/// How sparingly tips are shown.
#[derive(Clone, Debug)]
pub struct Policy {
    /// A tip retires after being shown this many times.
    pub max_shows_per_tip: u32,
    /// Once the user has pressed the shortcut this often, they know it.
    pub learned_after_uses: u32,
    /// Stop suggesting a shortcut once it is learned. Off only for trying rules out.
    pub retire_learned: bool,
    /// Wait after the 1st, 2nd, ... showing of a tip before it may show again.
    /// The last entry applies to all later showings.
    pub cooldowns_secs: Vec<u64>,
    /// Minimum quiet time between any two tips.
    pub min_gap_secs: u64,
    /// Most tips in any 24 hours. `None` uses the user's own setting.
    pub max_per_day: Option<usize>,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            max_shows_per_tip: 3,
            learned_after_uses: 2,
            retire_learned: true,
            cooldowns_secs: vec![DAY, 3 * DAY, 7 * DAY],
            min_gap_secs: 10 * 60,
            max_per_day: None,
        }
    }
}

impl Policy {
    /// No limits at all: every match shows a tip. For trying rules out.
    pub fn demo() -> Self {
        Policy {
            max_shows_per_tip: u32::MAX,
            learned_after_uses: 2,
            retire_learned: false,
            cooldowns_secs: vec![0],
            min_gap_secs: 0,
            max_per_day: Some(usize::MAX),
        }
    }
}

/// Something to put on screen: a tip, or a celebration.
#[derive(Debug, PartialEq)]
pub struct Shown {
    pub id: String,
    pub keys: String,
    pub line: String,
    pub mood: Mood,
    /// Tips offer "Don't show again"; celebrations do not.
    pub can_mute: bool,
}

/// How the character is feeling overall, for the tray icon.
#[derive(Debug, PartialEq, Serialize)]
pub struct Status {
    pub mood: Mood,
    /// The shortcut it is still hoping the user will try.
    pub waiting_on: Option<String>,
}

/// One tip as the Tips screen lists it.
#[derive(Debug, Serialize)]
pub struct TipView {
    pub id: String,
    pub apps: Vec<String>,
    pub keys: String,
    pub line: String,
    pub shown: u32,
    pub last_shown: u64,
    pub muted: bool,
    /// The user has pressed the shortcut often enough that we stop suggesting it.
    pub learned: bool,
}

/// How far the user is through a multi-step rule.
#[derive(Clone, Copy, Default)]
struct Progress {
    step: usize,
    started: u64,
}

pub struct Engine {
    rules: Vec<Rule>,
    by_app: HashMap<String, Vec<usize>>,
    progress: Vec<Progress>,
    state: State,
    policy: Policy,
    dirty: bool,
}

impl Engine {
    pub fn new(rules: Vec<Rule>, state: State, policy: Policy) -> Self {
        let mut by_app: HashMap<String, Vec<usize>> = HashMap::new();
        for (index, rule) in rules.iter().enumerate() {
            for app in &rule.apps {
                by_app.entry(app.clone()).or_default().push(index);
            }
        }
        let progress = vec![Progress::default(); rules.len()];
        Engine { rules, by_app, progress, state, policy, dirty: false }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn set_settings(&mut self, settings: Settings) {
        if self.state.settings != settings {
            self.state.settings = settings;
            self.dirty = true;
        }
    }

    pub fn set_muted(&mut self, id: &str, muted: bool) {
        if !self.rules.iter().any(|rule| rule.id == id) {
            return;
        }
        let tip = self.state.tips.entry(id.to_string()).or_default();
        if tip.muted != muted {
            tip.muted = muted;
            self.dirty = true;
        }
    }

    /// Every tip with its history, in the order of the rules file.
    pub fn tips(&self) -> Vec<TipView> {
        self.rules
            .iter()
            .map(|rule| {
                let tip = self.state.tips.get(&rule.id).cloned().unwrap_or_default();
                TipView {
                    id: rule.id.clone(),
                    apps: rule.apps.clone(),
                    keys: rule.shortcut.label(),
                    line: rule.lines[0].clone(),
                    shown: tip.shown,
                    last_shown: tip.last_shown,
                    muted: tip.muted,
                    learned: tip.used >= self.policy.learned_after_uses,
                }
            })
            .collect()
    }

    /// The character's overall mood right now.
    pub fn status(&self, now: u64) -> Status {
        let state = &self.state;
        if state.settings.paused {
            return Status { mood: Mood::Sleepy, waiting_on: None };
        }
        if state.last_cheer > 0 && now.saturating_sub(state.last_cheer) < PROUD_FOR {
            return Status { mood: Mood::Proud, waiting_on: None };
        }
        // A tip shown more than once and still never tried.
        let ignored = self.rules.iter().find(|rule| {
            state.tips.get(&rule.id).is_some_and(|tip| tip.shown >= 2 && tip.used == 0 && !tip.muted)
        });
        match ignored {
            Some(rule) => Status { mood: Mood::Pleading, waiting_on: Some(rule.shortcut.label()) },
            None => Status { mood: Mood::Knowing, waiting_on: None },
        }
    }

    /// False when the user has paused Keyflex or switched this app off, or
    /// when no tip could apply to it.
    pub fn is_watching(&self, app: &str) -> bool {
        let settings = &self.state.settings;
        let has_tips = self.by_app.contains_key(app)
            || (self.by_app.contains_key(EVERYWHERE) && !settings.disabled_apps.contains(EVERYWHERE));
        has_tips && !settings.paused && !settings.disabled_apps.contains(app)
    }

    /// True once since the last call if the state changed and should be saved.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// Feed in what the user just did. `now` is seconds since the Unix epoch.
    pub fn handle(&mut self, event: &Event, now: u64) -> Option<Shown> {
        if !self.is_watching(event.app) {
            return None;
        }
        let Engine { rules, by_app, progress, state, policy, dirty } = self;
        let mut completed = None;
        let mut cheer = None;

        // The app's own tips first, so they win over the everywhere-tips.
        let own = by_app.get(event.app).map_or(&[][..], Vec::as_slice);
        let everywhere = match by_app.get(EVERYWHERE) {
            Some(indices) if !state.settings.disabled_apps.contains(EVERYWHERE) => indices.as_slice(),
            _ => &[],
        };
        for &index in own.iter().chain(everywhere) {
            let rule = &rules[index];
            let progress = &mut progress[index];

            // The user did it the better way.
            if matches!(event.action, Action::Key(combo) if combo == rule.shortcut) {
                let tip = state.tips.entry(rule.id.clone()).or_default();
                if tip.used < policy.learned_after_uses {
                    tip.used += 1;
                    *dirty = true;
                    // Only celebrate shortcuts we taught, not ones the user already knew.
                    if tip.shown > 0 && !tip.muted {
                        let learned = tip.used >= policy.learned_after_uses;
                        if learned || tip.used == 1 {
                            cheer = Some((index, learned));
                        }
                        if learned && !policy.retire_learned {
                            tip.used = 0;
                        }
                    }
                }
                *progress = Progress::default();
                continue;
            }

            if progress.step > 0 && now.saturating_sub(progress.started) > rule.within_secs {
                *progress = Progress::default();
            }
            if rule.steps[progress.step].matches(event) {
                if progress.step == 0 {
                    progress.started = now;
                }
                progress.step += 1;
                if progress.step == rule.steps.len() {
                    *progress = Progress::default();
                    completed.get_or_insert(index);
                }
            } else if progress.step > 0 && rule.steps[0].matches(event) {
                *progress = Progress { step: 1, started: now };
            }
        }

        if let Some((index, learned)) = cheer {
            return Some(self.cheer(index, learned, now));
        }
        self.show(completed?, now)
    }

    /// Celebrate the user pressing a shortcut we taught them.
    fn cheer(&mut self, index: usize, learned: bool, now: u64) -> Shown {
        let rule = &self.rules[index];
        let lines = if learned { LEARNED_CHEERS } else { FIRST_USE_CHEERS };
        let keys = rule.shortcut.label();
        let line = lines[self.state.cheers as usize % lines.len()]
            .replace("{keys}", &keys.replace(" + ", "+"))
            .replace("{topic}", &rule.topic);
        self.state.cheers += 1;
        self.state.last_cheer = now;
        self.dirty = true;
        let mood = next_face(&mut self.state, if learned { LEARNED_FACES } else { FIRST_USE_FACES });
        Shown { id: rule.id.clone(), keys, line, mood, can_mute: false }
    }

    /// Apply the policy to a rule whose steps were just completed.
    fn show(&mut self, index: usize, now: u64) -> Option<Shown> {
        let Engine { rules, state, policy, dirty, .. } = self;
        let rule = &rules[index];
        let tip = state.tips.entry(rule.id.clone()).or_default();

        let learned = policy.retire_learned && tip.used >= policy.learned_after_uses;
        if tip.muted || learned || tip.shown >= policy.max_shows_per_tip {
            return None;
        }
        if tip.shown > 0 {
            let nth = (tip.shown as usize - 1).min(policy.cooldowns_secs.len().saturating_sub(1));
            let cooldown = policy.cooldowns_secs.get(nth).copied().unwrap_or(0);
            if now.saturating_sub(tip.last_shown) < cooldown {
                return None;
            }
        }
        state.recent_shows.retain(|&at| now.saturating_sub(at) < DAY);
        let too_soon =
            state.recent_shows.last().is_some_and(|&at| now.saturating_sub(at) < policy.min_gap_secs);
        let max_per_day = policy.max_per_day.unwrap_or(state.settings.tips_per_day);
        if too_soon || state.recent_shows.len() >= max_per_day {
            return None;
        }

        // The lines, like the face, escalate: a wink, then cheek, then a last plea.
        let line = rule.lines[tip.shown as usize % rule.lines.len()].clone();
        let faces = match tip.shown {
            0 => FIRST_FACES,
            1 => SECOND_FACES,
            _ => LAST_FACES,
        };
        tip.shown += 1;
        tip.last_shown = now;
        state.recent_shows.push(now);
        *dirty = true;
        let mood = next_face(state, faces);
        Some(Shown { id: rule.id.clone(), keys: rule.shortcut.label(), line, mood, can_mute: true })
    }
}

/// The next face from `faces`. One counter steps through every set, so two
/// popups in a row never wear the same face.
fn next_face(state: &mut State, faces: &[Mood]) -> Mood {
    let mood = faces[state.faces as usize % faces.len()];
    state.faces = state.faces.wrapping_add(1);
    mood
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combo::Combo;
    use crate::rules::{self, Control};

    const RULES: &str = r#"
        [[tip]]
        id = "downloads"
        apps = ["chrome.exe"]
        shortcut = "Ctrl+J"
        topic = "Downloads"
        lines = ["first line", "second line"]
        [[tip.step]]
        any = [{ click = "downloads", types = ["MenuItem"] }]

        [[tip]]
        id = "permanent-delete"
        apps = ["explorer.exe"]
        shortcut = "Shift+Delete"
        topic = "deleting for good"
        within_secs = 180
        lines = ["skip the bin"]
        [[tip.step]]
        any = [{ key = "Delete", window_not = "Recycle Bin" }, { click = "delete", window_not = "Recycle Bin" }]
        [[tip.step]]
        any = [{ key = "Delete", window = "Recycle Bin" }, { click = "empty recycle bin" }]
    "#;

    fn engine(policy: Policy) -> Engine {
        Engine::new(rules::parse(RULES).unwrap(), State::default(), policy)
    }

    fn click<'a>(app: &'a str, window: &'a str, name: &'a str, control: Control) -> Event<'a> {
        Event { app, window, typing: false, class: "", action: Action::Click { name, control } }
    }

    fn key<'a>(app: &'a str, window: &'a str, combo: &str) -> Event<'a> {
        Event { app, window, typing: false, class: "", action: Action::Key(Combo::parse(combo).unwrap()) }
    }

    fn downloads() -> Event<'static> {
        click("chrome.exe", "", "downloads", Control::MenuItem)
    }

    const WIDE_RULES: &str = r#"
        [[tip]]
        id = "copy"
        apps = ["*"]
        shortcut = "Ctrl+C"
        topic = "copying"
        lines = ["copy line"]
        [[tip.step]]
        any = [{ is = "copy", types = ["MenuItem"] }]

        [[tip]]
        id = "bold"
        apps = ["winword.exe"]
        shortcut = "Ctrl+B"
        topic = "bold"
        lines = ["bold line"]
        [[tip.step]]
        any = [{ is = "bold", while_typing = true }]

        [[tip]]
        id = "explorer"
        apps = ["explorer.exe"]
        shortcut = "Win+E"
        topic = "File Explorer"
        lines = ["explorer line"]
        [[tip.step]]
        any = [{ opened = true, class = "CabinetWClass" }]

        [[tip]]
        id = "task-manager"
        apps = ["taskmgr.exe"]
        shortcut = "Ctrl+Shift+Esc"
        topic = "Task Manager"
        lines = ["task line"]
        [[tip.step]]
        any = [{ opened = true }]
    "#;

    fn wide() -> Engine {
        Engine::new(rules::parse(WIDE_RULES).unwrap(), State::default(), Policy::demo())
    }

    #[test]
    fn everywhere_rules_fire_in_any_app_on_the_exact_name_only() {
        let mut engine = wide();
        assert!(engine.is_watching("some-unknown-app.exe"));
        assert_eq!(engine.handle(&click("gimp.exe", "", "copy", Control::MenuItem), 0).unwrap().id, "copy");
        assert_eq!(engine.handle(&click("gimp.exe", "", "copy link address", Control::MenuItem), 1), None);
        assert_eq!(engine.handle(&click("gimp.exe", "", "copy", Control::Button), 2), None, "one-click button");

        // Pressing the shortcut in a different app still counts as knowing it.
        engine.handle(&key("notepad.exe", "", "Ctrl+C"), 3);
        assert_eq!(engine.state().tips["copy"].used, 1);

        // The user can switch the everywhere-tips off without touching named apps.
        let mut settings = engine.state().settings.clone();
        settings.disabled_apps.insert("*".into());
        engine.set_settings(settings);
        assert!(!engine.is_watching("gimp.exe"));
        assert!(engine.is_watching("winword.exe"));
        assert_eq!(engine.handle(&click("gimp.exe", "", "copy", Control::MenuItem), 4), None);
    }

    #[test]
    fn typing_only_rules_need_hands_on_the_keyboard() {
        let mut engine = wide();
        let bold = |typing| Event {
            app: "winword.exe",
            window: "",
            typing,
            class: "",
            action: Action::Click { name: "bold", control: Control::Button },
        };
        assert_eq!(engine.handle(&bold(false), 0), None, "browsing with the mouse: the button is fine");
        assert_eq!(engine.handle(&bold(true), 1).unwrap().id, "bold");
    }

    #[test]
    fn opening_an_app_with_the_mouse_is_a_trigger() {
        let mut engine = wide();
        let opened = |app, class| Event { app, window: "", typing: false, class, action: Action::Opened };
        assert_eq!(engine.handle(&opened("taskmgr.exe", "TaskManagerWindow"), 0).unwrap().id, "task-manager");
        assert_eq!(engine.handle(&opened("winword.exe", "OpusApp"), 1), None);
        // A folder window counts as opening File Explorer; the desktop does not.
        assert_eq!(engine.handle(&opened("explorer.exe", "Progman"), 2), None);
        assert_eq!(engine.handle(&opened("explorer.exe", "CabinetWClass"), 3).unwrap().id, "explorer");
    }

    #[test]
    fn single_click_rule_fires() {
        let mut engine = engine(Policy::default());
        let shown = engine.handle(&downloads(), 1000).unwrap();
        assert_eq!(shown.id, "downloads");
        assert_eq!(shown.keys, "Ctrl + J");
        assert_eq!(shown.line, "first line");
        assert!(FIRST_FACES.contains(&shown.mood));
        assert!(shown.can_mute);
        assert!(engine.take_dirty());
        assert!(!engine.take_dirty());
    }

    #[test]
    fn wrong_app_or_control_type_does_not_fire() {
        let mut engine = engine(Policy::demo());
        // A web page button that happens to be called "Downloads".
        assert_eq!(engine.handle(&click("chrome.exe", "", "downloads", Control::Button), 0), None);
        assert_eq!(engine.handle(&click("notepad.exe", "", "downloads", Control::MenuItem), 0), None);
        assert_eq!(engine.handle(&click("chrome.exe", "", "settings", Control::MenuItem), 0), None);
    }

    #[test]
    fn sequence_fires_only_in_order_and_in_time() {
        let mut engine = engine(Policy::demo());
        let delete_in_folder = key("explorer.exe", "Documents", "Delete");
        let delete_in_bin = key("explorer.exe", "Recycle Bin", "Delete");

        // Second step alone is nothing.
        assert_eq!(engine.handle(&delete_in_bin, 0), None);

        // Both steps, with unrelated activity in between.
        assert_eq!(engine.handle(&delete_in_folder, 10), None);
        assert_eq!(engine.handle(&click("explorer.exe", "Documents", "view", Control::Button), 20), None);
        assert_eq!(engine.handle(&delete_in_bin, 60).unwrap().id, "permanent-delete");

        // Progress was reset, so the second step alone is nothing again.
        assert_eq!(engine.handle(&delete_in_bin, 70), None);

        // Too slow.
        assert_eq!(engine.handle(&delete_in_folder, 100), None);
        assert_eq!(engine.handle(&delete_in_bin, 100 + 181), None);

        // Mixed alternatives: click to delete, then empty the bin from the desktop.
        assert_eq!(engine.handle(&click("explorer.exe", "Documents", "delete", Control::Button), 500), None);
        let empty = click("explorer.exe", "", "empty recycle bin", Control::MenuItem);
        assert!(engine.handle(&empty, 510).is_some());
    }

    #[test]
    fn using_the_shortcut_cancels_a_sequence_in_progress() {
        let mut engine = engine(Policy::demo());
        assert_eq!(engine.handle(&key("explorer.exe", "Documents", "Delete"), 0), None);
        assert_eq!(engine.handle(&key("explorer.exe", "Documents", "Shift+Delete"), 5), None);
        assert_eq!(engine.handle(&key("explorer.exe", "Recycle Bin", "Delete"), 10), None);
    }

    #[test]
    fn cooldown_grows_then_tip_retires_and_lines_rotate() {
        let policy = Policy { min_gap_secs: 0, max_per_day: Some(usize::MAX), ..Policy::default() };
        let mut engine = engine(policy);

        assert_eq!(engine.handle(&downloads(), 0).unwrap().line, "first line");
        assert_eq!(engine.handle(&downloads(), DAY - 1), None);
        assert_eq!(engine.handle(&downloads(), DAY).unwrap().line, "second line");
        assert_eq!(engine.handle(&downloads(), DAY + 3 * DAY - 1), None);
        assert_eq!(engine.handle(&downloads(), DAY + 3 * DAY).unwrap().line, "first line");
        // Shown three times: retired for good.
        assert_eq!(engine.handle(&downloads(), 1000 * DAY), None);
    }

    #[test]
    fn learned_shortcut_is_never_suggested() {
        let mut engine = engine(Policy::default());
        let ctrl_j = key("chrome.exe", "", "Ctrl+J");
        assert_eq!(engine.handle(&ctrl_j, 0), None);
        assert!(engine.take_dirty());
        assert_eq!(engine.handle(&ctrl_j, 1), None);
        assert!(engine.take_dirty());
        assert_eq!(engine.handle(&downloads(), 2), None);

        // Further presses are not counted, so nothing needs saving.
        assert_eq!(engine.handle(&ctrl_j, 3), None);
        assert!(!engine.take_dirty());
        assert_eq!(engine.state().tips["downloads"].used, 2);
    }

    #[test]
    fn face_escalates_then_celebrates_when_the_shortcut_is_used() {
        let policy = Policy { min_gap_secs: 0, cooldowns_secs: vec![0], ..Policy::default() };
        let mut engine = engine(policy);
        let ctrl_j = key("chrome.exe", "", "Ctrl+J");
        assert_eq!(engine.status(0), Status { mood: Mood::Knowing, waiting_on: None });

        assert!(FIRST_FACES.contains(&engine.handle(&downloads(), 0).unwrap().mood));
        assert!(SECOND_FACES.contains(&engine.handle(&downloads(), 1).unwrap().mood));
        // Shown twice and never tried: the tray face starts pleading.
        assert_eq!(engine.status(2), Status { mood: Mood::Pleading, waiting_on: Some("Ctrl + J".into()) });

        // First use is celebrated, and cannot be muted.
        let cheer = engine.handle(&ctrl_j, 3).unwrap();
        assert!(FIRST_USE_FACES.contains(&cheer.mood));
        assert!(!cheer.can_mute);
        assert!(cheer.line.contains("Ctrl+J"), "{}", cheer.line);
        assert_eq!(engine.status(4).mood, Mood::Proud);

        // Second use graduates the tip, naming what it will stop mentioning.
        let graduated = engine.handle(&ctrl_j, 5).unwrap();
        assert!(LEARNED_FACES.contains(&graduated.mood));
        assert!(graduated.line.contains("Ctrl+J"), "{}", graduated.line);
        assert_eq!(engine.handle(&downloads(), 6), None);
        assert_eq!(engine.handle(&ctrl_j, 7), None, "no more celebrations once learned");

        // The pride wears off.
        assert_eq!(engine.status(5 + PROUD_FOR).mood, Mood::Knowing);
    }

    #[test]
    fn the_same_face_never_appears_twice_running() {
        // In demo mode every match shows a tip, so one tip walks through all its stages.
        let mut engine = wide();
        let copy = click("gimp.exe", "", "copy", Control::MenuItem);
        let faces: Vec<Mood> = (0..12).map(|at| engine.handle(&copy, at).unwrap().mood).collect();
        assert!(faces.windows(2).all(|pair| pair[0] != pair[1]), "{faces:?}");
        // And the whole set gets used, not just one or two of them.
        assert!(LAST_FACES.iter().all(|face| faces.contains(face)), "{faces:?}");
    }

    #[test]
    fn every_mood_has_a_distinct_name() {
        let names: std::collections::HashSet<_> = Mood::ALL.iter().map(|mood| mood.name()).collect();
        assert_eq!(names.len(), Mood::ALL.len());
    }

    #[test]
    fn no_celebration_for_a_shortcut_the_user_already_knew() {
        let mut engine = engine(Policy::default());
        assert_eq!(engine.handle(&key("chrome.exe", "", "Ctrl+J"), 0), None);
        assert_eq!(engine.status(1).mood, Mood::Knowing);
    }

    #[test]
    fn paused_face_is_asleep() {
        let mut engine = engine(Policy::default());
        let mut settings = engine.state().settings.clone();
        settings.paused = true;
        engine.set_settings(settings);
        assert_eq!(engine.status(0).mood, Mood::Sleepy);
    }

    #[test]
    fn muted_tip_stays_silent() {
        let mut state = State::default();
        state.tips.entry("downloads".into()).or_default().muted = true;
        let mut engine = Engine::new(rules::parse(RULES).unwrap(), state, Policy::demo());
        assert_eq!(engine.handle(&downloads(), 0), None);
    }

    #[test]
    fn muting_from_the_popup_and_settings_take_effect() {
        let mut engine = engine(Policy { min_gap_secs: 0, cooldowns_secs: vec![0], ..Policy::default() });
        assert!(engine.handle(&downloads(), 0).is_some());
        engine.take_dirty();

        engine.set_muted("downloads", true);
        assert!(engine.take_dirty());
        assert_eq!(engine.handle(&downloads(), 1), None);
        assert!(engine.tips()[0].muted);
        engine.set_muted("no-such-tip", true);
        assert!(!engine.take_dirty());
        engine.set_muted("downloads", false);

        // Paused, then one app switched off, then the user's own daily limit.
        let mut settings = engine.state().settings.clone();
        settings.paused = true;
        engine.set_settings(settings.clone());
        assert_eq!(engine.handle(&downloads(), 2), None);

        settings.paused = false;
        settings.disabled_apps.insert("chrome.exe".into());
        engine.set_settings(settings.clone());
        assert_eq!(engine.handle(&downloads(), 3), None);

        settings.disabled_apps.clear();
        settings.tips_per_day = 2;
        engine.set_settings(settings);
        assert!(engine.handle(&downloads(), 4).is_some());
        assert_eq!(engine.handle(&downloads(), 5), None, "two shown today, limit is two");
    }

    #[test]
    fn global_gap_and_daily_cap_apply_across_tips() {
        let policy = Policy { min_gap_secs: 600, max_per_day: Some(2), ..Policy::demo() };
        let mut engine = engine(policy);
        let bin = |engine: &mut Engine, at| {
            engine.handle(&key("explorer.exe", "Documents", "Delete"), at);
            engine.handle(&key("explorer.exe", "Recycle Bin", "Delete"), at)
        };

        assert!(engine.handle(&downloads(), 0).is_some());
        assert_eq!(bin(&mut engine, 599), None, "inside the quiet gap");
        assert!(bin(&mut engine, 600).is_some());

        // Two shown in the last 24 hours: capped until the oldest one ages out.
        assert_eq!(engine.handle(&downloads(), 1200), None);
        assert_eq!(engine.handle(&downloads(), DAY - 1), None);
        assert!(engine.handle(&downloads(), DAY).is_some());
    }
}
