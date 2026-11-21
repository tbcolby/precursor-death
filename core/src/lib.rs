//! Game of Death: a Life-like cellular automaton engine for the Precursor
//! app. Rules are `B/S` bitmasks with a tested parser, each preset carries
//! its own starting pattern, and the grid tracks which cells changed so the
//! shell draws only those. No Xous dependencies; runs under `cargo test`.
//!
//! The grid is a torus (opposite edges are neighbours); this is the only edge
//! rule in this app.

use precursor_appkit_core::entropy::{below, Entropy};

mod rule;
pub use rule::{ParseError, Rule};

/// Default grid: 4x4-pixel cells, 336 x 416 px.
pub const GRID_W: usize = 84;
pub const GRID_H: usize = 104;

/// How a preset starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Start {
    /// Random fill of the whole grid at this percent density.
    Soup(u32),
    /// Random fill at `density` percent inside a centred `size` x `size` square
    /// (rules that explode from anything bigger).
    Blob { size: usize, density: u32 },
}

/// A named rule with the start it behaves well from (audit: every rule used
/// to start from the same 25 % soup, which kills Anneal and Diamoeba and
/// floods Seeds and Replicator).
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    pub name: &'static str,
    pub notation: &'static str,
    pub start: Start,
}

impl Preset {
    /// The parsed rule. Presets are checked by `tests::presets_parse_and_round_trip`.
    pub fn rule(&self) -> Rule { Rule::parse(self.notation).expect("preset notation is valid") }
}

pub const PRESETS: &[Preset] = &[
    Preset { name: "Life", notation: "B3/S23", start: Start::Soup(25) },
    Preset { name: "HighLife", notation: "B36/S23", start: Start::Soup(25) },
    Preset { name: "Day & Night", notation: "B3678/S34678", start: Start::Soup(50) },
    Preset { name: "Seeds", notation: "B2/S", start: Start::Blob { size: 8, density: 50 } },
    Preset { name: "Diamoeba", notation: "B35678/S5678", start: Start::Soup(50) },
    Preset { name: "Morley", notation: "B368/S245", start: Start::Soup(25) },
    Preset { name: "Anneal", notation: "B4678/S35678", start: Start::Soup(50) },
    Preset { name: "2x2", notation: "B36/S125", start: Start::Soup(25) },
    Preset { name: "Maze", notation: "B3/S12345", start: Start::Blob { size: 16, density: 30 } },
    Preset { name: "Coagulations", notation: "B378/S235678", start: Start::Soup(30) },
    Preset { name: "Replicator", notation: "B1357/S1357", start: Start::Blob { size: 4, density: 75 } },
    Preset { name: "Stains", notation: "B3678/S235678", start: Start::Blob { size: 12, density: 40 } },
];

/// The grid with two persistent buffers.
pub struct Automaton {
    w: usize,
    h: usize,
    cells: Vec<u8>,
    next: Vec<u8>,
    rule: Rule,
    generation: u32,
    population: u32,
    changed: Vec<u32>,
}

impl Automaton {
    pub fn new(rule: Rule) -> Self { Self::with_size(GRID_W, GRID_H, rule) }

    pub fn with_size(w: usize, h: usize, rule: Rule) -> Self {
        let (w, h) = (w.max(3), h.max(3));
        Self { w, h, cells: vec![0; w * h], next: vec![0; w * h], rule, generation: 0, population: 0, changed: Vec::new() }
    }

    pub fn width(&self) -> usize { self.w }
    pub fn height(&self) -> usize { self.h }
    pub fn generation(&self) -> u32 { self.generation }
    pub fn population(&self) -> u32 { self.population }
    pub fn rule(&self) -> Rule { self.rule }

    /// Change the rule without touching the cells.
    pub fn set_rule(&mut self, rule: Rule) { self.rule = rule; }

    pub fn get(&self, x: usize, y: usize) -> bool { x < self.w && y < self.h && self.cells[y * self.w + x] == 1 }

    pub fn set(&mut self, x: usize, y: usize, alive: bool) {
        if x < self.w && y < self.h {
            let i = y * self.w + x;
            match (self.cells[i] == 1, alive) {
                (false, true) => self.population += 1,
                (true, false) => self.population -= 1,
                _ => {}
            }
            self.cells[i] = alive as u8;
        }
    }

    pub fn toggle(&mut self, x: usize, y: usize) { self.set(x, y, !self.get(x, y)); }

    pub fn clear(&mut self) {
        self.cells.fill(0);
        self.generation = 0;
        self.population = 0;
        self.changed.clear();
    }

    /// Cells that changed in the last step: `(x, y, now_alive)`.
    pub fn changed(&self) -> impl Iterator<Item = (usize, usize, bool)> + '_ {
        self.changed.iter().map(|&i| {
            let i = i as usize;
            (i % self.w, i / self.w, self.cells[i] == 1)
        })
    }

    /// Live runs in row `y` as `(start_x, len)`.
    pub fn row_runs(&self, y: usize) -> Vec<(usize, usize)> {
        let row = &self.cells[y * self.w..(y + 1) * self.w];
        let (mut runs, mut x) = (Vec::new(), 0);
        while x < self.w {
            if row[x] == 1 {
                let start = x;
                while x < self.w && row[x] == 1 {
                    x += 1;
                }
                runs.push((start, x - start));
            } else {
                x += 1;
            }
        }
        runs
    }

    /// Live neighbours with wrap-around (no `%` or division).
    pub fn neighbours(&self, x: usize, y: usize) -> u8 {
        let xs = [if x == 0 { self.w - 1 } else { x - 1 }, x, if x + 1 == self.w { 0 } else { x + 1 }];
        let ys = [if y == 0 { self.h - 1 } else { y - 1 }, y, if y + 1 == self.h { 0 } else { y + 1 }];
        let mut n = 0;
        for (yi, &yy) in ys.iter().enumerate() {
            for (xi, &xx) in xs.iter().enumerate() {
                if (xi, yi) != (1, 1) {
                    n += self.cells[yy * self.w + xx];
                }
            }
        }
        n
    }

    pub fn step(&mut self) {
        self.changed.clear();
        let mut pop = 0;
        for y in 0..self.h {
            for x in 0..self.w {
                let i = y * self.w + x;
                let alive = self.cells[i] == 1;
                let lives = self.rule.next(alive, self.neighbours(x, y));
                self.next[i] = lives as u8;
                pop += lives as u32;
                if lives != alive {
                    self.changed.push(i as u32);
                }
            }
        }
        core::mem::swap(&mut self.cells, &mut self.next);
        self.population = pop;
        self.generation += 1;
    }

    /// Clear and fill according to `start`. A failing entropy source leaves
    /// the grid empty.
    pub fn seed<E: Entropy>(&mut self, start: Start, rng: &mut E) {
        self.clear();
        let (x0, y0, x1, y1, density) = match start {
            Start::Soup(d) => (0, 0, self.w, self.h, d),
            Start::Blob { size, density } => {
                let (sw, sh) = (size.min(self.w), size.min(self.h));
                let (x0, y0) = ((self.w - sw) / 2, (self.h - sh) / 2);
                (x0, y0, x0 + sw, y0 + sh, density)
            }
        };
        for y in y0..y1 {
            for x in x0..x1 {
                if below(rng, 100).map(|v| v < density).unwrap_or(false) {
                    self.set(x, y, true);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
