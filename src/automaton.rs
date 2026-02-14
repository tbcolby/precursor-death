//! Multi-rule cellular automaton engine.
//! Supports arbitrary Birth/Survival rules beyond standard B3/S23.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

pub const GRID_W: usize = 84;
pub const GRID_H: usize = 114;
pub const CELL_SIZE: i16 = 4;

/// A rule set: which neighbor counts cause birth/survival.
#[derive(Clone)]
pub struct Rule {
    pub name: &'static str,
    pub birth: [bool; 9],    // birth[n] = true if n neighbors causes birth
    pub survival: [bool; 9], // survival[n] = true if n neighbors keeps cell alive
}

impl Rule {
    pub fn notation(&self) -> String {
        let mut s = String::from("B");
        for i in 0..9 { if self.birth[i] { s.push_str(&format!("{}", i)); } }
        s.push_str("/S");
        for i in 0..9 { if self.survival[i] { s.push_str(&format!("{}", i)); } }
        s
    }
}

fn rule(name: &'static str, birth: &[u8], survive: &[u8]) -> Rule {
    let mut b = [false; 9];
    let mut s = [false; 9];
    for &v in birth { if (v as usize) < 9 { b[v as usize] = true; } }
    for &v in survive { if (v as usize) < 9 { s[v as usize] = true; } }
    Rule { name, birth: b, survival: s }
}

/// 12 classic Life-like cellular automaton rules.
pub fn rule_presets() -> Vec<Rule> {
    let mut r = Vec::new();
    r.push(rule("Life (B3/S23)", &[3], &[2,3]));
    r.push(rule("HighLife (B36/S23)", &[3,6], &[2,3]));
    r.push(rule("Day & Night (B3678/S34678)", &[3,6,7,8], &[3,4,6,7,8]));
    r.push(rule("Seeds (B2/S)", &[2], &[]));
    r.push(rule("Diamoeba (B35678/S5678)", &[3,5,6,7,8], &[5,6,7,8]));
    r.push(rule("Morley (B368/S245)", &[3,6,8], &[2,4,5]));
    r.push(rule("Anneal (B4678/S35678)", &[4,6,7,8], &[3,5,6,7,8]));
    r.push(rule("2x2 (B36/S125)", &[3,6], &[1,2,5]));
    r.push(rule("Maze (B3/S12345)", &[3], &[1,2,3,4,5]));
    r.push(rule("Coagulations (B378/S235678)", &[3,7,8], &[2,3,5,6,7,8]));
    r.push(rule("Replicator (B1357/S1357)", &[1,3,5,7], &[1,3,5,7]));
    r.push(rule("Stains (B3678/S235678)", &[3,6,7,8], &[2,3,5,6,7,8]));
    r
}

pub struct Grid {
    pub cells: Vec<bool>,
    pub width: usize,
    pub height: usize,
    pub generation: u32,
    pub population: u32,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            cells: alloc::vec![false; GRID_W * GRID_H],
            width: GRID_W,
            height: GRID_H,
            generation: 0,
            population: 0,
        }
    }

    pub fn clear(&mut self) {
        for c in self.cells.iter_mut() { *c = false; }
        self.generation = 0;
        self.population = 0;
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.cells[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, alive: bool) {
        self.cells[y * self.width + x] = alive;
    }

    pub fn toggle(&mut self, x: usize, y: usize) {
        let i = y * self.width + x;
        self.cells[i] = !self.cells[i];
    }

    fn neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0u8;
        for dy in [self.height - 1, 0, 1].iter() {
            for dx in [self.width - 1, 0, 1].iter() {
                if *dx == 0 && *dy == 0 { continue; }
                let nx = (x + *dx) % self.width;
                let ny = (y + *dy) % self.height;
                if self.cells[ny * self.width + nx] { count += 1; }
            }
        }
        count
    }

    /// Step with arbitrary Birth/Survival rule.
    pub fn step(&mut self, rule: &Rule) {
        let mut next = alloc::vec![false; self.width * self.height];
        let mut pop = 0u32;
        for y in 0..self.height {
            for x in 0..self.width {
                let n = self.neighbors(x, y) as usize;
                let alive = self.get(x, y);
                let lives = if alive { rule.survival[n] } else { rule.birth[n] };
                next[y * self.width + x] = lives;
                if lives { pop += 1; }
            }
        }
        self.cells = next;
        self.generation += 1;
        self.population = pop;
    }

    pub fn count_population(&mut self) {
        self.population = self.cells.iter().filter(|&&c| c).count() as u32;
    }
}

/// Random fill using LCG.
pub fn random_fill(grid: &mut Grid, seed: u32, density: u32) {
    let mut rng = seed;
    for y in 0..grid.height {
        for x in 0..grid.width {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let val = (rng >> 16) & 0xFF;
            grid.set(x, y, val < density);
        }
    }
    grid.count_population();
}
