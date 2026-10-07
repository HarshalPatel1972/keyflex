//! The tip rules: what the user has to do for a tip to fire, and what we say.
//! Rules live in `tips.toml`, which is compiled into the binary.

use std::collections::HashSet;

use serde::Deserialize;

use crate::combo::Combo;

/// The kinds of UI element we ever look at.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize)]
pub enum Control {
    Button,
    MenuItem,
    SplitButton,
}

/// In a rule's `apps`, matches every app.
pub const EVERYWHERE: &str = "*";

/// Apps whose whole interface is drawn as a web page (Electron and the like).
/// For these, a button "inside a document" is still the app's own interface.
pub const WEB_UI_APPS: &[&str] = &["code.exe", "cursor.exe", "slack.exe", "discord.exe", "ms-teams.exe"];

/// Something the user just did.
pub struct Event<'a> {
    /// Lowercase exe name, e.g. `chrome.exe`.
    pub app: &'a str,
    /// Title of the active window.
    pub window: &'a str,
    /// The user was typing moments ago, so their hands are on the keyboard.
    pub typing: bool,
    /// For `Opened`, the kind of window (its Windows class name). Empty otherwise.
    pub class: &'a str,
    pub action: Action<'a>,
}

pub enum Action<'a> {
    Click {
        /// Element name as [`normalize`] returns it.
        name: &'a str,
        control: Control,
    },
    Key(Combo),
    /// A new window of `app` was opened using only the mouse.
    Opened,
}

/// An element's name reduced to what it says: lowercase, without the `&`
/// mnemonic marker, a trailing shortcut ("Copy\tCtrl+C", "Downloads Ctrl+J")
/// or a trailing ellipsis.
pub fn normalize(raw: &str) -> String {
    let mut name = raw.replace('&', "").to_lowercase();
    if let Some(tab) = name.find('\t') {
        name.truncate(tab);
    }
    for marker in [" ctrl+", " alt+", " shift+", " win+"] {
        if let Some(at) = name.find(marker) {
            name.truncate(at);
        }
    }
    name.trim().trim_end_matches("...").trim_end_matches('\u{2026}').trim().to_string()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub apps: Vec<String>,
    pub shortcut: Combo,
    /// What the tip is about, as it reads mid-sentence: "Downloads", "renaming".
    pub topic: String,
    #[serde(default = "default_within_secs")]
    pub within_secs: u64,
    pub lines: Vec<String>,
    #[serde(rename = "step")]
    pub steps: Vec<Step>,
}

fn default_within_secs() -> u64 {
    120
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub any: Vec<Matcher>,
}

