use crate::ai::drive_bots;
use crate::game::{asset, Action, Game, GameConfig, Language, Phase, Space, ASSETS, BOARD};
use crate::i18n::{log_line, text};
use crate::persistence::{Preferences, SaveStore, SaveSummary};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    NewGame,
    Saves,
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Overlay {
    None,
    Command,
    Assets,
    Help,
}

struct NewGameForm {
    name: String,
    bots: u8,
    rounds: u16,
    focus: usize,
}

impl Default for NewGameForm {
    fn default() -> Self {
        Self {
            name: "Player".into(),
            bots: 1,
            rounds: 100,
            focus: 0,
        }
    }
}

pub struct App {
    screen: Screen,
    overlay: Overlay,
    language: Language,
    game: Option<Game>,
    form: NewGameForm,
    command: String,
    message: String,
    should_quit: bool,
    confirm_quit: bool,
    confirm_delete: bool,
    store: SaveStore,
    saves: Vec<SaveSummary>,
    save_selection: usize,
    asset_selection: usize,
    current_save: Option<(String, String)>,
}

impl App {
    fn new(store: SaveStore) -> Self {
        let language = store.load_preferences().language;
        Self {
            screen: Screen::Home,
            overlay: Overlay::None,
            language,
            game: None,
            form: NewGameForm::default(),
            command: String::new(),
            message: String::new(),
            should_quit: false,
            confirm_quit: false,
            confirm_delete: false,
            store,
            saves: Vec::new(),
            save_selection: 0,
            asset_selection: 0,
            current_save: None,
        }
    }

    fn toggle_language(&mut self) {
        self.language = match self.language {
            Language::ZhCn => Language::En,
            Language::En => Language::ZhCn,
        };
        let _ = self.store.save_preferences(&Preferences {
            language: self.language,
        });
    }

    fn refresh_saves(&mut self) {
        match self.store.list() {
            Ok(saves) => {
                self.saves = saves;
                self.save_selection = self.save_selection.min(self.saves.len().saturating_sub(1));
            }
            Err(error) => self.message = error.to_string(),
        }
    }

    fn start_game(&mut self) {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let config = GameConfig {
            human_name: self.form.name.trim().to_string(),
            bot_count: self.form.bots,
            round_limit: self.form.rounds,
            seed,
        };
        match Game::new(config) {
            Ok(game) => {
                self.game = Some(game);
                self.current_save = None;
                self.screen = Screen::Game;
                self.overlay = Overlay::None;
                self.message.clear();
            }
            Err(error) => self.message = error.to_string(),
        }
    }

    fn apply_action(&mut self, action: Action) {
        let Some(game) = self.game.as_mut() else {
            return;
        };
        match game.apply(action).and_then(|_| drive_bots(game)) {
            Ok(()) => self.message.clear(),
            Err(error) => self.message = error.to_string(),
        }
    }

