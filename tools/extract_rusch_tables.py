#!/usr/bin/env python3
"""Extrai as tabelas de Rüsch do workbook do IME para JSON + Rust.

A planilha de origem (`Tabelas Rusch Final - IME.xls`, formato BIFF legado) não é
versionada. Este script consome a versão `.xlsx` já convertida e produz dois
artefatos determinísticos:

  crates/freerusch-core/data/rusch_tables.json   (auditoria, legível)
  crates/freerusch-core/src/rusch_data.rs        (compilado, @generated)

Regras de fidelidade:
  * nenhum valor é interpolado, arredondado ou reinterpretado;
  * qualquer célula ausente na região de dados é erro fatal, não `null`;
  * rótulos de coluna são preservados crus (inclusive prefixos `¾`, `(+)`, `-`, `_`),
    porque o significado desses prefixos ainda não foi confirmado com o documento
    original.

Conversão do `.xls` (uma vez, com o LibreOffice empacotado):
  node <libreoffice-kit>/cli.js convert \
      --input "Tabelas Rusch Final - IME.xls" --output target/ref/tabelas-rusch.xlsx

Uso:
  python tools/extract_rusch_tables.py --xlsx target/ref/tabelas-rusch.xlsx
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path

import openpyxl

# Recorte inicial: cinco tabelas cobrindo os layouts distintos encontrados.
# O extrator é genérico — ampliar esta lista traz o resto do conjunto.
SHEETS = ["Nr 1.1", "Nr 4.1", "Nr 16.1", "Nr 99", "Nr 27.1"]

DATA_ROWS = 13  # lx/a = 0.5, 1, 1.5, 2, 2.5, 3, 4, 5, 6, 7, 8, 9, 10

REPO = Path(__file__).resolve().parent.parent
OUT_JSON = REPO / "crates/freerusch-core/data/rusch_tables.json"
OUT_RS = REPO / "crates/freerusch-core/src/rusch_data.rs"


DASHES = {"-", "–", "—", ""}


def as_optional_float(value, where: str):
    """Converte uma célula em float, ou `None` para os traços de "sem valor".

    Nas tabelas, `-` significa que o coeficiente não se aplica (contribuição
    nula). Isso é preservado como ausência explícita, nunca como 0,0 inventado.
    """
    if isinstance(value, bool):
        raise SystemExit(f"célula booleana em {where}")
    if value is None:
        return None
    if isinstance(value, (int, float)):
        return float(value)
    text = str(value).strip()
    if text in DASHES:
        return None
    try:
        return float(text.replace(",", "."))
    except ValueError as exc:
        raise SystemExit(f"valor não numérico em {where}: {value!r}") from exc


def find_header(grid):
    for r, row in enumerate(grid):
        for c, v in enumerate(row):
            if isinstance(v, str) and v.strip().lower() == "lx/a":
                return r, c
    return None, None


def parse_sheet(name, grid):
    header_row, label_col = find_header(grid)
    if header_row is None:
        raise SystemExit(f"{name!r}: linha 'lx/a' não encontrada")
    if header_row + 3 >= len(grid):
        raise SystemExit(f"{name!r}: cabeçalho incompleto")

    group_row = grid[header_row]
    var_row = grid[header_row + 1]
    ta_row = grid[header_row + 2]
    letter_row = grid[header_row + 3]

    columns = []
    group = ""
    for c in range(label_col + 1, max(len(r) for r in grid[: header_row + 5])):
        g = group_row[c] if c < len(group_row) else None
        if isinstance(g, str) and g.strip():
            group = g.strip()
        var = var_row[c] if c < len(var_row) else None
        ta = ta_row[c] if c < len(ta_row) else None
        letter = letter_row[c] if c < len(letter_row) else None
        # A linha das letras (L / p / p') é o discriminador: colunas `p'` não
        # têm `t/a` próprio (herdam "für alle Werte t/a"), mas têm letra.
        if not (isinstance(letter, str) and letter.strip()):
            continue
        kind = "wheel" if isinstance(letter, str) and letter.strip() == "L" else "uniform"
        columns.append(
            {
                "index": c,
                "group": group,
                "kind": kind,
                "letter": (letter.strip() if isinstance(letter, str) else ""),
                "ta": (float(ta) if isinstance(ta, (int, float)) and not isinstance(ta, bool) else None),
                "values": [],
            }
        )

    if not columns:
        raise SystemExit(f"{name!r}: nenhuma coluna de coeficientes reconhecida")

    rows = []
    for r in range(header_row + 4, len(grid)):
        row = grid[r]
        label = row[label_col] if label_col < len(row) else None
        if not isinstance(label, (int, float)) or isinstance(label, bool):
            if rows:
                break
            continue
        rows.append(float(label))
        for col in columns:
            c = col["index"]
            cell = row[c] if c < len(row) else None
            col["values"].append(as_optional_float(cell, f"{name}!r{r + 1}c{c + 1}"))

    if len(rows) != DATA_ROWS:
        raise SystemExit(f"{name!r}: {len(rows)} linhas de dados, esperado {DATA_ROWS}")
    for col in columns:
        if len(col["values"]) != DATA_ROWS:
            raise SystemExit(f"{name!r}: coluna {col['group']!r} incompleta")

    # Cabeçalho: 'Nr. N.M' e o parâmetro ly/lx (ou l/a).
    number = variant = None
    for row in grid[:header_row]:
        for v in row:
            if isinstance(v, str):
                m = re.match(r"^Nr\.?\s*(\d+)(?:\.(\d+))?$", v.strip())
                if m:
                    number = int(m.group(1))
                    variant = int(m.group(2)) if m.group(2) else None

    param = ratio_raw = None
    for row in grid[:header_row]:
        for c, v in enumerate(row):
            if isinstance(v, str) and re.match(r"^(ly/lx|l/a)\s*=", v.strip()):
                param = v.strip().split("=")[0].strip()
                for k in range(c + 1, len(row)):
                    if row[k] is not None:
                        ratio_raw = str(row[k]).strip()
                        break

    ratio = None if ratio_raw in (None, "∞") else float(ratio_raw.replace(",", "."))

    return {
        "sheet": name,
        "number": number,
        "variant": variant,
        "param": param,
        "ratio_raw": ratio_raw,
        "ratio": ratio,
        "ratio_is_infinite": ratio_raw == "∞",
        "rows": rows,
        "columns": columns,
    }


def rust_float(v: float) -> str:
    if v == int(v) and abs(v) < 1e15:
        return f"{v:.1f}"
    return repr(v)


def rs(text: str) -> str:
    """Literal de string para Rust: UTF-8 cru, sem escapes \\uXXXX do JSON."""
    return json.dumps(text, ensure_ascii=False)


def emit_rust(tables, source_name, source_sha) -> str:
    lines = [
        "// @generated por tools/extract_rusch_tables.py — NÃO EDITAR À MÃO.",
        "//!",
        "//! Dados das tabelas de Rüsch extraídos de",
        f"//! `{source_name}` (SHA-256 `{source_sha}`).",
        "//!",
        "//! Recorte parcial: apenas as tabelas listadas em `SHEETS` no extrator.",
        "//! Os rótulos de coluna são preservados crus, incluindo prefixos ainda",
        "//! não interpretados (`¾`, `(+)`, `-`, `_`).",
        "",
        "use super::{RuschColumn, RuschColumnKind, RuschTableData};",
        "",
        "/// Tabelas extraídas, na ordem em que aparecem no extrator.",
        "pub static TABLES: &[RuschTableData] = &[",
    ]
    for t in tables:
        variant = "None" if t["variant"] is None else f"Some({t['variant']})"
        lines.append("    RuschTableData {")
        lines.append(f"        sheet: {rs(t['sheet'])},")
        lines.append(f"        number: {t['number']},")
        lines.append(f"        variant: {variant},")
        lines.append(f"        param: {rs(t['param'] or '')},")
        lines.append(f"        ratio_raw: {rs(t['ratio_raw'] or '')},")
        lines.append(f"        ratio_is_infinite: {str(t['ratio_is_infinite']).lower()},")
        rows = ", ".join(rust_float(r) for r in t["rows"])
        lines.append(f"        rows: &[{rows}],")
        lines.append("        columns: &[")
        for col in t["columns"]:
            ta = "None" if col["ta"] is None else f"Some({rust_float(col['ta'])})"
            kind = "Wheel" if col["kind"] == "wheel" else "Uniform"
            values = ", ".join(
                "None" if v is None else f"Some({rust_float(v)})"
                for v in col["values"]
            )
            lines.append("            RuschColumn {")
            lines.append(f"                group: {rs(col['group'])},")
            lines.append(f"                letter: {rs(col['letter'])},")
            lines.append(f"                ta: {ta},")
            lines.append(f"                kind: RuschColumnKind::{kind},")
            lines.append(f"                values: &[{values}],")
            lines.append("            },")
        lines.append("        ],")
        lines.append("    },")
    lines.append("];")
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--xlsx", required=True, help="workbook .xlsx convertido")
    parser.add_argument("--source-name", default="Tabelas Rusch Final - IME.xls")
    parser.add_argument("--source-sha256", default="")
    args = parser.parse_args()

    xlsx = Path(args.xlsx)
    workbook = openpyxl.load_workbook(xlsx, read_only=True, data_only=False)

    missing = [s for s in SHEETS if s not in workbook.sheetnames]
    if missing:
        raise SystemExit(f"planilhas ausentes: {missing}")

    tables = []
    for name in SHEETS:
        ws = workbook[name]
        grid = [[c.value for c in row] for row in ws.iter_rows(max_row=40, max_col=40)]
        table = parse_sheet(name, grid)
        tables.append(table)
        kinds = {}
        for col in table["columns"]:
            kinds[col["letter"]] = kinds.get(col["letter"], 0) + 1
        dashes = sum(1 for c in table["columns"] for v in c["values"] if v is None)
        print(
            f"  {name:<10} nr={table['number']}.{table['variant']} "
            f"{table['param']}={table['ratio_raw']} "
            f"cols={len(table['columns'])} {kinds} sem_valor={dashes}"
        )
    workbook.close()

    payload = {
        "source": {
            "name": args.source_name,
            "sha256": args.source_sha256,
            "xlsx_sha256": hashlib.sha256(xlsx.read_bytes()).hexdigest().upper(),
            "tool": "tools/extract_rusch_tables.py",
        },
        "rows_axis": {"name": "lx/a", "values": tables[0]["rows"]},
        "row_count": DATA_ROWS,
        "scope": {"note": "recorte parcial", "sheets": SHEETS},
        "tables": tables,
    }

    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    OUT_JSON.write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    OUT_RS.write_text(
        emit_rust(tables, args.source_name, args.source_sha256), encoding="utf-8"
    )

    total = sum(len(c["values"]) for t in tables for c in t["columns"])
    print(f"\ntabelas={len(tables)} valores={total}")
    print(f"escrito: {OUT_JSON.relative_to(REPO)}")
    print(f"escrito: {OUT_RS.relative_to(REPO)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
