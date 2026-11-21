# precursor-death

Game of Death: a Life-like cellular automaton explorer for [Precursor](https://precursor.dev). Twelve rule sets, a custom B/S rule editor, a cell editor and adjustable speed, on an 84x104 torus of 4-pixel cells.

## Rules

A rule is written `B<counts>/S<counts>`: a dead cell with that many live neighbours is born (B), a live cell with that many survives (S). Counts are 0 to 8.

| Name | Rule | Starts from |
|---|---|---|
| Life | B3/S23 | 25 % soup |
| HighLife | B36/S23 | 25 % soup |
| Day & Night | B3678/S34678 | 50 % soup |
| Seeds | B2/S | 8x8 blob, 50 % |
| Diamoeba | B35678/S5678 | 50 % soup |
| Morley | B368/S245 | 25 % soup |
| Anneal | B4678/S35678 | 50 % soup |
| 2x2 | B36/S125 | 25 % soup |
| Maze | B3/S12345 | 16x16 blob, 30 % |
| Coagulations | B378/S235678 | 30 % soup |
| Replicator | B1357/S1357 | 4x4 blob, 75 % |
| Stains | B3678/S235678 | 12x12 blob, 40 % |

Each rule starts from the density or blob it behaves well from (the old app used a 25 % soup for all of them, which collapses Anneal and Diamoeba and floods Seeds and Replicator). `tests::every_preset_start_is_viable` checks that every preset is still alive after 30 generations and has not filled the grid.

The rule parser (`Rule::parse`, in `core/src/rule.rs`) accepts `B3/S23`, `b36/s23`, `B2/S` and surrounding whitespace, and rejects: empty text, a missing `B`, `/` or `S`, any character other than a digit 0-8, the digit 9, a repeated digit, and trailing text. Presets are parsed from their notation at startup, so a typo in the table fails a test instead of silently vanishing (the old code dropped digits of 9 or more). The app has no text-entry screen for rules; you build a custom rule with the toggle editor, and the parser is used for the presets and covered by tests.

## Features

- Custom rule editor: nine birth toggles and nine survival toggles, F1 runs it from a 30 % soup.
- Cell editor with a frame cursor; leaving it keeps your pause state.
- Only cells that changed are redrawn each generation; the grid is a torus and this app has no other edge rule (`precursor-life` has a dead-edge option).
- Two persistent buffers; no allocation per generation.
- The pump runs only while the app is in front and a simulation is running; speed 50 to 1000 ms per generation.
- Random seeds come from the TRNG (cosmetic only).

## Keys

| Screen | Keys |
|---|---|
| Rule list | Up/Down pick, Enter start, E empty cell editor, Backspace or F4 quit |
| Running | Space pause/run, Enter step (paused), Right faster, Left slower, R new random start for this rule, E cell editor, Backspace or F4 back to the list |
| Custom rule editor | Arrows move, Enter or Space toggle, F1 run, Backspace or F4 back |
| Cell editor | Arrows move, Enter or Space toggle, C clear, F1 run, Backspace or F4 back (pause state kept) |

The menu key (centre of the D-pad) is consumed by Xous and never reaches the app.

## Storage

None. Custom rules are not saved.

## Limitations

- Layout has not been checked on a device. Renode capture not yet run: `capture.py` is written but not executed; verification so far is the hosted build, clippy, lint and host tests.
- Torus only.
- The status line (name, rule, generation, population, speed) is one line of small text and may be cut off.
- `precursor-life` and this app each carry their own grid engine in their own `core/`; a shared Life-like engine would be a candidate for `precursor-appkit-core`.

## Building

```bash
bin/integrate --xous-root ~/xous-core --grade ~/Dev/precursor/precursor-death
```

Host tests alone: `cd core && cargo test`.

## Layout

```
core/src/rule.rs    B/S rule masks and the parser
core/src/lib.rs     torus engine, presets with per-rule starts, seeding, changed-cell list
core/src/tests.rs   parser accept/reject table, round trips, Life/Seeds/HighLife behaviour, seeding, viability
src/main.rs, app.rs GAM registration, pump speed and focus control, keys, drawing, rule editor
capture.py          Renode capture sequence (not yet run)
```

## Author

Tyler Colby, [Colby's Data Movers, LLC](https://colbysdatamovers.com). Licensed under the Apache License, Version 2.0 (see [LICENSE](LICENSE)).
