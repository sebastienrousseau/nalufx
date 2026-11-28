//! The keys `nalufx tui` adds to the components' own, and the help bar
//! for the screens that are not lists.

use super::model::Screen;
use petitcarnets::bubbles::help::KeyMap;
use petitcarnets::bubbles::key::Binding;
use petitcarnets::bubbles::table;

/// Space: add or remove the highlighted ticker.
pub(crate) fn toggle() -> Binding {
    Binding::new(&["space"]).with_help("space", "choose")
}

/// Enter: go on.
pub(crate) fn next() -> Binding {
    Binding::new(&["enter"]).with_help("enter", "next")
}

/// Enter on the method list: run it.
pub(crate) fn run() -> Binding {
    Binding::new(&["enter"]).with_help("enter", "run")
}

/// Esc: the screen before.
pub(crate) fn back() -> Binding {
    Binding::new(&["esc"]).with_help("esc", "back")
}

/// q or Ctrl-C: quit.
pub(crate) fn quit() -> Binding {
    Binding::new(&["q", "ctrl+c"]).with_help("q", "quit")
}

/// Ctrl-C alone: quit where q is text.
pub(crate) fn force_quit() -> Binding {
    Binding::new(&["ctrl+c"]).with_help("ctrl+c", "quit")
}

/// The ticker list's keys, after the list's own.
pub(crate) fn ticker_keys() -> Vec<Binding> {
    vec![toggle(), next()]
}

/// The method list's keys, after the list's own.
pub(crate) fn method_keys() -> Vec<Binding> {
    vec![run(), back()]
}

/// The help bar of a screen that is not a list.
pub(crate) struct ScreenKeys<'a> {
    /// The screen.
    pub(crate) screen: Screen,
    /// The results table's keys.
    pub(crate) table: &'a table::KeyMap,
}

impl KeyMap for ScreenKeys<'_> {
    fn short_help(&self) -> Vec<Binding> {
        match self.screen {
            Screen::Days => vec![run(), back(), force_quit()],
            Screen::Running => vec![quit()],
            Screen::Results => {
                let mut keys = self.table.short_help();
                keys.extend([back(), quit()]);
                keys
            },
            Screen::Tickers | Screen::Methods => Vec::new(),
        }
    }

    fn full_help(&self) -> Vec<Vec<Binding>> {
        vec![self.short_help()]
    }
}
