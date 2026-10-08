//! Decides, for each thing the user does, whether a tip should appear now.

use std::collections::HashMap;

use serde::Serialize;

use crate::rules::{Action, Event, Rule, EVERYWHERE};
use crate::state::{Settings, State};

const DAY: u64 = 24 * 60 * 60;
/// How long the character stays visibly pleased after a celebration.
const PROUD_FOR: u64 = 6 * 60 * 60;
/// How long a passing reaction (a mute, a slip, a surprise) stays on its face.
const BRIEFLY: u64 = 10 * 60;
/// After its last mention it keeps hoping this long, then is sad for as long
/// again, then moves on.
const HOPING_FOR: u64 = 3 * DAY;
/// With nothing to mention for this long, it is at peace.
const QUIET_FOR: u64 = 7 * DAY;

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
    /// The very first tip, and a new user's tray: "hi, I'm new here."
    Hello,
    /// First showing: "oh! I've got one for this."
    Idea,
    /// First showing: "hmm, how do I put this?"
    Thinking,
    /// Second showing, and when a learned shortcut is skipped: "I saw that."
    Sideeye,
    /// Second showing: "here we go again."
    Eyeroll,
    /// Last showing: "third time... not that I'm counting."
    Nervous,
    /// Last showing, and once the day's tips are used up: worn out from talking.
    Tired,
    /// Last showing: "fine. Fine."
    Pouting,
    /// It has let a shortcut go, unlearned.
    Sad,
    /// Celebration: "phew, you tried it."
    Relieved,
    /// Celebration, and when the user turns out to know a shortcut already.
    Amazed,
    /// A shortcut has stuck: flattered.
    Blushing,
    /// A shortcut has stuck: saintly.
    Angel,
    /// A milestone: five, ten, twenty-five shortcuts learned.
    Crowned,
    /// Just told "Don't show again": lips sealed.
    Zipped,
    /// Nothing to mention for a week, and at peace with it.
    Zen,
    /// Windows is not in English, so it cannot read the menus.
    Confused,
}

impl Mood {
    pub const ALL: [Mood; 36] = [
        Mood::Knowing, Mood::Wink, Mood::Curious, Mood::Excited, Mood::Shocked, Mood::Cheeky,
        Mood::Smug, Mood::Deadpan, Mood::Grumpy, Mood::Pleading, Mood::Crying, Mood::Dizzy,
        Mood::Proud, Mood::Starstruck, Mood::Love, Mood::Cool, Mood::Party, Mood::Laughing,
        Mood::Sleepy, Mood::Hello, Mood::Idea, Mood::Thinking, Mood::Sideeye, Mood::Eyeroll,
        Mood::Nervous, Mood::Tired, Mood::Pouting, Mood::Sad, Mood::Relieved, Mood::Amazed,
        Mood::Blushing, Mood::Angel, Mood::Crowned, Mood::Zipped, Mood::Zen, Mood::Confused,
    ];

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
            Mood::Hello => "hello",
            Mood::Idea => "idea",
            Mood::Thinking => "thinking",
            Mood::Sideeye => "sideeye",
            Mood::Eyeroll => "eyeroll",
            Mood::Nervous => "nervous",
            Mood::Tired => "tired",
            Mood::Pouting => "pouting",
            Mood::Sad => "sad",
            Mood::Relieved => "relieved",
            Mood::Amazed => "amazed",
            Mood::Blushing => "blushing",
            Mood::Angel => "angel",
            Mood::Crowned => "crowned",
            Mood::Zipped => "zipped",
            Mood::Zen => "zen",
            Mood::Confused => "confused",
        }
    }
}

// The faces a tip can wear at each showing, and a celebration at each stage.
// They are used in rotation, so the same face never appears twice running.
const FIRST_FACES: &[Mood] = &[Mood::Wink, Mood::Curious, Mood::Excited, Mood::Idea, Mood::Thinking];
const SECOND_FACES: &[Mood] =
    &[Mood::Cheeky, Mood::Smug, Mood::Deadpan, Mood::Shocked, Mood::Sideeye, Mood::Eyeroll];
