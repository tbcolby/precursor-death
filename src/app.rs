//! Game of Death shell: keys, pump speed, changed-cell drawing, and a B/S
//! rule editor. The engine, rule parser and presets are in `death-core`.

use death_core::{Automaton, Rule, Start, GRID_H, GRID_W, PRESETS};
use gam::menu::Point;
use gam::{GlyphStyle, PixelColor};
use precursor_appkit::keys::Key;
use precursor_appkit::layout::ListState;
use precursor_appkit::rng::Trng;
use precursor_appkit::screen::{Screen, MARGIN};

/// Pixels per cell edge.
const CELL: isize = 4;
const SPEED_MIN: usize = 50;
const SPEED_MAX: usize = 1000;
const SPEED_STEP: usize = 50;
/// Start used for a custom rule.
const CUSTOM_START: Start = Start::Soup(30);

#[derive(PartialEq, Eq)]
pub enum Action {
    Stay,
    Leave,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    RuleSelect,
    RuleEdit,
    Running,
    Editor,
}

pub struct App {
    screen: Screen,
    trng: Trng,
    auto: Automaton,
    view: View,
    list: ListState,
    visible: usize,
    /// Name and start of the rule being run.
    name: String,
    start: Start,
    paused: bool,
    speed_ms: usize,
    cursor: (usize, usize),
    /// The user's own rule and its editor cursor (0..9 birth, 9..18 survival).
    custom: Rule,
    edit_cursor: usize,
}

impl App {
    pub fn new(screen: Screen, trng: Trng) -> Self {
        let custom = PRESETS[0].rule();
        Self {
            screen,
            trng,
            auto: Automaton::new(custom),
            view: View::RuleSelect,
            list: ListState::default(),
            visible: 1,
            name: PRESETS[0].name.to_string(),
            start: PRESETS[0].start,
            paused: false,
            speed_ms: 200,
            cursor: (GRID_W / 2, GRID_H / 2),
            custom,
            edit_cursor: 0,
        }
    }

    pub fn screen(&self) -> &Screen { &self.screen }

    /// True while generations are advancing (the pump runs only then).
    pub fn animating(&self) -> bool { self.view == View::Running && !self.paused }

    pub fn interval_ms(&self) -> usize { self.speed_ms }

    fn entries() -> usize { PRESETS.len() + 1 }

    fn seed(&mut self) {
        // Cosmetic randomness: TRNG seeds a deterministic SimRng.
        let seed = self.trng.try_u32().unwrap_or(0x1234_5678);
        self.auto.seed(self.start, &mut precursor_appkit::entropy::SimRng::new(seed));
    }

    fn run(&mut self, name: &str, rule: Rule, start: Start) {
        self.name = name.to_string();
        self.start = start;
        self.auto.set_rule(rule);
        self.seed();
        self.paused = false;
        self.view = View::Running;
    }

    pub fn on_key(&mut self, key: Key) -> Action {
        match self.view {
            View::RuleSelect => match key {
                Key::Up => self.list.step(-1, Self::entries(), self.visible),
                Key::Down => self.list.step(1, Self::entries(), self.visible),
                Key::Enter => {
                    let i = self.list.selected;
                    if let Some(p) = PRESETS.get(i) {
                        self.run(p.name, p.rule(), p.start);
                    } else {
                        self.view = View::RuleEdit;
                    }
                }
                Key::Char('e' | 'E') => {
                    self.auto.clear();
                    self.paused = true;
                    self.view = View::Editor;
                }
                k if k.is_back() => return Action::Leave,
                _ => {}
            },
            View::RuleEdit => match key {
                Key::Left => self.edit_cursor = (self.edit_cursor + 17) % 18,
                Key::Right => self.edit_cursor = (self.edit_cursor + 1) % 18,
                Key::Up | Key::Down => self.edit_cursor = (self.edit_cursor + 9) % 18,
                Key::Enter | Key::Char(' ') => {
                    let n = (self.edit_cursor % 9) as u8;
                    if self.edit_cursor < 9 {
                        self.custom.toggle_birth(n);
                    } else {
                        self.custom.toggle_survive(n);
                    }
                }
                Key::F(1) => self.run("Custom", self.custom, CUSTOM_START),
                k if k.is_back() => self.view = View::RuleSelect,
                _ => {}
            },
            View::Running => match key {
                Key::Char(' ') => self.paused = !self.paused,
                Key::Enter if self.paused => self.auto.step(),
                Key::Right => self.speed_ms = self.speed_ms.saturating_sub(SPEED_STEP).max(SPEED_MIN),
                Key::Left => self.speed_ms = (self.speed_ms + SPEED_STEP).min(SPEED_MAX),
                // New random start for this rule.
                Key::Char('r' | 'R') => self.seed(),
                Key::Char('e' | 'E') => self.view = View::Editor,
                k if k.is_back() => self.view = View::RuleSelect,
                _ => {}
            },
            View::Editor => {
                let (w, h) = (self.auto.width(), self.auto.height());
                let (x, y) = &mut self.cursor;
                match key {
                    Key::Up => *y = y.saturating_sub(1),
                    Key::Down => *y = (*y + 1).min(h - 1),
                    Key::Left => *x = x.saturating_sub(1),
                    Key::Right => *x = (*x + 1).min(w - 1),
                    Key::Enter | Key::Char(' ') => self.auto.toggle(*x, *y),
                    Key::Char('c' | 'C') => self.auto.clear(),
                    // F1 runs; Backspace/F4 leave the editor keeping the pause state.
                    Key::F(1) => {
                        self.paused = false;
                        self.view = View::Running;
                    }
                    k if k.is_back() => self.view = View::Running,
                    _ => {}
                }
            }
        }
        Action::Stay
    }