    fn save_game(&mut self, name: Option<&str>) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let result = match (name, self.current_save.as_ref()) {
            (None, Some((id, existing_name))) => self
                .store
                .overwrite(id, existing_name, game)
                .map(|_| (id.clone(), existing_name.clone())),
            (Some(name), _) => self
                .store
                .create(name, game)
                .map(|id| (id, name.trim().to_string())),
            (None, None) => {
                self.message = "save <name>".into();
                return;
            }
        };
        match result {
            Ok(save) => {
                self.current_save = Some(save);
                self.message = match self.language {
                    Language::ZhCn => "游戏已保存".into(),
                    Language::En => "Game saved".into(),
                };
            }
            Err(error) => self.message = error.to_string(),
        }
    }

    fn load_selected(&mut self) {
        let Some(summary) = self.saves.get(self.save_selection) else {
            return;
        };
        if summary.error.is_some() {
            self.message = summary.error.clone().unwrap_or_default();
            return;
        }
        match self.store.load(&summary.id) {
            Ok(envelope) => {
                let id = envelope.id.clone();
                let name = envelope.name.clone();
                self.game = Some(envelope.game);
                self.current_save = Some((id, name));
                self.screen = Screen::Game;
                self.overlay = Overlay::None;
                if let Some(game) = self.game.as_mut() {
                    if let Err(error) = drive_bots(game) {
                        self.message = error.to_string();
                    }
                }
            }
            Err(error) => self.message = error.to_string(),
        }
    }

    fn delete_selected(&mut self) {
        let Some(summary) = self.saves.get(self.save_selection) else {
            return;
        };
        if !self.confirm_delete {
            self.confirm_delete = true;
            self.message = text(self.language, "confirm_delete").into();
            return;
        }
        let id = summary.id.clone();
        match self.store.delete(&id) {
            Ok(()) => {
                if self
                    .current_save
                    .as_ref()
                    .is_some_and(|(current, _)| current == &id)
                {
                    self.current_save = None;
                }
                self.message.clear();
                self.confirm_delete = false;
                self.refresh_saves();
            }
            Err(error) => self.message = error.to_string(),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        if self.overlay == Overlay::Command {
            self.handle_command_key(key);
            return;
        }
        if self.overlay == Overlay::Assets {
            self.handle_assets_key(key);
            return;
        }
        if self.overlay == Overlay::Help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
                self.overlay = Overlay::None;
            }
            return;
        }
        match self.screen {
            Screen::Home => self.handle_home_key(key),
            Screen::NewGame => self.handle_new_game_key(key),
            Screen::Saves => self.handle_saves_key(key),
            Screen::Game => self.handle_game_key(key),
        }
    }

    fn handle_home_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('n') | KeyCode::Enter => self.screen = Screen::NewGame,
            KeyCode::Char('o') => {
                self.refresh_saves();
                self.screen = Screen::Saves;
            }
            KeyCode::Char('l') => self.toggle_language(),
            KeyCode::Char('?') => self.overlay = Overlay::Help,
            KeyCode::Char('q') | KeyCode::Esc => self.confirm_or_quit(),
            _ => {}
        }
    }

    fn handle_new_game_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.screen = Screen::Home,
            KeyCode::Tab | KeyCode::Down => self.form.focus = (self.form.focus + 1) % 3,
            KeyCode::BackTab | KeyCode::Up => self.form.focus = (self.form.focus + 2) % 3,
            KeyCode::Left => match self.form.focus {
                1 => self.form.bots = self.form.bots.saturating_sub(1).max(1),
                2 => self.form.rounds = self.form.rounds.saturating_sub(20).max(20),
                _ => {}
            },
            KeyCode::Right => match self.form.focus {
                1 => self.form.bots = (self.form.bots + 1).min(3),
                2 => self.form.rounds = (self.form.rounds + 20).min(500),
                _ => {}
            },
            KeyCode::Char('-') if self.form.focus == 2 => {
                self.form.rounds = self.form.rounds.saturating_sub(1).max(20);
            }
            KeyCode::Char('+') | KeyCode::Char('=') if self.form.focus == 2 => {
                self.form.rounds = (self.form.rounds + 1).min(500);
            }
            KeyCode::Backspace if self.form.focus == 0 => {
                self.form.name.pop();
            }
            KeyCode::Char(character)
                if self.form.focus == 0 && self.form.name.chars().count() < 24 =>
            {
                self.form.name.push(character)
            }
            KeyCode::Enter => self.start_game(),
            _ => {}
        }
    }

    fn handle_saves_key(&mut self, key: KeyEvent) {
        self.confirm_delete = false;
        match key.code {
            KeyCode::Esc => self.screen = Screen::Home,
            KeyCode::Up => self.save_selection = self.save_selection.saturating_sub(1),
            KeyCode::Down => {
                self.save_selection =
                    (self.save_selection + 1).min(self.saves.len().saturating_sub(1))
            }
            KeyCode::Enter => self.load_selected(),
            KeyCode::Char('d') => {
                self.confirm_delete = true;
                self.message = text(self.language, "confirm_delete").into();
            }
            KeyCode::Char('l') => self.toggle_language(),
            _ => {}
        }
        if key.code == KeyCode::Char('d') && self.confirm_delete {
            // A second d arrives through the next event; preserve the flag.
        }
    }

    fn handle_game_key(&mut self, key: KeyEvent) {
        let phase = self.game.as_ref().map(|g| g.phase);
        match key.code {
            KeyCode::Char(':') => {
                self.overlay = Overlay::Command;
                self.command.clear();
            }
            KeyCode::Char('?') => self.overlay = Overlay::Help,
            KeyCode::Char('l') => self.toggle_language(),
            KeyCode::Char('m') => {
                self.overlay = Overlay::Assets;
                self.asset_selection = 0;
            }
            KeyCode::Char('s') => {
                if self.current_save.is_some() {
                    self.save_game(None);
                } else {
                    self.overlay = Overlay::Command;
                    self.command = "save ".into();
                }
            }
            KeyCode::Char('r') if phase == Some(Phase::AwaitRoll) => {
                self.apply_action(Action::Roll)
            }
            KeyCode::Char('p') if matches!(phase, Some(Phase::OfferPurchase { .. })) => {
                self.apply_action(Action::Buy)
            }
            KeyCode::Char('a') if matches!(phase, Some(Phase::OfferPurchase { .. })) => {
                self.apply_action(Action::Decline)
            }
            KeyCode::Char('b') if phase == Some(Phase::Auction) => {
                if let Some(game) = self.game.as_ref() {
                    if game.auction_actor() == Some(0) {
                        let minimum = game
                            .auction
                            .as_ref()
                            .map(|a| if a.high_bid == 0 { 10 } else { a.high_bid + 10 })
                            .unwrap_or(10);
                        self.apply_action(Action::AuctionBid(minimum));
                    }
                }
            }
            KeyCode::Char('a') if phase == Some(Phase::Auction) => {
                self.apply_action(Action::AuctionPass)
            }
            KeyCode::Char('e') if phase == Some(Phase::Manage) => {
                self.apply_action(Action::EndTurn)
            }
            KeyCode::Char('q') | KeyCode::Esc => self.confirm_or_quit(),
            _ => {
                self.confirm_quit = false;
            }
        }
    }

    fn handle_assets_key(&mut self, key: KeyEvent) {
        let owned = self.owned_tiles();
        match key.code {
            KeyCode::Esc | KeyCode::Char('m') => self.overlay = Overlay::None,
            KeyCode::Up => self.asset_selection = self.asset_selection.saturating_sub(1),
            KeyCode::Down => {
                self.asset_selection = (self.asset_selection + 1).min(owned.len().saturating_sub(1))
            }
            KeyCode::Char('b') => {
                if let Some(tile) = owned.get(self.asset_selection) {
                    self.apply_action(Action::Build(*tile));
                }
            }
            KeyCode::Char('x') => {
                if let Some(tile) = owned.get(self.asset_selection) {
                    self.apply_action(Action::SellHouse(*tile));
                }
            }
            KeyCode::Char('g') => {
                if let Some(tile) = owned.get(self.asset_selection) {
                    self.apply_action(Action::Mortgage(*tile));
                }
            }
            KeyCode::Char('u') => {
                if let Some(tile) = owned.get(self.asset_selection) {
                    self.apply_action(Action::Unmortgage(*tile));
                }
            }
            _ => {}
        }
    }

    fn owned_tiles(&self) -> Vec<usize> {
        self.game
            .as_ref()
            .map(|game| {
                ASSETS
                    .iter()
                    .filter(|definition| game.assets[&definition.tile].owner == Some(0))
                    .map(|definition| definition.tile)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn handle_command_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.overlay = Overlay::None,
            KeyCode::Backspace => {
                self.command.pop();
            }
            KeyCode::Enter => {
                let command = std::mem::take(&mut self.command);
                self.overlay = Overlay::None;
                self.execute_command(command.trim());
            }
            KeyCode::Char(character) if self.command.chars().count() < 80 => {
                self.command.push(character)
            }
            _ => {}
        }
    }

    fn execute_command(&mut self, input: &str) {
        let mut parts = input.split_whitespace();
        let Some(command) = parts.next() else { return };
        let tile = || parts.clone().next().and_then(|v| v.parse::<usize>().ok());
        match command.to_ascii_lowercase().as_str() {
            "roll" => self.apply_action(Action::Roll),
            "buy" => self.apply_action(Action::Buy),
            "auction" | "pass" => {
                let phase = self.game.as_ref().map(|g| g.phase);
                self.apply_action(if phase == Some(Phase::Auction) {
                    Action::AuctionPass
                } else {
                    Action::Decline
                });
            }
            "bid" => {
                if let Some(amount) = parts.next().and_then(|v| v.parse().ok()) {
                    self.apply_action(Action::AuctionBid(amount));
                } else {
                    self.message = "bid <amount>".into();
                }
            }
            "end" => self.apply_action(Action::EndTurn),
            "payjail" => self.apply_action(Action::PayJail),
            "usecard" => self.apply_action(Action::UseJailCard),
            "build" => {
                if let Some(tile) = tile() {
                    self.apply_action(Action::Build(tile));
                } else {
                    self.message = "build <tile>".into();
                }
            }
            "sell" => {
                if let Some(tile) = tile() {
                    self.apply_action(Action::SellHouse(tile));
                } else {
                    self.message = "sell <tile>".into();
                }
            }
            "mortgage" => {
                if let Some(tile) = tile() {
                    self.apply_action(Action::Mortgage(tile));
                } else {
                    self.message = "mortgage <tile>".into();
                }
            }
            "unmortgage" => {
                if let Some(tile) = tile() {
                    self.apply_action(Action::Unmortgage(tile));
                } else {
                    self.message = "unmortgage <tile>".into();
                }
            }
            "save" => {
                let name = parts.collect::<Vec<_>>().join(" ");
                self.save_game((!name.is_empty()).then_some(name.as_str()));
            }
            "load" => {
                if let Some(id) = parts.next() {
                    match self.store.load(id) {
                        Ok(envelope) => {
                            self.current_save = Some((envelope.id.clone(), envelope.name.clone()));
                            self.game = Some(envelope.game);
                            if let Some(game) = self.game.as_mut() {
                                let _ = drive_bots(game);
                            }
                        }
                        Err(error) => self.message = error.to_string(),
                    }
                } else {
                    self.message = "load <id>".into();
                }
            }
            "status" => {
                if let Some(game) = self.game.as_ref() {
                    self.message = format!(
                        "round {}/{} · phase {:?}",
                        game.round, game.config.round_limit, game.phase
                    );
                }
            }
            "help" => self.overlay = Overlay::Help,
            "quit" => self.confirm_or_quit(),
            _ => self.message = format!("unknown command: {command}"),
        }
    }

    fn confirm_or_quit(&mut self) {
        if self.confirm_quit {
            self.should_quit = true;
        } else {
            self.confirm_quit = true;
            self.message = text(self.language, "confirm_quit").into();
        }
    }
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let store = SaveStore::discover()?;
    let mut app = App::new(store);
    let mut terminal = TerminalGuard::enter()?;
    terminal.terminal.clear()?;
    while !app.should_quit {
        terminal.terminal.draw(|frame| render(frame, &app))?;
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if app.screen == Screen::Saves
                    && key.kind == KeyEventKind::Press
                    && key.code == KeyCode::Char('d')
                {
                    if app.confirm_delete {
                        app.delete_selected();
                    } else {
                        app.confirm_delete = true;
                        app.message = text(app.language, "confirm_delete").into();
                    }
                } else {
                    app.handle_key(key);
                }
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        frame.render_widget(
            Paragraph::new(text(app.language, "resize")).alignment(Alignment::Center),
            area,
        );
        return;
    }
    match app.screen {
        Screen::Home => render_home(frame, app),
        Screen::NewGame => render_new_game(frame, app),
        Screen::Saves => render_saves(frame, app),
        Screen::Game => render_game(frame, app),
    }
    if !app.message.is_empty() {
        let status = Rect::new(
            area.x + 1,
            area.bottom().saturating_sub(2),
            area.width.saturating_sub(2),
            1,
        );
        frame.render_widget(
            Paragraph::new(app.message.as_str()).style(Style::default().fg(Color::Yellow)),
            status,
        );
    }
    match app.overlay {
        Overlay::Command => render_command(frame, app),
        Overlay::Assets => render_assets(frame, app),
        Overlay::Help => render_help(frame, app),
        Overlay::None => {}
    }
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width.min(area.width),
        height.min(area.height),
    )
}

