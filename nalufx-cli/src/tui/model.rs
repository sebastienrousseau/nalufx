//! The `nalufx tui` model: which screen is showing, the components on
//! each, and what each key does.
//!
//! The data: the [`Source`] (a file's returns held in memory, or the
//! tickers to fetch), shared with the job thread through an [`Arc`]; one
//! component per screen, all kept for the whole session so going back
//! finds a screen as it was left; and the last error, shown until the
//! next key.

use super::job::{self, Job, Outcome, Request, Source};
use super::keys;
use crate::cli::{Bounds, Method};
use crate::commands::{kebab, METHODS};
use crate::error::CliError;
use petitcarnets as tea;
use std::sync::Arc;
use tea::bubbles::list::{self, DefaultDelegate, DefaultItem, Item};
use tea::bubbles::{help, spinner, table, textinput};

/// The screens, in the order they come.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Choosing tickers.
    Tickers,
    /// Choosing a method, or forecast.
    Methods,
    /// The days to forecast.
    Days,
    /// A job running.
    Running,
    /// The job's table.
    Results,
}

/// The model's messages: each component's own, and a job's result.
#[derive(Debug, Clone)]
pub enum Msg {
    /// The ticker list's.
    Tickers(list::Msg),
    /// The method list's.
    Methods(list::Msg),
    /// The days input's.
    Days(textinput::Msg),
    /// The spinner's.
    Spinner(spinner::TickMsg),
    /// A job finished.
    Done(Result<Outcome, CliError>),
}

/// A ticker in the list, and whether it is chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticker {
    /// The symbol, or the file's column name.
    pub symbol: String,
    /// Whether it goes into the job.
    pub chosen: bool,
}

impl Item for Ticker {
    fn filter_value(&self) -> String {
        self.symbol.clone()
    }
}

impl DefaultItem for Ticker {
    fn title(&self) -> String {
        format!("[{}] {}", if self.chosen { "x" } else { " " }, self.symbol)
    }

    fn description(&self) -> String {
        String::new()
    }
}

/// A method, or forecasting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// `nalufx allocate --method`, and what it does.
    Method(Method, &'static str),
    /// `nalufx forecast`.
    Forecast,
}

impl Item for Choice {
    fn filter_value(&self) -> String {
        self.title()
    }
}

impl DefaultItem for Choice {
    fn title(&self) -> String {
        match self {
            Self::Method(m, _) => kebab(m.engine_name()),
            Self::Forecast => "forecast".into(),
        }
    }

    fn description(&self) -> String {
        match self {
            Self::Method(_, what) => (*what).into(),
            Self::Forecast => "each chosen ticker's returns, with a 95% interval".into(),
        }
    }
}

/// `nalufx tui`.
#[derive(Debug)]
pub struct App {
    pub(super) source: Arc<Source>,
    pub(super) bounds: Bounds,
    pub(super) risk_aversion: Option<f64>,
    pub(super) screen: Screen,
    pub(super) tickers: list::Model<Ticker, DefaultDelegate>,
    pub(super) methods: list::Model<Choice, DefaultDelegate>,
    pub(super) days: textinput::Model,
    pub(super) spinner: spinner::Model,
    pub(super) table: table::Model,
    pub(super) help: help::Model,
    pub(super) job: Option<Job>,
    pub(super) outcome: Option<Outcome>,
    pub(super) error: Option<CliError>,
    pub(super) size: tea::Size,
}

impl App {
    /// The interface over `source`, every ticker chosen.
    #[must_use]
    pub fn new(source: Source, bounds: Bounds, risk_aversion: Option<f64>) -> Self {
        Self {
            tickers: ticker_list(source.tickers()),
            methods: method_list(),
            days: days_input(),
            source: Arc::new(source),
            bounds,
            risk_aversion,
            screen: Screen::Tickers,
            spinner: spinner::Model::new().with_spinner(spinner::Spinner::DOT),
            table: table::Model::new().with_focused(true),
            help: help::Model::new(),
            job: None,
            outcome: None,
            error: None,
            size: tea::Size { width: 80, height: 24 },
        }
    }

    /// The screen showing.
    #[must_use]
    pub const fn screen(&self) -> Screen {
        self.screen
    }

