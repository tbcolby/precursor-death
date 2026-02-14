# Game of Death

A multi-rule cellular automaton explorer for the [Precursor](https://precursor.dev) handheld, featuring 12 Life-like rule sets with cell editor and adjustable speed on the 336x536 monochrome 1-bit display.

## Features

- **12 rule sets** — Life, HighLife, Day & Night, Seeds, Diamoeba, Morley, Anneal, 2x2, Maze, Coagulations, Replicator, Stains
- **B/S notation** — Birth/Survival rules displayed for each automaton
- **84x114 grid** — 4px cells with toroidal wrapping
- **Cell editor** — Draw initial patterns with crosshair cursor
- **Random seeding** — ~25% density random fill
- **Adjustable speed** — 50ms to 1000ms per generation
- **Pause & step** — Single-step through generations

## Architecture

- `automaton.rs` — Grid, arbitrary B/S rule step, 12 rule presets, LCG random fill
- `app.rs` — 3-state machine (RuleSelect → Running ↔ Editor)
- `ui.rs` — Rule browser with notation, run-length grid rendering, crosshair editor
- `main.rs` — Xous event loop with ticktimer auto-step

## Controls

| Key | Action |
|-----|--------|
| Up/Down | Browse rules / Move cursor |
| Enter | Start (random) / Toggle cell / Step (paused) |
| Space | Pause/Resume / Toggle cell |
| Left/Right | Adjust speed / Move cursor |
| E | Open editor |
| R | Reset with new random seed |
| C | Clear grid (editor) |
| Backspace | Run from editor |
| Menu (∴) | Back/Quit |

---

## Development

This app was developed using the methodology described in [xous-dev-toolkit](https://github.com/tbcolby/xous-dev-toolkit) — an LLM-assisted approach to Precursor app development on macOS ARM64.

---

## Author

Made by Tyler Colby — [Colby's Data Movers, LLC](https://colbysdatamovers.com)

Contact: [tyler@colbysdatamovers.com](mailto:tyler@colbysdatamovers.com) | [GitHub Issues](https://github.com/tbcolby/precursor-death/issues)

---

## License

Licensed under the Apache License, Version 2.0.

See [LICENSE](LICENSE) for the full text.
