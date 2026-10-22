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
├── docs/ROADMAP.md             # roadmap, critérios de aceite e bloqueios
├── CHANGELOG.md                # histórico de mudanças (Keep a Changelog)
├── tools/                      # extrator das tabelas de Rüsch para o core
├── .github/workflows/ci.yml    # fmt · clippy · test · build (Linux/Windows/macOS)
├── crates/
│   ├── freerusch-core/         # motor de cálculo PURO (sem UI):
│   │                           #  - RuschTable / EdgeCondition / interpolação linear
│   │                           #  - TrainType / VehicleGeometry (NBR 7188:2013)
│   │                           #  - design_moment / moment_diagram / MomentDiagram
│   ├── freerusch-report/       # memória de cálculo em HTML:
│   │                           #  - render_html (escape seguro de entradas)
│   │                           #  - diagram_svg (diagrama de momentos inline)
│   │                           #  - write_report via craft-atomic (escrita atômica)
│   └── freerusch-project/      # arquivo de projeto `.frus` (JSON versionado):
│                               #  - Project, save/load com validação prévia
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

A janela tem três abas: **Inputs** (tabela de Rüsch e cargas), **Results**
(momentos característicos e de projeto, por resultante) e **Report** (memória de
cálculo). O projeto completo é gravado em `.frus` (JSON versionado, escrita
atômica via `craft-atomic`) e a memória de cálculo é exportada em HTML
autocontido, declarando a tabela usada, as hipóteses e os coeficientes — sem
arquivos externos nem JavaScript.

Defina `FREERUSCH_NO_PREFS=1` para rodar sem criar `config.toml` no perfil do
sistema (útil em CI/containers).

## Uso como biblioteca

```rust
use freerusch_core::{RuschDesignInput, design_moments};

// Exemplo resolvido do workbook: tabela Nr 65.1, lx = 7,00 m.
let input = RuschDesignInput {
    table_number: 65,
    table_variant: Some(1),
    lx_m: 7.0,
    slab_m: 0.26,
    pavement_m: 0.07,
    wheel_spacing_m: 2.0,
    permanent_load_kn_m2: 10.5209,
    wheel_load_kn: 75.0,
    uniform_load_kn_m2: 5.0,
};
let moments = design_moments(&input)?;
for m in &moments {
    println!(
        "{:<4} M_g {:8.3}  M_q {:8.3}  M_d {:8.3}",
        m.resultant, m.permanent, m.live, m.design
    );
}
```

As **186 tabelas** de Rüsch (DIN/IBSW) estão embarcadas em
`crates/freerusch-core/data/rusch_tables.json` — 56.693 coeficientes — e são
endereçadas por número, como nas tabelas impressas:

```rust
use freerusch_core::RuschTableData;

let table = RuschTableData::find(65, Some(1)).expect("tabela Nr 65.1");
assert_eq!(table.sheet, "Nr 65.1");
```

> Os caminhos de cálculo são validados contra **duas fontes independentes**: o
> exemplo resolvido do workbook do IME (seis momentos com erro < 0,2 %) e as notas
> de aula do mesmo curso (φ = 1,351 e espraiamento t = 0,69 m idênticos). Detalhes
> e fontes em [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Roadmap

O planejamento completo — itens, critérios de aceite, dependências e bloqueios —
está em [`docs/ROADMAP.md`](docs/ROADMAP.md). O histórico de mudanças fica no
[`CHANGELOG.md`](CHANGELOG.md).

Resumo:

- **v0.1.0** (em andamento) — motor de cálculo sobre as tabelas de Rüsch
  embarcadas, com carga permanente e carga móvel validadas contra o exemplo do
  workbook, e release no padrão da família Craft.
- **v0.2.0** — armadura e ELU (NBR 6118), combinações de ações, `.frus` v2.
- **v1.0.0** — consolidação: carga de pneu, i18n pt-BR/en-US, release com checksums.

> **Escopo e limites desta versão.** O FreeRusch aplica o método das tabelas de
> Rüsch, com coeficiente de impacto `φ = 1,4 − 0,007·lx` e espraiamento de roda
> `t = √(0,20·0,50) + 2·(pav + h/2)`. A carga móvel é combinada com a permanente
> por `M_d = 1,35·M_g + 1,5·M_q`. **Não** cobre ainda: cargas de passeio,
> dimensionamento de armadura (NBR 6118), punção por pneu e posicionamento
> automático crítico do trem-tipo — todos itens abertos no roadmap.
>
> As cargas de multidão seguem a NBR 7188:2013: **5,0 kN/m²** (TB-450) e
> **4,0 kN/m²** (TB-240), atuando em conjunto com o veículo-tipo.

## Distribuição e release

O FreeRusch segue o padrão de release da família Craft: o workflow
[`.github/workflows/release.yml`](.github/workflows/release.yml) é um chamador fino
que delega ao pipeline compartilhado `storytold/craft-common-actions`, disparado
por push na branch `release`. Ele produz macOS universal (DMG assinado e
notarizado), Windows MSI (x64/x86/ARM64), Linux AppImage/deb/rpm/tar.gz, Flatpak,
Arch, RISC-V e, opcionalmente, wasm — e publica um *draft release* com
`SHA256SUMS.txt`.

A versão vem de `[workspace.package] version` no `Cargo.toml` raiz, lida por
[`packaging/env.sh`](packaging/env.sh) e validada como semver pelo pipeline.

> **Pendências para o primeiro release assinado:** os segredos de assinatura
> (Apple Developer ID + notarização, Azure Trusted Signing) ainda não estão
> configurados no ambiente `release`, e o ícone do aplicativo ainda não foi
> fornecido — ver [`assets/app-icon/README.md`](assets/app-icon/README.md). Até
> lá, o pipeline roda apenas como *draft* de teste com `workflow_dispatch`.

Convenções de desenvolvimento, gate de qualidade e proveniência dos dados estão
em [`AGENTS.md`](AGENTS.md).

## Licença

MIT **ou** Apache-2.0, à sua escolha — ver [`LICENSE-MIT`](LICENSE-MIT) e
[`LICENSE-APACHE`](LICENSE-APACHE).

Material de terceiros, referências normativas e a proveniência das tabelas de
Rüsch embarcadas estão em [`NOTICE`](NOTICE) e
[`ATTRIBUTION.md`](ATTRIBUTION.md).
