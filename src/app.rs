//! State machine for Game of Death.
//! States: RuleSelect, Running, Editor

extern crate alloc;
use alloc::vec::Vec;
use crate::automaton::{self, Grid, Rule, GRID_W, GRID_H};

const KEY_UP: char = '\u{F700}';
const KEY_DOWN: char = '\u{F701}';
const KEY_LEFT: char = '\u{F702}';
const KEY_RIGHT: char = '\u{F703}';
const KEY_ENTER: char = '\u{000D}';
const KEY_BACKSPACE: char = '\u{0008}';
const KEY_MENU: char = '\u{2234}';

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState { RuleSelect, Running, Editor }

pub struct DeathApp {
    pub state: AppState,
    pub needs_redraw: bool,
    pub grid: Grid,
    pub rules: Vec<Rule>,
    pub rule_cursor: usize,
    pub paused: bool,
    pub speed: u32,
    pub edit_x: usize,
    pub edit_y: usize,
    pub seed: u32,
    pub scroll: usize,
}

impl DeathApp {
    pub fn new(seed: u32) -> Self {
        Self {
            state: AppState::RuleSelect,
            needs_redraw: true,
            grid: Grid::new(),
            rules: automaton::rule_presets(),
            rule_cursor: 0,
            paused: false,
            speed: 200,
            edit_x: GRID_W / 2,
            edit_y: GRID_H / 2,
            seed,
            scroll: 0,
        }
    }

    pub fn current_rule(&self) -> &Rule {
        &self.rules[self.rule_cursor]
    }

    pub fn tick(&mut self) {
        if self.state == AppState::Running && !self.paused {
            let rule = self.rules[self.rule_cursor].clone();
            self.grid.step(&rule);
            self.needs_redraw = true;
        }
    }

    pub fn handle_key(&mut self, key: char) -> bool {
        self.needs_redraw = true;
        match self.state {
            AppState::RuleSelect => self.key_select(key),
            AppState::Running => self.key_running(key),
            AppState::Editor => self.key_editor(key),
        }
    }

    fn key_select(&mut self, key: char) -> bool {
        match key {
            KEY_MENU => return false,
            KEY_UP => {
                if self.rule_cursor > 0 {
                    self.rule_cursor -= 1;
                    if self.rule_cursor < self.scroll { self.scroll = self.rule_cursor; }
                }
            }
            KEY_DOWN => {
                if self.rule_cursor < self.rules.len() - 1 {
                    self.rule_cursor += 1;
                    if self.rule_cursor >= self.scroll + 10 { self.scroll = self.rule_cursor - 9; }
                }
            }
            KEY_ENTER => {
                self.grid.clear();
                self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
                automaton::random_fill(&mut self.grid, self.seed, 64);
                self.paused = false;
                self.state = AppState::Running;
            }
            'e' | 'E' => {
                self.grid.clear();
                self.edit_x = GRID_W / 2;
                self.edit_y = GRID_H / 2;
                self.state = AppState::Editor;
            }
            _ => {}
        }
        true
    }

    fn key_running(&mut self, key: char) -> bool {
        match key {
            KEY_MENU => { self.state = AppState::RuleSelect; }
            ' ' => { self.paused = !self.paused; }
            KEY_ENTER => { if self.paused { let r = self.rules[self.rule_cursor].clone(); self.grid.step(&r); } }
            KEY_RIGHT => { if self.speed > 50 { self.speed -= 50; } }
            KEY_LEFT => { if self.speed < 1000 { self.speed += 50; } }
            'r' | 'R' => {
                self.grid.clear();
                self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
                automaton::random_fill(&mut self.grid, self.seed, 64);
            }
            'e' | 'E' => {
                self.paused = true;
                self.edit_x = GRID_W / 2;
                self.edit_y = GRID_H / 2;
                self.state = AppState::Editor;
            }
            _ => {}
        }
        true
    }

    fn key_editor(&mut self, key: char) -> bool {
        match key {
            KEY_MENU => { self.state = AppState::Running; self.paused = false; }
            KEY_UP => { if self.edit_y > 0 { self.edit_y -= 1; } }
            KEY_DOWN => { if self.edit_y < GRID_H - 1 { self.edit_y += 1; } }
            KEY_LEFT => { if self.edit_x > 0 { self.edit_x -= 1; } }
            KEY_RIGHT => { if self.edit_x < GRID_W - 1 { self.edit_x += 1; } }
            KEY_ENTER | ' ' => { self.grid.toggle(self.edit_x, self.edit_y); self.grid.count_population(); }
            KEY_BACKSPACE => { self.grid.count_population(); self.paused = false; self.state = AppState::Running; }
            'c' | 'C' => { self.grid.clear(); }
            _ => {}
        }
        true
    }
}
