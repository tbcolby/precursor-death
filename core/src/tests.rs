use super::*;
use precursor_appkit_core::entropy::testing::Broken;
use precursor_appkit_core::entropy::SimRng;

fn life() -> Rule { Rule::parse("B3/S23").unwrap() }

fn cells(a: &Automaton) -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    for y in 0..a.height() {
        for x in 0..a.width() {
            if a.get(x, y) {
                v.push((x, y));
            }
        }
    }
    v
}

fn from(w: usize, h: usize, rule: Rule, live: &[(usize, usize)]) -> Automaton {
    let mut a = Automaton::with_size(w, h, rule);
    for &(x, y) in live {
        a.set(x, y, true);
    }
    a
}

// ---- rule parser ----

#[test]
fn parses_standard_notation() {
    let r = Rule::parse("B3/S23").unwrap();
    assert_eq!(r, Rule::new(&[3], &[2, 3]).unwrap());
    assert_eq!(Rule::parse("B36/S23").unwrap(), Rule::new(&[3, 6], &[2, 3]).unwrap());
    assert_eq!(Rule::parse("B3678/S34678").unwrap().notation(), "B3678/S34678");
    assert_eq!(Rule::parse("B2/S").unwrap(), Rule::new(&[2], &[]).unwrap());
    assert_eq!(Rule::parse("B/S").unwrap(), Rule::new(&[], &[]).unwrap());
    assert_eq!(Rule::parse("B012345678/S012345678").unwrap().notation(), "B012345678/S012345678");
}

#[test]
fn parser_accepts_case_and_whitespace_and_canonicalises_order() {
    assert_eq!(Rule::parse("  b3/s23\n").unwrap().notation(), "B3/S23");
    assert_eq!(Rule::parse("B63/S32").unwrap().notation(), "B36/S23");
}

#[test]
fn parser_rejects_malformed_rules() {
    use ParseError::*;
    let cases: &[(&str, ParseError)] = &[
        ("", Empty),
        ("   ", Empty),
        ("3/23", MissingBirth),
        ("S23/B3", MissingBirth),
        ("B3S23", MissingSlash),
        ("B3", MissingSlash),
        ("B3/23", MissingSurvival),
        ("B3/", MissingSurvival),
        ("B3/B23", MissingSurvival),
        ("B3/S9", DigitTooLarge('9')),
        ("B9/S23", DigitTooLarge('9')),
        ("B33/S23", Duplicate('3')),
        ("B3/S223", Duplicate('2')),
        ("B3/S2 3", BadChar(' ')),
        ("B3/Sx", BadChar('x')),
        ("B-3/S23", BadChar('-')),
        ("B3/S23/S4", BadChar('/')),
        ("BB3/S23", BadChar('B')),
        ("B3/S2\u{663}", BadChar('\u{663}')),
    ];
    for (text, err) in cases {
        assert_eq!(Rule::parse(text), Err(*err), "{text:?}");
    }
    assert_eq!(format!("{}", ParseError::DigitTooLarge('9')), "9 is more than 8 neighbours");
}

#[test]
fn old_bug_digits_above_eight_are_errors_not_dropped() {
    // The old rule() silently dropped digits >= 9.
    assert!(Rule::new(&[3, 9], &[2]).is_err());
    assert!(Rule::new(&[3], &[200]).is_err());
}

#[test]
fn notation_round_trips_for_every_mask() {
    for b in (0..512u16).step_by(7) {
        for s in (0..512u16).step_by(11) {
            let bl: Vec<u8> = (0..9).filter(|n| b >> n & 1 == 1).collect();
            let sl: Vec<u8> = (0..9).filter(|n| s >> n & 1 == 1).collect();
            let r = Rule::new(&bl, &sl).unwrap();
            assert_eq!(Rule::parse(&r.notation()), Ok(r));
        }
    }
}

#[test]
fn rule_queries_and_toggles() {
    let mut r = life();
    assert!(r.born(3) && !r.born(2) && r.survives(2) && r.survives(3) && !r.survives(4));
    assert!(!r.born(9) && !r.survives(9) && !r.born(255));
    assert!(r.next(false, 3) && !r.next(false, 2) && r.next(true, 2) && !r.next(true, 1));
    r.toggle_birth(6);
    r.toggle_survive(3);
    assert_eq!(r.notation(), "B36/S2");
    r.toggle_birth(9); // out of range: ignored
    assert_eq!(r.notation(), "B36/S2");
}

#[test]
fn presets_parse_and_round_trip() {
    let expected = [
        "B3/S23", "B36/S23", "B3678/S34678", "B2/S", "B35678/S5678", "B368/S245",
        "B4678/S35678", "B36/S125", "B3/S12345", "B378/S235678", "B1357/S1357", "B3678/S235678",
    ];
    assert_eq!(PRESETS.len(), expected.len());
    for (p, e) in PRESETS.iter().zip(expected) {
        assert_eq!(p.notation, e, "{}", p.name);
        assert_eq!(p.rule().notation(), e, "{}", p.name);
    }
}

// ---- engine ----

#[test]
fn life_rule_behaves_like_life_on_known_patterns() {
    // Blinker and block under the parsed B3/S23 rule.
    let mut a = from(7, 7, life(), &[(2, 3), (3, 3), (4, 3)]);
    a.step();
    assert_eq!(cells(&a), vec![(3, 2), (3, 3), (3, 4)]);
    a.step();
    assert_eq!(cells(&a), vec![(2, 3), (3, 3), (4, 3)]);
    let block = [(1, 1), (2, 1), (1, 2), (2, 2)];
    let mut a = from(6, 6, life(), &block);
    a.step();
    assert_eq!(cells(&a), block.to_vec());
}

