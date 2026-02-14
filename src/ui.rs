//! UI rendering for Game of Death.

extern crate alloc;
use alloc::format;
use alloc::string::String;

use gam::{Gam, Point, Line, TextView, TextBounds, GlyphStyle};
use crate::app::{DeathApp, AppState};
use crate::automaton::CELL_SIZE;

const SCREEN_W: i16 = 336;
const HEADER_H: i16 = 30;
const FOOTER_Y: i16 = 490;

fn draw_header(gam: &Gam, title: &str) {
    gam.draw_rectangle(Point::new(0, 0), Point::new(SCREEN_W, HEADER_H)).ok();
    let mut tv = TextView::new(TextBounds::BoundingBox(Point::new(8, 2), Point::new(SCREEN_W - 8, HEADER_H)));
    tv.style = GlyphStyle::Bold; tv.invert = true;
    tv.text = String::from(title);
    gam.post_textview(&mut tv).ok();
}

fn draw_footer(gam: &Gam, text: &str) {
    gam.draw_line(Line::new(Point::new(0, FOOTER_Y), Point::new(SCREEN_W, FOOTER_Y))).ok();
    let mut tv = TextView::new(TextBounds::BoundingBox(Point::new(8, FOOTER_Y + 4), Point::new(SCREEN_W - 8, FOOTER_Y + 40)));
    tv.style = GlyphStyle::Small;
    tv.text = String::from(text);
    gam.post_textview(&mut tv).ok();
}

pub fn draw(gam: &Gam, app: &DeathApp) {
    gam.clear().ok();
    match app.state {
        AppState::RuleSelect => draw_select(gam, app),
        AppState::Running => draw_running(gam, app),
        AppState::Editor => draw_editor(gam, app),
    }
    gam.redraw().ok();
}

fn draw_select(gam: &Gam, app: &DeathApp) {
    draw_header(gam, "Game of Death");

    let start_y: i16 = 38;
    let line_h: i16 = 44;
    let visible = 10;
    let end = (app.scroll + visible).min(app.rules.len());

    for vi in app.scroll..end {
        let row = vi - app.scroll;
        let y = start_y + (row as i16) * line_h;
        let selected = vi == app.rule_cursor;
        let rule = &app.rules[vi];

        if selected {
            gam.draw_rectangle(Point::new(4, y - 2), Point::new(SCREEN_W - 4, y + line_h - 6)).ok();
        }

        let mut tv = TextView::new(TextBounds::BoundingBox(Point::new(12, y), Point::new(SCREEN_W - 12, y + 22)));
        tv.style = GlyphStyle::Regular;
        if selected { tv.invert = true; }
        tv.text = String::from(rule.name);
        gam.post_textview(&mut tv).ok();

        let mut tv2 = TextView::new(TextBounds::BoundingBox(Point::new(16, y + 22), Point::new(SCREEN_W - 12, y + line_h)));
        tv2.style = GlyphStyle::Small;
        if selected { tv2.invert = true; }
        tv2.text = rule.notation();
        gam.post_textview(&mut tv2).ok();
    }

    draw_footer(gam, "Up/Down: browse  Enter: random start  E: editor  Menu: quit");
}

fn draw_running(gam: &Gam, app: &DeathApp) {
    let rule = app.current_rule();
    let status = if app.paused {
        format!("PAUSED {} G:{} P:{}", rule.notation(), app.grid.generation, app.grid.population)
    } else {
        format!("{} G:{} P:{}", rule.notation(), app.grid.generation, app.grid.population)
    };
    draw_header(gam, &status);

    let offset_y: i16 = HEADER_H;
    draw_grid_cells(gam, app, 0, offset_y);

    draw_footer(gam, "Space: pause  Enter: step  L/R: speed  R: reset  Menu: back");
}

fn draw_editor(gam: &Gam, app: &DeathApp) {
    let status = format!("Edit ({},{}) {} P:{}", app.edit_x, app.edit_y, app.current_rule().notation(), app.grid.population);
    draw_header(gam, &status);

    let offset_y: i16 = HEADER_H;
    draw_grid_cells(gam, app, 0, offset_y);

    // Crosshair cursor
    let cx = (app.edit_x as i16) * CELL_SIZE + CELL_SIZE / 2;
    let cy = offset_y + (app.edit_y as i16) * CELL_SIZE + CELL_SIZE / 2;
    gam.draw_line(Line::new(Point::new(cx - 8, cy), Point::new(cx + 8, cy))).ok();
    gam.draw_line(Line::new(Point::new(cx, cy - 8), Point::new(cx, cy + 8))).ok();

    draw_footer(gam, "Arrows: move  Enter: toggle  BS: run  C: clear");
}

fn draw_grid_cells(gam: &Gam, app: &DeathApp, offset_x: i16, offset_y: i16) {
    for y in 0..app.grid.height {
        let mut x = 0;
        while x < app.grid.width {
            if app.grid.get(x, y) {
                let start_x = x;
                while x < app.grid.width && app.grid.get(x, y) { x += 1; }
                let px1 = offset_x + (start_x as i16) * CELL_SIZE;
                let py1 = offset_y + (y as i16) * CELL_SIZE;
                let px2 = offset_x + (x as i16) * CELL_SIZE;
                let py2 = py1 + CELL_SIZE;
                gam.draw_rectangle(Point::new(px1, py1), Point::new(px2, py2)).ok();
            } else {
                x += 1;
            }
        }
    }
}