    /// The last job's result, if it succeeded.
    #[must_use]
    pub const fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }

    /// The error showing, if any.
    #[must_use]
    pub const fn error(&self) -> Option<&CliError> {
        self.error.as_ref()
    }

    /// The chosen tickers, in the list's order.
    fn chosen(&self) -> Vec<String> {
        self.tickers.items().iter().filter(|t| t.chosen).map(|t| t.symbol.clone()).collect()
    }

    /// Sizes every component to the window, leaving room for the error
    /// showing.
    fn layout(&mut self) {
        let (w, h) = (usize::from(self.size.width), usize::from(self.size.height));
        let error =
            self.error.as_ref().map_or(0, |e| super::view::error_text(e, w).lines().count());
        let lists = h.saturating_sub(1 + error);
        self.tickers.set_size(w, lists);
        self.methods.set_size(w, lists);
        self.table.set_width(w.min(80));
        self.table.set_height(h.saturating_sub(7).max(3));
        self.help.set_width(w);
    }

    /// A key, on the screen showing.
    fn key(&mut self, msg: &tea::Msg<Msg>, key: &tea::Key) -> tea::Cmd<Msg> {
        if *key == "ctrl+c" {
            return tea::quit();
        }
        self.error = None;
        match self.screen {
            Screen::Tickers => self.tickers_key(msg, key),
            Screen::Methods => self.methods_key(msg, key),
            Screen::Days => self.days_key(msg, key),
            Screen::Running => quit_on(key),
            Screen::Results => self.results_key(msg, key),
        }
    }

    /// A key on the ticker list: the list's own while its filter is being
    /// typed.
    fn tickers_key(&mut self, msg: &tea::Msg<Msg>, key: &tea::Key) -> tea::Cmd<Msg> {
        if !self.tickers.setting_filter() {
            match key.to_string().as_str() {
                "q" => return tea::quit(),
                "space" => {
                    self.toggle();
                    return tea::Cmd::none();
                },
                "enter" => {
                    self.open_methods();
                    return tea::Cmd::none();
                },
                _ => {},
            }
        }
        forward(msg, pick_tickers, |m| self.tickers.update(m)).map(Msg::Tickers)
    }

    /// Adds or removes the highlighted ticker.
    fn toggle(&mut self) {
        let index = self.tickers.global_index();
        if let Some(t) = self.tickers.items().get(index).cloned() {
            self.tickers.set_item(index, Ticker { chosen: !t.chosen, ..t });
        }
    }

    /// On to the methods, once a ticker is chosen.
    fn open_methods(&mut self) {
        if self.chosen().is_empty() {
            self.error =
                Some(CliError::input("no ticker chosen", "press space on a ticker to choose it"));
        } else {
            self.screen = Screen::Methods;
        }
    }

    /// A key on the method list.
    fn methods_key(&mut self, msg: &tea::Msg<Msg>, key: &tea::Key) -> tea::Cmd<Msg> {
        if !self.methods.setting_filter() && !self.methods.is_filtered() {
            match key.to_string().as_str() {
                "q" => return tea::quit(),
                "esc" => {
                    self.screen = Screen::Tickers;
                    return tea::Cmd::none();
                },
                _ => {},
            }
        }
        if !self.methods.setting_filter() && *key == "enter" {
            return self.choose();
        }
        forward(msg, pick_methods, |m| self.methods.update(m)).map(Msg::Methods)
    }

    /// Runs the highlighted method, or asks for the days to forecast.
    fn choose(&mut self) -> tea::Cmd<Msg> {
        match self.methods.selected_item().copied() {
            Some(Choice::Method(method, _)) => self.start(Job::Allocate(method)),
            Some(Choice::Forecast) => {
                self.screen = Screen::Days;
                self.days.focus().map(Msg::Days)
            },
            None => tea::Cmd::none(),
        }
    }

    /// A key on the days input.
    fn days_key(&mut self, msg: &tea::Msg<Msg>, key: &tea::Key) -> tea::Cmd<Msg> {
        match key.to_string().as_str() {
            "esc" => {
                self.days.blur();
                self.screen = Screen::Methods;
                tea::Cmd::none()
            },
            "enter" => match parse_days(&self.days.value()) {
                Ok(days) => {
                    self.days.blur();
                    self.start(Job::Forecast { days })
                },
                Err(e) => {
                    self.error = Some(e);
                    tea::Cmd::none()
                },
            },
            _ => forward(msg, pick_days, |m| self.days.update(m)).map(Msg::Days),
        }
    }

    /// A key on the results.
    fn results_key(&mut self, msg: &tea::Msg<Msg>, key: &tea::Key) -> tea::Cmd<Msg> {
        match key.to_string().as_str() {
            "q" => tea::quit(),
            "esc" => {
                self.screen = Screen::Methods;
                tea::Cmd::none()
            },
            _ => self.table.update(msg),
        }
    }

    /// Starts `job` on its own thread, with the spinner going.
    fn start(&mut self, job: Job) -> tea::Cmd<Msg> {
        let request = Request {
            job,
            tickers: self.chosen(),
            bounds: self.bounds,
            risk_aversion: self.risk_aversion,
        };
        let source = Arc::clone(&self.source);
        self.job = Some(job);
        self.screen = Screen::Running;
        tea::batch([
            tea::Cmd::task(move || Msg::Done(job::run(&source, &request))),
            self.spinner.tick().map(Msg::Spinner),
        ])
    }

    /// A job's result: its table, or its error on the method list.
    fn done(&mut self, result: Result<Outcome, CliError>) {
        match result {
            Ok(outcome) => {
                let (columns, rows) = super::view::table_of(&outcome);
                self.table.set_columns(columns);
                self.table.set_rows(rows);
                self.table.goto_top();
                self.outcome = Some(outcome);
                self.screen = Screen::Results;
            },
            Err(e) => {
                self.error = Some(e);
                self.screen = Screen::Methods;
            },
        }
    }

    /// A message for the component showing.
    fn forward_other(&mut self, msg: &tea::Msg<Msg>) -> tea::Cmd<Msg> {
        match self.screen {
            Screen::Tickers => {
                forward(msg, pick_tickers, |m| self.tickers.update(m)).map(Msg::Tickers)
            },
            Screen::Methods => {
                forward(msg, pick_methods, |m| self.methods.update(m)).map(Msg::Methods)
            },
            Screen::Days => forward(msg, pick_days, |m| self.days.update(m)).map(Msg::Days),
            Screen::Running | Screen::Results => tea::Cmd::none(),
        }
    }
}