impl Step {
    pub fn matches(&self, event: &Event) -> bool {
        self.any.iter().any(|m| m.matches(event))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Matcher {
    /// A clicked element whose name starts with this.
    click: Option<String>,
    /// A clicked element whose name is exactly this.
    is: Option<String>,
    key: Option<Combo>,
    /// A new window of the app, opened using only the mouse.
    #[serde(default)]
    opened: bool,
    #[serde(default)]
    types: Vec<Control>,
    /// Only when the user was typing just before, so the mouse was a detour.
    #[serde(default)]
    while_typing: bool,
    window: Option<String>,
    window_not: Option<String>,
    /// With `opened`: only this kind of window, e.g. a folder window but not the desktop.
    class: Option<String>,
}

impl Matcher {
    fn matches(&self, event: &Event) -> bool {
        let action_ok = match &event.action {
            Action::Click { name, control } => {
                let named = self.click.as_deref().is_some_and(|prefix| name.starts_with(prefix))
                    || self.is.as_deref() == Some(*name);
                named && (self.types.is_empty() || self.types.contains(control))
            }
            Action::Key(combo) => self.key == Some(*combo),
            Action::Opened => self.opened,
        };
        action_ok
            && (!self.while_typing || event.typing)
            && self.class.as_deref().is_none_or(|class| event.class == class)
            && self.window.as_deref().is_none_or(|text| event.window.contains(text))
            && self.window_not.as_deref().is_none_or(|text| !event.window.contains(text))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleFile {
    tip: Vec<Rule>,
}

/// The rules shipped with the app.
pub fn builtin() -> Vec<Rule> {
    parse(include_str!("../tips.toml")).expect("tips.toml is invalid")
}

pub fn parse(text: &str) -> Result<Vec<Rule>, String> {
    let mut rules = toml::from_str::<RuleFile>(text).map_err(|e| e.to_string())?.tip;
    let mut ids = HashSet::new();
    for rule in &mut rules {
        let id = &rule.id;
        if !ids.insert(id.clone()) {
            return Err(format!("duplicate tip id \"{id}\""));
        }
        if rule.apps.is_empty() || rule.lines.is_empty() || rule.steps.is_empty() {
            return Err(format!("tip \"{id}\" needs at least one app, line and step"));
        }
        for app in &mut rule.apps {
            *app = app.to_lowercase();
        }
        for matcher in rule.steps.iter_mut().flat_map(|step| &mut step.any) {
            let triggers = [matcher.click.is_some(), matcher.is.is_some(), matcher.key.is_some(), matcher.opened];
            if triggers.iter().filter(|set| **set).count() != 1 {
                return Err(format!(
                    "tip \"{id}\": each alternative needs exactly one of click, is, key or opened"
                ));
            }
            for name in [&mut matcher.click, &mut matcher.is].into_iter().flatten() {
                *name = name.to_lowercase();
            }
        }
    }
    Ok(rules)
}

/// Every app named by a tip. Contains [`EVERYWHERE`] when some tip applies to all apps.
pub fn apps(rules: &[Rule]) -> HashSet<String> {
    rules.iter().flat_map(|rule| rule.apps.iter().cloned()).collect()
}

/// Every key combination some rule cares about. Nothing else leaves the keyboard hook.
pub fn combos(rules: &[Rule]) -> HashSet<u32> {
    let step_keys = rules
        .iter()
        .flat_map(|rule| &rule.steps)
        .flat_map(|step| &step.any)
        .filter_map(|matcher| matcher.key);
    rules.iter().map(|rule| rule.shortcut).chain(step_keys).map(Combo::code).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_rules_are_valid() {
        let rules = builtin();
        assert!(!rules.is_empty());
        assert!(apps(&rules).contains("chrome.exe"));
        assert!(combos(&rules).contains(&Combo::parse("Ctrl+J").unwrap().code()));
        // A key used only as a step, never as a shortcut, is still watched.
        assert!(combos(&rules).contains(&Combo::parse("Delete").unwrap().code()));
    }

    #[test]
    fn names_are_reduced_to_what_they_say() {
        assert_eq!(normalize("&Copy\tCtrl+C"), "copy");
        assert_eq!(normalize("Downloads Ctrl+J"), "downloads");
        assert_eq!(normalize("Print..."), "print");
        assert_eq!(normalize("Save as\u{2026}"), "save as");
        assert_eq!(normalize("Copy link address"), "copy link address");
        assert_eq!(normalize("  Select All "), "select all");
    }

    #[test]
    fn rejects_bad_files() {
        let tip = |body: &str| {
            format!(
                "[[tip]]\nid = \"a\"\napps = [\"x.exe\"]\nshortcut = \"F2\"\ntopic = \"x\"\nlines = [\"hi\"]\n{body}"
            )
        };
        assert!(parse(&tip("[[tip.step]]\nany = [{ click = \"go\" }]")).is_ok());
        assert!(parse(&tip("")).is_err(), "no steps");
        assert!(parse(&tip("[[tip.step]]\nany = [{ types = [\"Button\"] }]")).is_err(), "no click or key");
        assert!(parse(&tip("[[tip.step]]\nany = [{ click = \"go\", key = \"F2\" }]")).is_err(), "both");
        assert!(parse(&tip("[[tip.step]]\nany = [{ is = \"Go\" }]")).is_ok());
        assert!(parse(&tip("[[tip.step]]\nany = [{ opened = true }]")).is_ok());
        assert!(parse(&tip("[[tip.step]]\nany = [{ click = \"go\", colour = 1 }]")).is_err(), "typo");
        let twice = tip("[[tip.step]]\nany = [{ click = \"go\" }]\n").repeat(2);
        assert!(parse(&twice).is_err(), "duplicate id");
    }
}