const LAST_FACES: &[Mood] =
    &[Mood::Pleading, Mood::Crying, Mood::Grumpy, Mood::Dizzy, Mood::Tired, Mood::Pouting, Mood::Nervous];
const FIRST_USE_FACES: &[Mood] =
    &[Mood::Proud, Mood::Starstruck, Mood::Laughing, Mood::Cool, Mood::Relieved, Mood::Amazed];
const LEARNED_FACES: &[Mood] = &[Mood::Party, Mood::Love, Mood::Starstruck, Mood::Angel, Mood::Blushing];

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

/// Learned-shortcut counts worth a crown.
const MILESTONES: &[usize] = &[5, 10, 25, 50, 100];

/// Said in place of the usual cheer when a milestone is reached.
const MILESTONE_CHEERS: &[&str] = &[
    "{keys} makes {count}. {count} shortcuts you'll never dig through a menu for again. Crown's on.",
    "{count} shortcuts, all yours. {keys} was the one that did it. I'd bow, but I'm a keycap.",
];

/// Said when the last shortcut there is has been learned.
const ALL_LEARNED_CHEER: &str =
    "{keys} was the last one. That's all {count}. I have nothing left to teach you, and I'm fine. Totally fine.";

/// How sparingly tips are shown.
///
/// The aim is a friend who notices a habit and mentions it once in a while,
/// not a teacher repeating themselves: a tip waits until the long way has been
/// seen more than once, backs off for longer each time, never arrives like
/// clockwork, and steps aside once the user has shown they know the shortcut.
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
    /// How many times the long way must be seen before a tip is shown, counted
    /// afresh after each showing and each use of the shortcut.
    pub habit_sightings: u32,
    /// A new user's first few tips appear the first time, so the app is not
    /// silent while they are finding out what it does.
    pub welcome_tips: u32,
    /// After the user presses a shortcut, trust them: do not mention it again
    /// for this long, even if they go back to the long way.
    pub trust_after_use_secs: u64,
    /// Stretch each cooldown by a different amount, so reminders feel
    /// unplanned rather than scheduled.
    pub uneven: bool,
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
            habit_sightings: 2,
            welcome_tips: 2,
            trust_after_use_secs: 7 * DAY,
            uneven: true,
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
            habit_sightings: 1,
            welcome_tips: 0,
            trust_after_use_secs: 0,
            uneven: false,
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
    /// The shortcut the mood is about, when it is about one: the one it is
    /// still hoping for, or has just given up on.
    pub about: Option<String>,
}

impl Status {
    fn plain(mood: Mood) -> Self {
        Status { mood, about: None }
    }
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

/// How far the user is through a rule's steps.
#[derive(Clone, Copy, Default)]
struct Progress {
    step: usize,
    /// Times the current step has happened so far.
    count: u32,
    started: u64,
}

impl Progress {
    fn begun(&self) -> bool {
        self.step > 0 || self.count > 0
    }
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