impl tea::Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: tea::Msg<Msg>) -> tea::Cmd<Msg> {
        let cmd = self.handle(&msg);
        self.layout();
        cmd
    }

    fn view(&self) -> tea::View {
        tea::View::new(super::view::render(self)).with_alt_screen(true)
    }
}

impl App {
    /// A message, before the layout follows what it changed.
    fn handle(&mut self, msg: &tea::Msg<Msg>) -> tea::Cmd<Msg> {
        match msg {
            tea::Msg::WindowSize(size) => {
                self.size = *size;
                tea::Cmd::none()
            },
            tea::Msg::KeyPress(key) => self.key(msg, key),
            tea::Msg::App(Msg::Spinner(tick)) if self.screen == Screen::Running => {
                self.spinner.update(&tea::Msg::App(*tick)).map(Msg::Spinner)
            },
            tea::Msg::App(Msg::Done(result)) => {
                self.done(result.clone());
                tea::Cmd::none()
            },
            _ => self.forward_other(msg),
        }
    }
}

/// `msg` passed down through `update` if it is a runtime message or the
/// child's own, as `pick` says.
fn forward<N>(
    msg: &tea::Msg<Msg>,
    pick: fn(&Msg) -> Option<N>,
    update: impl FnOnce(&tea::Msg<N>) -> tea::Cmd<N>,
) -> tea::Cmd<N> {
    msg.route(pick).map_or_else(tea::Cmd::none, |m| update(&m))
}

fn pick_tickers(m: &Msg) -> Option<list::Msg> {
    if let Msg::Tickers(x) = m {
        Some(*x)
    } else {
        None
    }
}

fn pick_methods(m: &Msg) -> Option<list::Msg> {
    if let Msg::Methods(x) = m {
        Some(*x)
    } else {
        None
    }
}

fn pick_days(m: &Msg) -> Option<textinput::Msg> {
    if let Msg::Days(x) = m {
        Some(*x)
    } else {
        None
    }
}

/// q quits; nothing else does anything.
fn quit_on(key: &tea::Key) -> tea::Cmd<Msg> {
    if *key == "q" {
        tea::quit()
    } else {
        tea::Cmd::none()
    }
}

/// The days typed, as `nalufx forecast --days` takes them.
fn parse_days(text: &str) -> Result<u16, CliError> {
    match text.trim().parse::<u16>() {
        Ok(days @ 1..=366) => Ok(days),
        _ => Err(CliError::input(
            format!("`{}` is not a number of days", text.trim()),
            "type a whole number from 1 to 366",
        )),
    }
}

/// The tickers, every one chosen, with the app's keys in the list's help.
fn ticker_list(symbols: &[String]) -> list::Model<Ticker, DefaultDelegate> {
    let items = symbols.iter().map(|s| Ticker { symbol: s.clone(), chosen: true }).collect();
    let mut l = list::Model::new(items, compact(), 0, 0);
    l.title = "Tickers".into();
    l.set_status_bar_item_name("ticker", "tickers");
    l.additional_short_help_keys = Some(keys::ticker_keys);
    l.disable_quit_keybindings();
    l
}

/// The methods, then forecast.
fn method_list() -> list::Model<Choice, DefaultDelegate> {
    let mut items: Vec<Choice> = METHODS.iter().map(|&(m, what)| Choice::Method(m, what)).collect();
    items.push(Choice::Forecast);
    let mut l = list::Model::new(items, DefaultDelegate::new(), 0, 0);
    l.title = "Method".into();
    l.set_status_bar_item_name("method", "methods");
    l.additional_short_help_keys = Some(keys::method_keys);
    l.disable_quit_keybindings();
    l
}

/// One line per item, no gap.
fn compact() -> DefaultDelegate {
    let mut d = DefaultDelegate::new();
    d.show_description = false;
    d.set_spacing(0);
    d
}

/// The days input, with `nalufx forecast`'s default.
fn days_input() -> textinput::Model {
    let mut t = textinput::Model::new();
    t.prompt = "Days to forecast: ".into();
    t.char_limit = 3;
    t.set_value("6");
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_are_those_forecast_takes() {
        assert_eq!(parse_days(" 6 ").unwrap(), 6);
        assert_eq!(parse_days("366").unwrap(), 366);
        for bad in ["0", "367", "", "six", "-1"] {
            assert_eq!(parse_days(bad).unwrap_err().exit_code(), 3, "{bad}");
        }
    }
}