fn render_home(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 52, 15);
    let block = Block::default()
        .title(text(app.language, "title"))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            text(app.language, "title"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("[N/Enter] {}", text(app.language, "new_game"))),
        Line::from(format!("[O] {}", text(app.language, "load_game"))),
        Line::from(format!("[L] {}", text(app.language, "language"))),
        Line::from(format!("[?] {}", text(app.language, "help"))),
        Line::from(format!("[Q] {}", text(app.language, "quit"))),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
}

fn render_new_game(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 58, 15);
    let rows = [
        format!("{}: {}", text(app.language, "player_name"), app.form.name),
        format!("{}: {}  ← →", text(app.language, "bots"), app.form.bots),
        format!(
            "{}: {}  ← → ±20, +/- ±1",
            text(app.language, "round_limit"),
            app.form.rounds
        ),
    ];
    let lines = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Line::from(Span::styled(
                row.clone(),
                if index == app.form.focus {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ))
        })
        .chain([
            Line::from(""),
            Line::from(format!(
                "[Enter] {}   [Esc] {}",
                text(app.language, "start"),
                text(app.language, "back")
            )),
        ])
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(text(app.language, "new_game"))
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn render_saves(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 72, 20);
    let items: Vec<ListItem> = if app.saves.is_empty() {
        vec![ListItem::new(text(app.language, "no_saves"))]
    } else {
        app.saves
            .iter()
            .map(|save| {
                let status = save
                    .error
                    .as_ref()
                    .map(|_| text(app.language, "corrupt").to_string())
                    .unwrap_or_else(|| format!("{} {}", text(app.language, "round"), save.round));
                ListItem::new(format!(
                    "{}  [{}]  {}  id:{}",
                    save.name, status, save.updated_at_ms, save.id
                ))
            })
            .collect()
    };
    let mut state =
        ListState::default().with_selected((!app.saves.is_empty()).then_some(app.save_selection));
    let list = List::new(items)
        .block(
            Block::default()
                .title(text(app.language, "saves"))
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut state);
    let help = Rect::new(
        area.x + 1,
        area.bottom().saturating_sub(2),
        area.width - 2,
        1,
    );
    frame.render_widget(
        Paragraph::new("↑↓ select · Enter load · d delete · Esc back"),
        help,
    );
}

