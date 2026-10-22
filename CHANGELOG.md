# Changelog

Todas as mudanças relevantes do FreeRusch são registradas aqui.

O formato segue o [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e o
projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

O escopo de cada release está em [docs/ROADMAP.md](docs/ROADMAP.md).

## [Não publicado] — destino: v0.1.0

Nada foi publicado ainda: não existe tag no repositório. O histórico já está na
`main` e nas branches de fase; o release `0.1.0` fecha quando o roadmap de release
fechar (ver [docs/ROADMAP.md](docs/ROADMAP.md)).

### Adicionado

- **Infraestrutura de release da família Craft.** `.github/workflows/release.yml`
  como chamador fino do pipeline compartilhado `storytold/craft-common-actions`
  (gatilho: branch `release`), `packaging/env.sh` como fonte única da versão e
  `assets/app-icon/` documentando os ícones exigidos por plataforma.
- **Licenciamento duplo e atribuição.** `LICENSE-MIT` + `LICENSE-APACHE`,
  `NOTICE`, e `ATTRIBUTION.md` gerado de `cargo metadata`
  (`tools/generate_attribution.py`), com a proveniência das tabelas embarcadas.
- **`AGENTS.md` / `CLAUDE.md`**, `clippy.toml` e `Cargo.lock` versionado,
  alinhando o repositório às convenções de CI e reprodutibilidade da família.
- **Tabelas de Rüsch completas.** As 186 tabelas do workbook de referência foram
  extraídas para `freerusch-core/data/rusch_tables.json` (56.693 coeficientes),
  com extrator `tools/extract_rusch_tables.py`. Células `-` viram ausência e
  erros/typos da planilha ficam registrados em `warnings` em vez de virarem
  número.
- **Momentos de carga permanente a partir das tabelas.** `uniform_load_moments()`
  usa os coeficientes fixos do painel (`M = k · q · lx²`). Reproduz o exemplo
  resolvido do workbook (aba `Main`, tabela Nr 65.1) nos seis momentos, com teste
  que cita as células de origem.
- **Carga móvel pelo método das tabelas de Rüsch.** `live_load_moments()`
  implementa `M = φ · (P·M_L + q·M_p + q·M_p')` com interpolação bilinear em
  `(lx/a, t/a)`; `impact_coefficient()` calcula `φ = 1,4 − 0,007·lx` e
  `load_spread_width_m()` o espraiamento da roda. Validado contra o exemplo do
  workbook (seis momentos dentro de 0,2 %) e contra as notas de aula do IME
  (φ = 1,351 e t = 0,69 m idênticos). Com isso **R2 deixa de ser bloqueio**.
- **GUI com abas.** Janela com as abas *Inputs*, *Results* e *Report*, construídas
  sobre os primitivos de `craft_ui::tabs` (`fit` + `Tab::show_at`), já que a
  `craft-ui` fornece o átomo de aba e a matemática de larguras, mas não um
  contêiner de abas. A aba *Results* mostra coeficientes, momentos e uma
  pré-visualização do diagrama.
- **Diagrama de momentos em SVG na memória de cálculo.** `freerusch_report::diagram_svg`
  gera um SVG autocontido (sem JavaScript nem arquivos externos), embutido
  automaticamente por `render_html`.
- **`MomentDiagram` em `freerusch-core`**, com `min_moment`, `max_moment` e
  `max_abs_moment`, para que a GUI e o relatório compartilhem a mesma
  representação do diagrama.
- **Crate `freerusch-project` e o formato `.frus`.** Projeto completo em JSON
  versionado (`schema_version`), validado antes de tocar o disco e gravado com
  escrita atômica (`craft-atomic`). Versões de esquema mais novas são recusadas em
  vez de mal interpretadas.
- **`VehicleGeometry` (NBR 7188:2013).** Cargas por eixo, distâncias de 4,00 m e
  1,25 m, faixa de 3,00 m e contagem de faixas `floor(b/3,00)` com mínimo de uma.
- **Aviso de resultados preliminares.** `freerusch_core::RUSCH_TABLES_PRELIMINARY`
  faz a GUI e a memória de cálculo declararem que os coeficientes de Rüsch ainda
  não são as tabelas homologadas.

### Alterado

- **README** descreve as abas, o arquivo `.frus` e o diagrama SVG no relatório.
- **Extensão do arquivo de projeto** padronizada em `.frus`; o README dizia
  `.frusch`, nome que nenhum outro ponto do repositório usava.
- Janela principal ampliada para acomodar a barra de abas e o diagrama.

### Corrigido

- **Cargas de multidão da NBR 7188:2013.** O motor trazia 4,5 kN/m² (TB-450) e
  2,4 kN/m² (TB-240); os valores normativos são **5,0 kN/m²** e **4,0 kN/m²**,
  atuando em conjunto com o veículo-tipo na região externa a ele. Isso altera todos
  os momentos calculados — no caso de referência (tabuleiro de 6 m, vão de 8 m,
  TB-450) o momento positivo vai de 18,0 para 20,0 kN·m/m.

### Notas

- 30 testes automatizados; `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo test --workspace` e `cargo build --workspace` verdes em Linux, Windows e
  macOS.
- As tabelas completas de Rüsch e a regra de `b_min`/`b_max` com o fator θ seguem
  bloqueadas por falta de fonte normativa — ver
  [Bloqueios](docs/ROADMAP.md#bloqueios).
