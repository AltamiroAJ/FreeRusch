# AGENTS.md

Guidance for AI agents and humans working in this repository.

## What this is

FreeRusch computes structural design bending moments for **bridge slabs by the
Rüsch method**, in pure Rust. It is a member of the **Craft family** of
clean-room reimplementations, and consumes the shared family libraries from
[`storytold/craft-libs`](https://github.com/storytold/craft-libs) pinned by `rev`
in the root `Cargo.toml`.

- Repository: <https://github.com/AltamiroAJ/FreeRusch> (private for now)
- Engineering reference only: T-Rüsch (TQS). Never copy its code, data or UI.

## Layout

```
crates/freerusch-core      pure calculation engine (no UI, no file I/O)
crates/freerusch-report    calculation memo in self-contained HTML
crates/freerusch-project   .frus project file, versioned JSON, atomic writes
apps/freerusch             eframe/egui desktop GUI
tools/                     extract_rusch_tables.py — regenerates embedded tables
packaging/                 release packaging scripts (env.sh holds shared setup)
```

## Quality gate

Run all four before claiming work is done. CI runs the same set on
Linux/Windows/macOS:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Tests come first: a behaviour change without a failing-then-passing test is not
finished. Test names state the behaviour; assertions cite the source of expected
values (spreadsheet cell, normative clause, hand calculation).

## Conventions

- **Pure Rust.** No C/C++ or external binaries added deliberately. Known
  transitive exception: `aws-lc-sys` arrives through `craft-print` → `winprint`
  → `sigstore`; it needs cmake and a C toolchain to build.
- **Never invent normative values.** Every coefficient or factor carries a
  comment naming its source. If the source is missing, add the item to the
  blocked list in `docs/ROADMAP.md` instead of guessing.
- **No hardcoded UI colors.** Interface code reads `theme::pal(ui)` and uses only
  its tokens; the palette derives from the active egui theme and its contrast is
  covered by tests. Prefer the configured text styles over literal font sizes.
- Errors use `thiserror`, typed per crate (`RuschError`, `ReportError`,
  `ProjectError`).
- `///` docs on every public item. Public API changes update the README example.
- Tabular data is generated, never hand-edited: change
  `tools/extract_rusch_tables.py`, rerun it, and commit the regenerated JSON.
- Commits are atomic and follow `type: imperative summary` with a body that
  explains *why*.

## Embedded tables

`crates/freerusch-core/data/rusch_tables.json` is the single artifact for the
186 Rüsch tables. `freerusch-core` embeds it with `include_str!` and
deserializes it once behind `rusch_tables()`. Do not generate Rust source for the
data: ~57k coefficients as expressions made compilation several times slower for
no benefit.

The source workbook is not versioned; its SHA-256 is stamped into the JSON
header. Regenerate with:

```sh
python tools/extract_rusch_tables.py --xlsx <converted workbook.xlsx> --all \
  --source-sha256 <sha256 of the original .xls>
```

The extractor records fidelity gaps instead of guessing: a `-` cell becomes
`null` (absent, never `0.0`), and a spreadsheet error (`=#REF!`) or an
unparseable number (`2,.82`) becomes `null` **and** an entry in the JSON
`warnings` array. Check that array when the tables change.

The engine calculates from the embedded tables through `design_moments()`:
permanent load via `uniform_load_moments()` (`M = k·q·lx²`) and live load via
`live_load_moments()` (`M = φ·(P·M_L + q·M_p + q·M_p')`), combined as
`M_d = 1.35·M_g + 1.5·M_q`. Both paths are validated against the workbook's
worked example and the IME course notes — see `docs/ROADMAP.md` (R1/R2).

The historical simplified block was removed in D4: there is exactly **one**
calculation path. Do not reintroduce a second coefficient model "for
compatibility" — a table-based result and a simplified result that disagree is
worse than having no fallback at all.

## Release

Releases follow the Craft family standard: a thin caller workflow
(`.github/workflows/release.yml`) delegates to
`storytold/craft-common-actions`, triggered by pushing the `release` branch.
The version comes from `[workspace.package] version` in the root `Cargo.toml`,
which `packaging/env.sh` reads. Signing credentials are not configured yet; do
not push `release` until they exist in the `release` environment.