    pub fn set_muted(&mut self, id: &str, muted: bool, now: u64) {
        if !self.rules.iter().any(|rule| rule.id == id) {
            return;
        }
        let tip = self.state.tips.entry(id.to_string()).or_default();
        if tip.muted != muted {
            tip.muted = muted;
            // Its lips stay sealed for a while; taking the mute back unseals them.
            self.state.last_mute = if muted { now } else { 0 };
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
                    keys: rule.keys(),
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
        let within = |at: u64, span: u64| at > 0 && now.saturating_sub(at) < span;
        if state.settings.paused {
            return Status::plain(Mood::Sleepy);
        }
        // Passing reactions first, the most recent kind of news on top.
        if within(state.last_mute, BRIEFLY) {
            return Status::plain(Mood::Zipped);
        }
        if within(state.last_slip, BRIEFLY) && state.last_slip >= state.last_cheer {
            return Status::plain(Mood::Sideeye);
        }
        if within(state.last_cheer, PROUD_FOR) {
            return Status::plain(Mood::Proud);
        }
        if within(state.last_knew, BRIEFLY) {
            return Status::plain(Mood::Amazed);
        }
        let today = state.recent_shows.iter().filter(|&&at| now.saturating_sub(at) < DAY).count();
        if today > 0 && today >= self.policy.max_per_day.unwrap_or(state.settings.tips_per_day) {
            return Status::plain(Mood::Tired);
        }

        // Tips shown more than once and never tried: it hopes, then it lets go.
        let ignored = || {
            self.rules.iter().filter_map(|rule| {
                let tip = state.tips.get(&rule.id)?;
                (tip.shown >= 2 && tip.used == 0 && !tip.muted).then_some((rule, tip))
            })
        };
        let has_more_to_say = |shown: u32| shown < self.policy.max_shows_per_tip;
        if let Some((rule, _)) =
            ignored().find(|(_, tip)| has_more_to_say(tip.shown) || within(tip.last_shown, HOPING_FOR))
        {
            return Status { mood: Mood::Pleading, about: Some(rule.keys()) };
        }
        if let Some((rule, _)) = ignored().find(|(_, tip)| within(tip.last_shown, 2 * HOPING_FOR)) {
            return Status { mood: Mood::Sad, about: Some(rule.keys()) };
        }

        let last_shown = state.tips.values().map(|tip| tip.last_shown).max().unwrap_or(0);
        if last_shown == 0 && state.cheers == 0 {
            return Status::plain(Mood::Hello);
        }
        let learned = state.tips.values().any(|tip| tip.used >= self.policy.learned_after_uses);
        if learned && !within(last_shown, QUIET_FOR) && !within(state.last_cheer, QUIET_FOR) {
            return Status::plain(Mood::Zen);
        }
        Status::plain(Mood::Knowing)
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
            if matches!(event.action, Action::Key(combo) if rule.is_answer(combo)) {
                let tip = state.tips.entry(rule.id.clone()).or_default();
                if tip.used < policy.learned_after_uses {
                    tip.used += 1;
                    tip.last_used = now;
                    tip.sightings = 0;
                    *dirty = true;
                    // Only celebrate shortcuts we taught, not ones the user already knew.
                    // Those earn a quiet look of surprise on the tray face instead.
                    if tip.shown == 0 {
                        state.last_knew = now;
                    } else if !tip.muted {
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

            if progress.begun() && now.saturating_sub(progress.started) > rule.within_secs {
                *progress = Progress::default();
            }
            let step = &rule.steps[progress.step];
            let matched = step.matches(event);
            if !matched && progress.step > 0 && rule.steps[0].matches(event) {
                // Not the next step, but the first one again: start over from here.
                *progress = Progress::default();
            }
            if matched || !progress.begun() && rule.steps[0].matches(event) {
                if !progress.begun() {
                    progress.started = now;
                }
                progress.count += 1;
                if progress.count >= rule.steps[progress.step].times {
                    progress.step += 1;
                    progress.count = 0;
                }
                if progress.step == rule.steps.len() {
                    *progress = Progress::default();
                    completed.get_or_insert(index);
                }
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
        let nth = self.state.cheers as usize;
        // How many shortcuts are theirs now, when this one just joined them.
        let count = self.state.tips.values().filter(|tip| tip.used >= self.policy.learned_after_uses).count();
        let milestone = learned && self.policy.retire_learned;
        let (line, crowned) = if milestone && count == self.rules.len() && count > 1 {
            (ALL_LEARNED_CHEER, true)
        } else if milestone && MILESTONES.contains(&count) {
            (MILESTONE_CHEERS[nth % MILESTONE_CHEERS.len()], true)
        } else {
            let lines = if learned { LEARNED_CHEERS } else { FIRST_USE_CHEERS };
            (lines[nth % lines.len()], false)
        };
        let keys = rule.keys();
        let line = line
            .replace("{keys}", &keys.replace(" + ", "+"))
            .replace("{topic}", &rule.topic)
            .replace("{count}", &count.to_string());
        self.state.cheers += 1;
        self.state.last_cheer = now;
        self.dirty = true;
        let mood = if crowned {
            Mood::Crowned
        } else {
            next_face(&mut self.state, if learned { LEARNED_FACES } else { FIRST_USE_FACES })
        };
        Shown { id: rule.id.clone(), keys, line, mood, can_mute: false }
    }

    /// Apply the policy to a rule whose steps were just completed.
    fn show(&mut self, index: usize, now: u64) -> Option<Shown> {
        let Engine { rules, state, policy, dirty, .. } = self;
        let rule = &rules[index];
        let tips_so_far: u32 = state.tips.values().map(|tip| tip.shown).sum();
        let tip = state.tips.entry(rule.id.clone()).or_default();

        let learned = policy.retire_learned && tip.used >= policy.learned_after_uses;
        if tip.muted || tip.shown >= policy.max_shows_per_tip && !learned {
            return None;
        }
        if learned {
            // The promise was never to bring it up again, and it won't. The tray
            // face may still give them a look, noted at most once a minute.
            if now.saturating_sub(state.last_slip) >= 60 {
                state.last_slip = now;
                *dirty = true;
            }
            return None;
        }
        // From here on the long way counts as a sighting, shown or not.
        tip.sightings = tip.sightings.saturating_add(1);
        *dirty = true;

        // They pressed the shortcut recently. They know it; let it go.
        if tip.last_used > 0 && now.saturating_sub(tip.last_used) < policy.trust_after_use_secs {
            return None;
        }
        // Once is a one-off. Wait until it looks like a habit.
        let welcome = tip.shown == 0 && tips_so_far < policy.welcome_tips;
        if !welcome && tip.sightings < policy.habit_sightings {
            return None;
        }
        if tip.shown > 0 {
            let nth = (tip.shown as usize - 1).min(policy.cooldowns_secs.len().saturating_sub(1));
            let mut cooldown = policy.cooldowns_secs.get(nth).copied().unwrap_or(0);
            if policy.uneven {
                cooldown = stretched(cooldown, &rule.id, tip.shown);
            }
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
        tip.sightings = 0;
        state.recent_shows.push(now);
        // The first tip anyone ever sees is an introduction.
        let mood = if tips_so_far == 0 && state.cheers == 0 { Mood::Hello } else { next_face(state, faces) };
        Some(Shown { id: rule.id.clone(), keys: rule.keys(), line, mood, can_mute: true })
    }
}

/// `wait` made up to 60% longer, by an amount that differs for each tip and
/// each showing but is always the same for a given one.
fn stretched(wait: u64, id: &str, shown: u32) -> u64 {
    let seed = id.bytes().fold(shown.wrapping_mul(31), |hash, byte| hash.wrapping_mul(131).wrapping_add(byte as u32));
    wait + wait * (seed % 61) as u64 / 100
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

    /// The real limits, minus the patience: a tip shows the first time the long
    /// way is seen, on an exact schedule. For tests about the other limits.
    fn impatient() -> Policy {
        Policy { habit_sightings: 1, trust_after_use_secs: 0, uneven: false, ..Policy::default() }
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

    const HABIT_RULES: &str = r#"
        [[tip]]
        id = "delete-word"
        apps = ["*"]
        shortcut = "Ctrl+Backspace"
        topic = "deleting words"
        lines = ["word line"]
        [[tip.step]]
        any = [{ held = "Backspace", while_typing = true }]

        [[tip]]
        id = "jump"
        apps = ["*"]
        shortcut = "Ctrl+Right"
        also = ["Ctrl+Left", "Home", "End"]
        keys = "Ctrl + Arrow"
        topic = "moving through text"
        lines = ["jump line"]
        [[tip.step]]
        any = [{ held = "Left" }, { held = "Right" }]

        [[tip]]
        id = "alt-tab"
        apps = ["*"]
        shortcut = "Alt+Tab"
        topic = "switching windows"
        within_secs = 45
        lines = ["switch line"]
        [[tip.step]]
        times = 4
        any = [{ switched = true }]

        [[tip]]
        id = "clipboard"
        apps = ["*"]
        shortcut = "Win+V"
        topic = "copying several things"
        within_secs = 150
        lines = ["clipboard line"]
        [[tip.step]]
        any = [{ key = "Ctrl+C" }]
        [[tip.step]]
        any = [{ key = "Ctrl+V" }]
        [[tip.step]]
        any = [{ key = "Ctrl+C" }]
        [[tip.step]]
        any = [{ key = "Ctrl+V" }]
    "#;

    fn habits() -> Engine {
        Engine::new(rules::parse(HABIT_RULES).unwrap(), State::default(), Policy::demo())
    }

    fn doing<'a>(app: &'a str, typing: bool, action: Action<'a>) -> Event<'a> {
        Event { app, window: "", typing, class: "", action }
    }

    #[test]
    fn holding_a_key_is_a_habit_but_only_while_writing() {
        let mut engine = habits();
        let backspace = Combo::parse("Backspace").unwrap();
        // Held in a game or a video editor: none of our business.
        assert_eq!(engine.handle(&doing("resolve.exe", false, Action::Held(backspace)), 0), None);
        let shown = engine.handle(&doing("winword.exe", true, Action::Held(backspace)), 1).unwrap();
        assert_eq!((shown.id.as_str(), shown.keys.as_str()), ("delete-word", "Ctrl + Backspace"));
    }

    #[test]
    fn a_tip_can_name_its_keys_and_accept_several_answers() {
        let mut engine = habits();
        let shown = engine.handle(&doing("notepad.exe", false, Action::Held(Combo::parse("Left").unwrap())), 0).unwrap();
        assert_eq!(shown.keys, "Ctrl + Arrow");
        // Home is not the tip's shortcut, but it shows the user knows a better way.
        engine.handle(&key("notepad.exe", "", "Home"), 1);
        assert_eq!(engine.state().tips["jump"].used, 1);
    }

    #[test]
    fn something_done_several_times_in_a_short_while_is_a_habit() {
        let mut engine = habits();
        let switch = |engine: &mut Engine, at| engine.handle(&doing("chrome.exe", false, Action::Switched), at);

        // Three switches, then a long pause: not a pattern.
        assert_eq!((switch(&mut engine, 0), switch(&mut engine, 5), switch(&mut engine, 10)), (None, None, None));
        assert_eq!(switch(&mut engine, 100), None, "the first three were too long ago");
        // Four inside the window.
        assert_eq!((switch(&mut engine, 105), switch(&mut engine, 110)), (None, None));
        assert_eq!(switch(&mut engine, 115).unwrap().id, "alt-tab");
    }

    #[test]
    fn ferrying_things_between_windows_suggests_clipboard_history() {
        let mut engine = habits();
        let press = |engine: &mut Engine, combo, at| engine.handle(&key("chrome.exe", "", combo), at);
        assert_eq!(press(&mut engine, "Ctrl+C", 0), None);
        assert_eq!(press(&mut engine, "Ctrl+V", 10), None);
        // Copying twice in a row just restarts the pattern.
        assert_eq!(press(&mut engine, "Ctrl+C", 20), None);
        assert_eq!(press(&mut engine, "Ctrl+C", 25), None);
        assert_eq!(press(&mut engine, "Ctrl+V", 30), None);
        assert_eq!(press(&mut engine, "Ctrl+C", 40), None);
        assert_eq!(press(&mut engine, "Ctrl+V", 50).unwrap().id, "clipboard");
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
        assert_eq!(shown.mood, Mood::Hello, "the first tip ever is an introduction");
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
        let policy = Policy { min_gap_secs: 0, max_per_day: Some(usize::MAX), ..impatient() };
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
        let policy = Policy { min_gap_secs: 0, cooldowns_secs: vec![0], ..impatient() };
        let mut engine = engine(policy);
        let ctrl_j = key("chrome.exe", "", "Ctrl+J");
        assert_eq!(engine.status(0), Status::plain(Mood::Hello));

        assert_eq!(engine.handle(&downloads(), 0).unwrap().mood, Mood::Hello);
        assert!(SECOND_FACES.contains(&engine.handle(&downloads(), 1).unwrap().mood));
        // Shown twice and never tried: the tray face starts pleading.
        assert_eq!(engine.status(2), Status { mood: Mood::Pleading, about: Some("Ctrl + J".into()) });

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
    fn a_tip_waits_for_a_habit_not_a_one_off() {
        // Past the welcome tips, with nothing else in the way.
        let policy = Policy { welcome_tips: 0, min_gap_secs: 0, uneven: false, ..Policy::default() };
        let mut engine = engine(policy);

        assert_eq!(engine.handle(&downloads(), 0), None, "seen once: could be a one-off");
        assert!(engine.handle(&downloads(), 60).is_some(), "seen twice: a habit");

        // After a showing the count starts again, on top of the cooldown.
        assert_eq!(engine.handle(&downloads(), 2 * DAY), None, "first time since the tip");
        assert!(engine.handle(&downloads(), 2 * DAY + 60).is_some());
    }

    #[test]
    fn a_new_users_first_tips_appear_straight_away() {
        let mut engine = engine(Policy { min_gap_secs: 0, ..Policy::default() });
        assert!(engine.handle(&downloads(), 0).is_some(), "the first tip is not held back");
    }

    #[test]
    fn using_the_shortcut_earns_trust_even_after_a_relapse() {
        let policy = Policy { min_gap_secs: 0, uneven: false, ..Policy::default() };
        let mut engine = engine(policy);
        assert!(engine.handle(&downloads(), 0).is_some());

        // They try the shortcut once, then go back to the menu minutes later, repeatedly.
        assert!(engine.handle(&key("chrome.exe", "", "Ctrl+J"), 100).is_some(), "celebrated");
        for minute in 1..30 {
            assert_eq!(engine.handle(&downloads(), 100 + minute * 60), None, "no nagging after a use");
        }
        // Days later, still inside the week of trust, and past the tip's own cooldown.
        assert_eq!(engine.handle(&downloads(), 100 + 6 * DAY), None);
        // Only once the week is up, and the habit is clearly back, is it mentioned again.
        assert!(engine.handle(&downloads(), 100 + 7 * DAY).is_some());
    }

    #[test]
    fn reminders_do_not_arrive_like_clockwork() {
        for (id, shown) in [("a", 1), ("browser.downloads", 1), ("browser.downloads", 2), ("explorer.rename", 3)] {
            let wait = stretched(DAY, id, shown);
            assert!((DAY..=DAY + DAY * 6 / 10).contains(&wait), "{id} {shown}: {wait}");
            assert_eq!(wait, stretched(DAY, id, shown), "the same every time for a given tip and showing");
        }
        assert_ne!(stretched(DAY, "browser.downloads", 1), stretched(DAY, "browser.downloads", 2));
        assert_ne!(stretched(DAY, "browser.downloads", 1), stretched(DAY, "explorer.rename", 1));
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
        assert_eq!(engine.handle(&key("chrome.exe", "", "Ctrl+J"), 1000), None);
        assert_eq!(engine.status(1001).mood, Mood::Amazed, "but the tray face is impressed for a while");
        assert_eq!(engine.status(1000 + BRIEFLY).mood, Mood::Hello);
    }

    #[test]
    fn muting_seals_its_lips_for_a_while() {
        let mut engine = engine(Policy::default());
        assert!(engine.handle(&downloads(), 1000).is_some());
        engine.set_muted("downloads", true, 1010);
        assert_eq!(engine.status(1020).mood, Mood::Zipped);
        assert_eq!(engine.status(1010 + BRIEFLY).mood, Mood::Knowing);
        // Taking the mute back unseals them at once.
        engine.set_muted("downloads", true, 5000);
        engine.set_muted("downloads", false, 5010);
        assert_eq!(engine.status(5020).mood, Mood::Knowing);
    }

    #[test]
    fn slipping_back_after_learning_earns_a_look_but_never_a_tip() {
        let mut engine = engine(Policy { min_gap_secs: 0, ..impatient() });
        assert!(engine.handle(&downloads(), 1000).is_some());
        let ctrl_j = key("chrome.exe", "", "Ctrl+J");
        assert!(engine.handle(&ctrl_j, 1010).is_some());
        assert!(engine.handle(&ctrl_j, 1020).is_some(), "learned");
        assert_eq!(engine.status(1030).mood, Mood::Proud);

        assert_eq!(engine.handle(&downloads(), 2000), None, "the promise holds");
        assert_eq!(engine.status(2010).mood, Mood::Sideeye);
        assert_eq!(engine.status(2000 + BRIEFLY).mood, Mood::Proud, "and the look passes");
    }

    #[test]
    fn it_is_talked_out_once_the_days_tips_are_used_up() {
        let policy = Policy { min_gap_secs: 0, max_per_day: Some(2), ..impatient() };
        let mut engine = engine(policy);
        let bin = |engine: &mut Engine, at| {
            engine.handle(&key("explorer.exe", "Documents", "Delete"), at);
            engine.handle(&key("explorer.exe", "Recycle Bin", "Delete"), at)
        };
        assert!(engine.handle(&downloads(), 1000).is_some());
        assert_ne!(engine.status(1010).mood, Mood::Tired);
        assert!(bin(&mut engine, 1020).is_some());
        assert_eq!(engine.status(1030).mood, Mood::Tired);
        assert_ne!(engine.status(1030 + DAY).mood, Mood::Tired);
    }

    #[test]
    fn an_ignored_tip_is_hoped_for_then_mourned_then_let_go() {
        let policy = Policy { min_gap_secs: 0, max_per_day: Some(usize::MAX), cooldowns_secs: vec![0], ..impatient() };
        let mut engine = engine(policy);
        for at in [1000, 1001, 1002] {
            assert!(engine.handle(&downloads(), at).is_some());
        }
        assert_eq!(engine.handle(&downloads(), 1003), None, "three mentions were all it had");
        let about = Some("Ctrl + J".to_string());
        assert_eq!(engine.status(1002 + DAY), Status { mood: Mood::Pleading, about: about.clone() });
        assert_eq!(engine.status(1002 + HOPING_FOR + DAY), Status { mood: Mood::Sad, about });
        assert_eq!(engine.status(1002 + 2 * HOPING_FOR).mood, Mood::Knowing);
    }

    #[test]
    fn a_long_quiet_spell_after_learning_is_peace() {
        let mut engine = engine(Policy { min_gap_secs: 0, ..impatient() });
        assert!(engine.handle(&downloads(), 1000).is_some());
        let ctrl_j = key("chrome.exe", "", "Ctrl+J");
        engine.handle(&ctrl_j, 1010);
        engine.handle(&ctrl_j, 1020);
        assert_eq!(engine.status(1020 + PROUD_FOR).mood, Mood::Knowing);
        assert_eq!(engine.status(1020 + QUIET_FOR).mood, Mood::Zen);
    }

    #[test]
    fn learning_the_last_shortcut_there_is_earns_a_crown() {
        let mut engine = engine(Policy { min_gap_secs: 0, ..impatient() });
        let ctrl_j = key("chrome.exe", "", "Ctrl+J");
        let shift_delete = key("explorer.exe", "Documents", "Shift+Delete");
        assert!(engine.handle(&downloads(), 1000).is_some());
        engine.handle(&ctrl_j, 1010);
        let first = engine.handle(&ctrl_j, 1020).unwrap();
        assert!(LEARNED_FACES.contains(&first.mood), "one of two: an ordinary graduation");

        engine.handle(&key("explorer.exe", "Documents", "Delete"), 2000);
        assert!(engine.handle(&key("explorer.exe", "Recycle Bin", "Delete"), 2001).is_some());
        engine.handle(&shift_delete, 2010);
        let last = engine.handle(&shift_delete, 2020).unwrap();
        assert_eq!(last.mood, Mood::Crowned);
        assert!(last.line.contains("all 2"), "{}", last.line);
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
        let mut engine = engine(Policy { min_gap_secs: 0, cooldowns_secs: vec![0], ..impatient() });
        assert!(engine.handle(&downloads(), 0).is_some());
        engine.take_dirty();

        engine.set_muted("downloads", true, 1);
        assert!(engine.take_dirty());
        assert_eq!(engine.handle(&downloads(), 1), None);
        assert!(engine.tips()[0].muted);
        engine.set_muted("no-such-tip", true, 1);
        assert!(!engine.take_dirty());
        engine.set_muted("downloads", false, 1);

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
