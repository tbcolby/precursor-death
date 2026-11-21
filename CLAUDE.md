# Death build notes

- `core/src/rule.rs`: `Rule` is two `u16` masks (bit n = n neighbours). `Rule::parse` is strict B/S notation (see its error enum); `Rule::new` rejects counts above 8. Presets are strings parsed on use.
- `core/src/lib.rs`: torus `Automaton`, `step` records `changed` indices, `Start::{Soup, Blob}` per preset, seeding via `Entropy` (a failing source leaves the grid empty).
- Shell: pump runs only when foreground and `App::animating()`; `interval_ms()` follows the speed setting. Custom rule editor has 18 toggles (0..9 birth, 9..18 survival).
- Backspace/F4 leave every screen; the cell editor returns to Running with the prior pause state, F1 runs.
- Tests: `cd core && cargo test`.
