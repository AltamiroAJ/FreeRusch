# FreeRusch

**Cálculo estrutural de lajes de pontes pelo método de Rüsch — 100% Rust.**

O FreeRusch é um software livre da família **Craft**: reimplementação limpa e
aberta de ferramentas comerciais de dimensionamento de lajes de pontes
(método de Rüsch, tabelas de coeficientes, trens-tipo TB-450/TB-240 conforme a
NBR 7188:2013, interpolação de tabelas, diagramas de momentos fletores e
emissão de memória de cálculo em HTML).

> ⚠️ O FreeRusch é uma reimplementação independente. Ele não usa nem copia o
> software T-Rüsch; este é apenas referência funcional do domínio.

## Arquitetura

Monorepo organizado como **Cargo Workspace** (edition 2024, resolver v3), com
bibliotecas compartilhadas da família consumidas do repositório
[`craft-libs`](https://github.com/storytold/craft-libs) fixadas em um commit
(`rev` no `Cargo.toml` raiz):

```
freerusch/
├── Cargo.toml                  # workspace + [workspace.dependencies] (craft-* por git rev)
├── rust-toolchain.toml         # canal stable
├── .github/workflows/ci.yml    # fmt · clippy · test · build (Linux/Windows/macOS)
├── crates/
│   ├── freerusch-core/         # motor de cálculo PURO (sem UI):
│   │                           #  - RuschTable / EdgeCondition / interpolação linear
│   │                           #  - TrainType (TB-450 / TB-240, NBR 7188:2013)
│   │                           #  - bending_moment / design_moment / moment_diagram
│   └── freerusch-report/       # memória de cálculo em HTML:
│                               #  - render_html (escape seguro de entradas)
│                               #  - write_report via craft-atomic (escrita atômica)
└── apps/
    └── freerusch/              # binário GUI (eframe/egui + craft-ui/craft-settings)
```

Regras da família Craft:

- **Pure Rust** — sem C/C++ nem binários externos.
- Crates do projeto usam o prefixo `freerusch-`; bibliotecas da família são
  as `craft-*` (`craft-ui`, `craft-settings`, `craft-atomic`, `craft-print`).
- Erros com `thiserror`; configurações persistidas em `config.toml`.

*(Nota: `craft-core` ainda não existe no `craft-libs`; o núcleo GUI da família
é o próprio `craft-ui` sobre `egui`/`eframe`. A dependência fica registrada no
workspace para quando a lib for publicada.)*

## Executando

```sh
cargo run -p freerusch          # abre a janela "FreeRusch"
cargo test --workspace          # testes do motor de cálculo e do relatório
```

Defina `FREERUSCH_NO_PREFS=1` para rodar sem criar `config.toml` no perfil do
sistema (útil em CI/containers).

## Uso como biblioteca

```rust
use freerusch_core::{bending_moment, EdgeCondition, RuschTable, TrainType};

let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
let m = bending_moment(&table, &TrainType::TB450, /*l1=*/ 8.0, /*l2=*/ 0.0, /*q=*/ 25.0)?;
println!("M⁺ = {:.2} kN·m/m (α = {:.4})", m.max_positive, m.alpha);
```

## Status do domínio (roadmap)

Este repositório está no **esqueleto inicial** (Fase 1):

- [x] Workspace, CI multiplataforma, motor mínimo com testes.
- [ ] Tabelas de Rüsch completas (todos os vínculos e faixas l₂/l₁).
- [ ] Trens-tipo NBR 7188:2013 com roda-paleta, `b_min` e fator θ por faixa de influência.
- [ ] Diagramas de momento vetoriais (SVG) na memória de cálculo.
- [ ] Arquivo de projeto (`.frusch`) salvo atomicamente via `craft-atomic`.

## Licença

MIT OR Apache-2.0 (ver `LICENSE`).