fn render_game(frame: &mut Frame, app: &App) {
    let Some(game) = app.game.as_ref() else {
        return;
    };
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
        .split(frame.area());
    render_board(frame, app, game, chunks[0]);
    let side = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(chunks[1]);
    let player_lines = game
        .players
        .iter()
        .map(|player| {
            let marker = if player.id == game.current_player {
                "▶"
            } else {
                " "
            };
            let state = if player.bankrupt {
                "✕"
            } else if player.jail_turns > 0 {
                "⚿"
            } else {
                ""
            };
            Line::from(format!(
                "{marker}{} {}  ${}  {}:{}",
                player.id + 1,
                player.name,
                player.cash,
                text(app.language, "worth"),
                game.net_worth(player.id)
            ))
            .style(if player.is_human {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            })
            .patch_style(if player.bankrupt {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            })
            .spans
            .into_iter()
            .chain([Span::raw(state)])
            .collect::<Vec<_>>()
            .into()
        })
        .collect::<Vec<Line>>();
    let title = format!(
        "{} {}/{} · {:?}",
        text(app.language, "round"),
        game.round,
        game.config.round_limit,
        game.phase
    );
    frame.render_widget(
        Paragraph::new(player_lines)
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        side[0],
    );
    let log_lines = game
        .logs
        .iter()
        .rev()
        .take(side[1].height.saturating_sub(2) as usize)
        .rev()
        .map(|entry| Line::from(log_line(game, app.language, entry)))
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(log_lines)
            .block(
                Block::default()
                    .title(text(app.language, "event_log"))
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        side[1],
    );
}

