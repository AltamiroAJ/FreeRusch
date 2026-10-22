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
crates/freerusch-report    calculation memo in HTML (+ inline SVG diagram)
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
- Errors use `thiserror`, typed per crate (`RuschError`, `ReportError`,
  `ProjectError`).
- `///` docs on every public item. Public API changes update the README example.
- Tabular data is generated, never hand-edited: change
  `tools/extract_rusch_tables.py`, rerun it, and commit both the JSON and the
  generated Rust.
- Commits are atomic and follow `type: imperative summary` with a body that
  explains *why*.

## Embedded tables

`crates/freerusch-core/data/rusch_tables.json` is the audit artifact;
`crates/freerusch-core/src/rusch_data.rs` is generated from it and must not be
edited by hand. The source workbook is not versioned; its SHA-256 is stamped into
the generated header. Regenerate with:

```sh
python tools/extract_rusch_tables.py --xlsx <converted workbook.xlsx> \
  --source-sha256 <sha256 of the original .xls>
```

The extractor aborts on a missing cell and maps `-` cells to `None` (absent),
never to an invented `0.0`.

## Release

Releases follow the Craft family standard: a thin caller workflow
(`.github/workflows/release.yml`) delegates to
`storytold/craft-common-actions`, triggered by pushing the `release` branch.
The version comes from `[workspace.package] version` in the root `Cargo.toml`,
which `packaging/env.sh` reads. Signing credentials are not configured yet; do
not push `release` until they exist in the `release` environment.