#[test]
fn seeds_rule_everything_dies_each_generation() {
    // B2/S: a pair of neighbours gives birth to the cells around them, the pair itself dies.
    let mut a = from(9, 9, Rule::parse("B2/S").unwrap(), &[(4, 4), (5, 4)]);
    a.step();
    assert!(!a.get(4, 4) && !a.get(5, 4));
    // The four dead cells that touch both live cells.
    assert_eq!(a.population(), 4);
}

#[test]
fn torus_wrap_in_both_axes() {
    let a = from(5, 4, life(), &[(4, 3)]);
    assert_eq!(a.neighbours(0, 0), 1);
    assert_eq!(a.neighbours(3, 2), 1);
    assert_eq!(a.neighbours(2, 1), 0);
    // A glider on a small torus comes home: 4 * lcm(8, 6) generations.
    let mut a = from(8, 6, life(), &[(4, 3), (5, 4), (3, 5), (4, 5), (5, 5)]);
    let start = cells(&a);
    for _ in 0..96 {
        a.step();
        assert_eq!(a.population(), 5);
    }
    assert_eq!(cells(&a), start);
}

#[test]
fn highlife_differs_from_life_only_at_six_neighbours() {
    // Dead centre with 6 live neighbours: born under B36, not under B3.
    let ring = [(1, 1), (2, 1), (3, 1), (1, 2), (3, 2), (1, 3)];
    let mut hl = from(5, 5, Rule::parse("B36/S23").unwrap(), &ring);
    let mut cl = from(5, 5, life(), &ring);
    assert_eq!(hl.neighbours(2, 2), 6);
    hl.step();
    cl.step();
    assert!(hl.get(2, 2));
    assert!(!cl.get(2, 2));
}

#[test]
fn changed_cells_are_exactly_the_difference_and_population_tracks() {
    let mut a = Automaton::with_size(30, 20, Rule::parse("B36/S23").unwrap());
    a.seed(Start::Soup(30), &mut SimRng::new(11));
    for _ in 0..8 {
        assert_eq!(a.population() as usize, cells(&a).len());
        let before = cells(&a);
        a.step();
        let after = cells(&a);
        let mut expected: Vec<_> = before.iter().filter(|c| !after.contains(c)).copied().collect();
        expected.extend(after.iter().filter(|c| !before.contains(c)));
        expected.sort();
        let mut got: Vec<_> = a.changed().map(|(x, y, alive)| {
            assert_eq!(alive, a.get(x, y));
            (x, y)
        }).collect();
        got.sort();
        assert_eq!(got, expected);
    }
    a.toggle(2, 2);
    a.toggle(2, 2);
    a.set(500, 500, true);
    assert_eq!(a.population() as usize, cells(&a).len());
    a.clear();
    assert_eq!((a.population(), a.generation()), (0, 0));
}

// ---- seeding ----

#[test]
fn soup_density_is_close_to_requested() {
    for d in [25, 50] {
        let mut a = Automaton::new(life());
        a.seed(Start::Soup(d), &mut SimRng::new(42));
        let pct = a.population() as usize * 100 / (GRID_W * GRID_H);
        assert!((d as usize - 5..=d as usize + 5).contains(&pct), "{d}% gave {pct}%");
    }
}

#[test]
fn blob_start_stays_inside_its_square() {
    let mut a = Automaton::new(Rule::parse("B2/S").unwrap());
    a.seed(Start::Blob { size: 8, density: 100 }, &mut SimRng::new(1));
    assert_eq!(a.population(), 64);
    let (x0, y0) = ((GRID_W - 8) / 2, (GRID_H - 8) / 2);
    for (x, y) in cells(&a) {
        assert!((x0..x0 + 8).contains(&x) && (y0..y0 + 8).contains(&y));
    }
    // A blob bigger than the grid is clamped, not a panic.
    let mut small = Automaton::with_size(5, 5, life());
    small.seed(Start::Blob { size: 99, density: 100 }, &mut SimRng::new(1));
    assert_eq!(small.population(), 25);
}

#[test]
fn every_preset_start_is_viable() {
    // Audit: rules designed around other densities collapsed or exploded from a 25% soup.
    // Each preset's own start must leave life on the board after 30 generations and
    // not fill the grid.
    for p in PRESETS {
        let mut a = Automaton::new(p.rule());
        a.seed(p.start, &mut SimRng::new(2024));
        assert!(a.population() > 0, "{}", p.name);
        for _ in 0..30 {
            a.step();
        }
        let pct = a.population() as usize * 100 / (GRID_W * GRID_H);
        assert!(a.population() > 0, "{} died out", p.name);
        assert!(pct < 90, "{} filled the grid ({pct}%)", p.name);
    }
}

#[test]
fn failing_entropy_leaves_an_empty_grid_and_seeding_is_reproducible() {
    let mut a = Automaton::new(life());
    a.seed(Start::Soup(25), &mut Broken);
    assert_eq!(a.population(), 0);
    let (mut x, mut y) = (Automaton::new(life()), Automaton::new(life()));
    x.seed(Start::Soup(25), &mut SimRng::new(3));
    y.seed(Start::Soup(25), &mut SimRng::new(3));
    assert_eq!(cells(&x), cells(&y));
}

#[test]
fn row_runs_and_set_rule() {
    let mut a = from(10, 3, life(), &[(1, 1), (2, 1), (3, 1), (7, 1)]);
    assert_eq!(a.row_runs(1), vec![(1, 3), (7, 1)]);
    assert!(a.row_runs(0).is_empty());
    a.set_rule(Rule::parse("B2/S").unwrap());
    assert_eq!(a.rule().notation(), "B2/S");
}