fn render_board(frame: &mut Frame, app: &App, game: &Game, area: Rect) {
    let cell_w = area.width / 6;
    let cell_h = area.height / 6;
    for (index, space) in BOARD.iter().enumerate() {
        let (column, row) = match index {
            0..=5 => (index as u16, 0),
            6..=9 => (5, index as u16 - 5),
            10..=15 => (15 - index as u16, 5),
            _ => (0, 20 - index as u16),
        };
        let rect = Rect::new(
            area.x + column * cell_w,
            area.y + row * cell_h,
            cell_w.max(1),
            cell_h.max(1),
        );
        let name = match space {
            Space::Start => text(app.language, "start_tile"),
            Space::Event => text(app.language, "event_tile"),
            Space::Tax(_) => text(app.language, "tax_tile"),
            Space::Jail => text(app.language, "jail_tile"),
            Space::FreeParking => text(app.language, "free_tile"),
            Space::GoToJail => text(app.language, "go_jail_tile"),
            Space::Asset(tile) => asset(*tile).unwrap().name(app.language),
        };
        let players = game
            .players
            .iter()
            .filter(|p| !p.bankrupt && p.position == index)
            .map(|p| format!("{}", p.id + 1))
            .collect::<Vec<_>>()
            .join(" ");
        let detail = match space {
            Space::Asset(tile) => {
                let state = &game.assets[tile];
                let owner = state
                    .owner
                    .map(|id| format!("P{}", id + 1))
                    .unwrap_or_else(|| format!("${}", asset(*tile).unwrap().price));
                format!(
                    "{owner} {}",
                    if state.mortgaged {
                        "M".into()
                    } else if state.houses > 0 {
                        format!("H{}", state.houses)
                    } else {
                        String::new()
                    }
                )
            }
            Space::Tax(amount) => format!("-${amount}"),
            _ => String::new(),
        };
        let owner_style = match space {
            Space::Asset(tile) => game.assets[tile]
                .owner
                .map(player_color)
                .unwrap_or(Color::DarkGray),
            _ => Color::DarkGray,
        };
        let block = Block::default()
            .title(format!("{index} {name}"))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(owner_style));
        frame.render_widget(
            Paragraph::new(vec![Line::from(detail), Line::from(players)])
                .block(block)
                .alignment(Alignment::Center),
            rect,
        );
    }
    let center = Rect::new(area.x + cell_w, area.y + cell_h, cell_w * 4, cell_h * 4);
    let controls = match game.phase {
        Phase::AwaitRoll => "r roll · : command · s save · m assets",
        Phase::OfferPurchase { .. } => "p buy · a auction · : command",
        Phase::Auction => "b bid +10 · a pass · : bid <amount>",
        Phase::Manage => "m assets · e end turn · s save",
        Phase::GameOver => "q quit · : save <name>",
    };
    let winner = if game.phase == Phase::GameOver {
        let names = game
            .winners
            .iter()
            .map(|id| game.players[*id].name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!("\n{}: {}", text(app.language, "winner"), names)
    } else {
        String::new()
    };
    frame.render_widget(
        Paragraph::new(format!(
            "{}\n\n{}{}",
            text(app.language, "title"),
            controls,
            winner
        ))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true }),
        center,
    );
}