    pub fn on_foreground(&mut self) { self.screen.refresh_size(); }

    /// One generation, drawing only the cells that changed.
    pub fn on_tick(&mut self) {
        if !self.animating() {
            return;
        }
        self.auto.step();
        let (dark, light): (Vec<_>, Vec<_>) = self.auto.changed().partition(|&(_, _, alive)| alive);
        self.screen.fill_rects(dark.iter().map(|&(x, y, _)| self.cell_rect(x, y)), PixelColor::Dark);
        self.screen.fill_rects(light.iter().map(|&(x, y, _)| self.cell_rect(x, y)), PixelColor::Light);
        self.draw_status();
        self.screen.flush();
    }

    fn origin_x(&self) -> isize { ((self.screen.size().x - GRID_W as isize * CELL) / 2).max(0) }

    fn cell_rect(&self, x: usize, y: usize) -> (Point, Point) {
        let tl = Point::new(self.origin_x() + x as isize * CELL, y as isize * CELL);
        (tl, Point::new(tl.x + CELL - 1, tl.y + CELL - 1))
    }

    fn draw_status(&self) {
        self.screen.footer(&format!(
            "{} {} g{} p{} {}ms{}",
            self.name,
            self.auto.rule().notation(),
            self.auto.generation(),
            self.auto.population(),
            self.speed_ms,
            if self.paused { "  paused" } else { "" }
        ));
    }

    fn draw_grid(&self) {
        let x0 = self.origin_x();
        let rects: Vec<(Point, Point)> = (0..GRID_H)
            .flat_map(|y| self.auto.row_runs(y).into_iter().map(move |(x, len)| (x, y, len)))
            .map(|(x, y, len)| {
                let tl = Point::new(x0 + x as isize * CELL, y as isize * CELL);
                (tl, Point::new(tl.x + len as isize * CELL - 1, tl.y + CELL - 1))
            })
            .collect();
        self.screen.fill_rects(rects, PixelColor::Dark);
    }

    fn draw_rule_editor(&self) {
        self.screen.header(&format!("Custom {}", self.custom.notation()));
        let bottom = self.screen.footer("arrows move  ⏎ toggle  F1 run  ⌫ back");
        let w = self.screen.size().x - 2 * MARGIN;
        let cell = w / 9;
        let row_h = self.screen.line_height(GlyphStyle::Regular) + 10;
        let mut y = self.screen.line_height(GlyphStyle::Bold) + 30;
        for (row, label, on) in [
            (0usize, "Birth: a dead cell with this many live neighbours comes alive", (0..9u8).map(|n| self.custom.born(n)).collect::<Vec<_>>()),
            (1, "Survival: a live cell with this many neighbours stays alive", (0..9u8).map(|n| self.custom.survives(n)).collect()),
        ] {
            y = self.screen.para(y, bottom, GlyphStyle::Small, label);
            for (n, &set) in on.iter().enumerate() {
                let tl = Point::new(MARGIN + n as isize * cell, y);
                let br = Point::new(tl.x + cell - 1, y + row_h - 1);
                let text = if set { format!("[{n}]") } else { n.to_string() };
                self.screen.text_centered(tl, br, GlyphStyle::Regular, &text);
                if self.edit_cursor == row * 9 + n {
                    self.screen.frame(tl, br, 2);
                }
            }
            y += row_h + 24;
        }
    }

    pub fn draw(&mut self) {
        self.screen.clear();
        match self.view {
            View::RuleSelect => {
                let top = self.screen.header("Game of Death");
                let bottom = self.screen.footer("↑↓ pick  ⏎ start  e editor  ⌫ quit");
                let mut items: Vec<String> =
                    PRESETS.iter().map(|p| format!("{}  {}", p.name, p.notation)).collect();
                items.push(format!("Custom  {}", self.custom.notation()));
                self.visible = self.screen.list(top, bottom, &items, &self.list);
            }
            View::RuleEdit => self.draw_rule_editor(),
            View::Running => {
                self.draw_grid();
                self.draw_status();
            }
            View::Editor => {
                self.draw_grid();
                let (tl, br) = self.cell_rect(self.cursor.0, self.cursor.1);
                self.screen.frame(Point::new(tl.x - 2, tl.y - 2), Point::new(br.x + 2, br.y + 2), 1);
                self.screen.footer(&format!(
                    "edit ({},{}) p{}  ⏎ toggle  c clear  F1 run  ⌫ back",
                    self.cursor.0,
                    self.cursor.1,
                    self.auto.population()
                ));
            }
        }
        self.screen.flush();
    }
}
