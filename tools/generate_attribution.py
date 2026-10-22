#!/usr/bin/env python3
"""Gera ATTRIBUTION.md a partir do grafo real de dependências.

A lista de dependências é derivada de `cargo metadata`, não escrita à mão, para
que a atribuição acompanhe o `Cargo.lock` em vez de envelhecer em silêncio.

Uso:
  cargo metadata --format-version 1 --locked > target/ref/metadata.json
  python tools/generate_attribution.py
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
OUT = REPO / "ATTRIBUTION.md"
TABLES_JSON = REPO / "crates/freerusch-core/data/rusch_tables.json"

HEADER = """# Attribution

FreeRusch is licensed under the MIT License **or** the Apache License, Version
2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.

This file records third-party material bundled with or required by FreeRusch.
The dependency list below is generated from `cargo metadata`; regenerate it with
`tools/generate_attribution.py` whenever `Cargo.lock` changes.

## Embedded Rüsch coefficient tables
"""

TABLES_TMPL = """The tabulated coefficients under `crates/freerusch-core/data/` are
engineering data transcribed from the DIN/IBSW Rüsch tables as compiled in the
academic workbook **{source}** (SHA-256 `{sha}`), through
`tools/extract_rusch_tables.py`.

- The workbook itself is **not** redistributed with this repository.
- The extraction is mechanical: values are copied verbatim, `-` cells become
  "absent", and no coefficient is interpolated or invented at extraction time.
- Coverage is partial: only a subset of the workbook's tables is embedded so
  far. See `docs/ROADMAP.md` (item R1).

## Normative references

NBR 7188:2013 (road traffic loads on bridges) and NBR 6118:2023 (design of
concrete structures) are cited as engineering references. Their text is not
distributed with this software and no excerpt is reproduced in the source tree.

## Runtime dependencies
"""

FOOTER = """
## Build-only dependencies

The Rust toolchain, `cmake` and a C toolchain are required to build: `aws-lc-sys`
arrives transitively through `craft-print` → `winprint` → `sigstore`. See
`AGENTS.md`.

## Notes

- `craft-*` crates come from `storytold/craft-libs`, pinned by git `rev` in the
  root `Cargo.toml`; they are consumed as Apache-2.0 licensed libraries.
- Workspace-local crates (`freerusch-core`, `freerusch-report`,
  `freerusch-project`, `freerusch`) are excluded: they are this project.
"""


def load_tables_source() -> tuple[str, str]:
    if not TABLES_JSON.exists():
        return "Tabelas Rusch Final - IME.xls", "(não registrado)"
    payload = json.loads(TABLES_JSON.read_text(encoding="utf-8"))
    source = payload.get("source", {})
    return (
        source.get("name", "Tabelas Rusch Final - IME.xls"),
        source.get("sha256") or "(não registrado)",
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", default=str(REPO / "target/ref/metadata.json"))
    parser.add_argument("--out", default=str(OUT))
    args = parser.parse_args()

    meta = json.loads(Path(args.metadata).read_text(encoding="utf-8"))
    workspace_names = {p["name"] for p in meta["packages"] if p["source"] is None}

    rows = []
    for pkg in meta["packages"]:
        if pkg["name"] in workspace_names:
            continue
        license_id = pkg.get("license") or "(ver LICENSE do pacote)"
        source = pkg.get("source") or ""
        origin = "git" if source.startswith("git+") else "registry"
        rows.append((pkg["name"], pkg["version"], license_id, origin))

    rows.sort(key=lambda r: (r[2].lower(), r[0].lower(), r[1]))

    source_name, source_sha = load_tables_source()
    lines = [HEADER, TABLES_TMPL.format(source=source_name, sha=source_sha)]
    lines.append(
        f"{len(rows)} packages outside this workspace, grouped by the licence "
        "declared in their package metadata.\n"
    )

    current = None
    for name, version, license_id, origin in rows:
        if license_id != current:
            current = license_id
            lines.append(f"\n### {license_id}\n")
            lines.append("| Package | Version | Source |")
            lines.append("| --- | --- | --- |")
        lines.append(f"| `{name}` | {version} | {origin} |")

    lines.append(FOOTER)
    Path(args.out).write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(f"packages listed: {len(rows)}")
    print(f"licences: {len({r[2] for r in rows})}")
    print(f"written: {Path(args.out).relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