fn player_color(id: usize) -> Color {
    [Color::Cyan, Color::Magenta, Color::Green, Color::Yellow][id % 4]
}

fn render_command(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 70, 3);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(format!(":{}", app.command)).block(
            Block::default()
                .title(text(app.language, "command"))
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn render_assets(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 68, 18);
    frame.render_widget(Clear, area);
    let owned = app.owned_tiles();
    let items = if owned.is_empty() {
        vec![ListItem::new(if app.language == Language::ZhCn {
            "暂无地产"
        } else {
            "No assets"
        })]
    } else {
        owned
            .iter()
            .map(|tile| {
                let definition = asset(*tile).unwrap();
                let state = &app.game.as_ref().unwrap().assets[tile];
                ListItem::new(format!(
                    "#{tile} {} · H{} · {} · ${}",
                    definition.name(app.language),
                    state.houses,
                    if state.mortgaged {
                        "MORTGAGED"
                    } else {
                        "ACTIVE"
                    },
                    definition.price
                ))
            })
            .collect()
    };
    let mut state = ListState::default().with_selected(
        (!owned.is_empty()).then_some(app.asset_selection.min(owned.len().saturating_sub(1))),
    );
    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(
                    "{} · b build · x sell · g mortgage · u unmortgage · Esc",
                    text(app.language, "assets")
                ))
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().bg(Color::Blue))
        .highlight_symbol("▶ ");
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_help(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), 74, 20);
    frame.render_widget(Clear, area);
    let help = match app.language {
        Language::ZhCn => {
            "游戏目标：让其他玩家破产，或在回合上限时拥有最高净资产。\n\n\
            r 掷骰 · p 购买 · a 拒购/拍卖 · b 最小加价\n\
            m 资产管理 · e 结束回合 · s 保存 · l 切换语言\n\
            : 打开命令面板 · q 安全退出 · Esc 关闭弹窗\n\n\
            命令：roll, buy, auction, bid <金额>, end, build <格号>,\n\
            sell <格号>, mortgage <格号>, unmortgage <格号>,\n\
            payjail, usecard, save [名称], load <id>, status, help, quit\n\n\
            集齐同色地产后可均匀建房。抵押前必须卖掉该组全部房屋。\n\
            拒绝购买会触发所有未破产玩家参与的拍卖。"
        }
        Language::En => {
            "Goal: bankrupt every opponent, or have the highest net worth at the round limit.\n\n\
            r roll · p purchase · a decline/auction · b minimum bid\n\
            m asset manager · e end turn · s save · l language\n\
            : command palette · q safe quit · Esc close overlay\n\n\
            Commands: roll, buy, auction, bid <amount>, end, build <tile>,\n\
            sell <tile>, mortgage <tile>, unmortgage <tile>,\n\
            payjail, usecard, save [name], load <id>, status, help, quit\n\n\
            Complete a color group to build evenly. Sell every house in the\n\
            group before mortgaging it. Declining a purchase starts an auction."
        }
    };
    frame.render_widget(
        Paragraph::new(help)
            .block(
                Block::default()
                    .title(text(app.language, "help"))
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn renders_home_in_both_languages() {
        let root = std::env::temp_dir().join(format!("monopoly-ui-test-{}", std::process::id()));
        let mut app = App::new(SaveStore::at(root));
        for language in [Language::ZhCn, Language::En] {
            app.language = language;
            let backend = TestBackend::new(100, 30);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.draw(|frame| render(frame, &app)).unwrap();
            let content = terminal
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|cell| cell.symbol())
                .filter(|symbol| !symbol.trim().is_empty())
                .collect::<String>();
            let expected = text(language, "title").replace(' ', "");
            assert!(content.contains(&expected));
        }
    }

    #[test]
    fn renders_resize_message() {
        let app = App::new(SaveStore::at(std::env::temp_dir()));
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let content = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .filter(|symbol| !symbol.trim().is_empty())
            .collect::<String>();
        let expected = text(app.language, "resize").replace(' ', "");
        assert!(content.contains(&expected));
    }

    #[test]
    fn new_game_renders_board_and_accepts_command() {
        let root =
            std::env::temp_dir().join(format!("monopoly-ui-game-test-{}", std::process::id()));
        let mut app = App::new(SaveStore::at(root));
        app.start_game();
        app.execute_command("roll");
        assert!(app.game.as_ref().unwrap().last_roll.is_some());
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let content = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .filter(|symbol| !symbol.trim().is_empty())
            .collect::<String>();
        assert!(content.contains("晨曦巷"));
    }
}
