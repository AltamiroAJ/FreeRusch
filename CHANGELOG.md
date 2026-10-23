# Changelog

Todas as mudanças relevantes do FreeRusch são registradas aqui.

O formato segue o [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e o
projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

O escopo de cada release está em [docs/ROADMAP.md](docs/ROADMAP.md).

## [Não publicado]

### Adicionado

- **Aba Inputs reorganizada** no fluxo de um cálculo de laje de ponte: barra de
  menus (Arquivo, Opções, Resultados, Sobre), desenho esquemático vetorial à
  esquerda, parâmetros à direita, barra de ações, tabela de coeficientes com a
  interpolação em uso destacada, painel de resultados e rodapé com a unidade e o
  estado do cálculo.
- **Desenho esquemático em `egui::Painter`** (`apps/freerusch/src/schematic.rs`):
  seção da laje com pavimentação, símbolos de vinculação por borda (apoio,
  engaste, livre, indefinida), área de carga em trapézio, cotas de `lx`, `h` e `a`
  e o momento em exibição. Reage a cada mudança de campo.
- **Módulo `theme`** (`apps/freerusch/src/theme.rs`): paleta derivada do tema
  ativo (`TEXT`, `TEXT_WEAK`, `SURFACE`, `GRID`, `HIGHLIGHT`, `READ_ONLY`,
  `ACCENT`, `DANGER`) e tipografia de 14 pt para corpo e 16 pt para títulos.
  Contraste mínimo verificado por teste em tema claro e escuro.
- **Tema selecionável** (Sistema/Claro/Escuro) em **Opções → Tema**, persistido em
  `config.toml`; `FREERUSCH_THEME=dark|light` força o tema na inicialização.
- **Diálogos** de caminhos de arquivo e "Sobre", e menu **Resultados** com os
  esforços `Mxm`/`Mym`/`Mxe`.
- **API de leitura de coeficientes em `freerusch-core`** para a UI não calcular
  nada por conta própria: `interpolated_wheel_coefficient`,
  `interpolated_uniform_coefficient`, `panel_coefficient`, `wheel_columns`,
  `uniform_column` e a constante `WHEEL_SPACING_M` (`a = 2,00 m`, convenção das
  tabelas).
- O exemplo do workbook já abre calculado; o rodapé indica `Calculado` ou
  `Alterado após o último cálculo`.
- Capturas da aba Inputs nos dois temas: `docs/img/inputs-dark.png` e
  `docs/img/inputs-light.png`.

### Corrigido

- **Fundo da janela nunca era pintado.** O `App::ui` do eframe não pinta o fundo;
  a janela mostrava a cor de limpeza do backend. No tema escuro passava
  despercebido, no claro o texto escuro ficava sobre fundo preto. Agora o fundo
  é pintado a partir de `visuals().panel_fill`.
- **Campos somente leitura ilegíveis no tema claro**, por derivarem a superfície
  de cinzas internos do egui. `SURFACE` e `READ_ONLY` passam a ser derivados do
  texto do tema, com teste de contraste.
- **Coluna da tabela encolhendo**: `allocate_ui_with_layout` aloca apenas a largura
  usada, o que puxava o painel de resultados para o meio da tela. A coluna agora
  preenche a largura reservada.

## [0.1.0] — 2026-10-22

Primeiro release público. O binário de Windows x86_64 sai como artefato desta tag;
instaladores, assinatura de código e pacotes para as demais plataformas ficam para
a próxima versão (ver [docs/ROADMAP.md](docs/ROADMAP.md)).

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
- **Crate `freerusch-project` e o formato `.frus`.** Projeto completo em JSON
  versionado (`schema_version`), validado antes de tocar o disco e gravado com
  escrita atômica (`craft-atomic`). Versões de esquema mais novas são recusadas em
  vez de mal interpretadas.
- **`VehicleGeometry` (NBR 7188:2013).** Cargas por eixo, distâncias de 4,00 m e
  1,25 m, faixa de 3,00 m e contagem de faixas `floor(b/3,00)` com mínimo de uma.
- **Motor calculando pelas tabelas de Rüsch.** `design_moments()` combina carga
  permanente e carga móvel com `M_d = 1,35·M_g + 1,5·M_q`. A GUI passou a escolher
  a tabela por número (`Nr N.M`) e o relatório declara tabela, hipóteses, `φ`, `t`,
  `lx/a` e `t/a`.
- **`.frus` esquema v2.** Projeto passa a guardar tabela, espessuras, pavimento,
  distância entre rodas e cargas; arquivos v1 continuam carregando (campos novos
  caem no default).

### Alterado

- **README** descreve as abas, o arquivo `.frus` e o método de cálculo.
- **Extensão do arquivo de projeto** padronizada em `.frus`; o README dizia
  `.frusch`, nome que nenhum outro ponto do repositório usava.

### Removido

- **Aviso de resultados preliminares.** `RUSCH_TABLES_PRELIMINARY` e os avisos na
  GUI e no HTML foram removidos: deixaram de ser verdade quando o motor passou a
  usar as tabelas embarcadas.
- **Motor simplificado histórico.** `RuschTable`, `EdgeCondition`, `RuschPoint`,
  `RuschRow`, `MomentResult`, `MomentDiagram`, `design_moment`, `bending_moment`,
  `moment_diagram` e `load_per_lane_knm` foram removidos do crate. Eram o motor do
  esqueleto inicial, que não usava as tabelas embarcadas e podia produzir números
  enganosos se chamado; agora existe **um único caminho de cálculo**.
- **Diagrama de momentos sintético** da GUI e do relatório: era uma parábola
  desenhada, não o diagrama calculado. O relatório agora traz a tabela de momentos
  por resultante.
- Janela principal ampliada para acomodar a barra de abas e o diagrama.

### Corrigido

- **Cargas de multidão da NBR 7188:2013.** O motor trazia 4,5 kN/m² (TB-450) e
  2,4 kN/m² (TB-240); os valores normativos são **5,0 kN/m²** e **4,0 kN/m²**,
  atuando em conjunto com o veículo-tipo na região externa a ele. Isso altera todos
  os momentos calculados — no caso de referência (tabuleiro de 6 m, vão de 8 m,
  TB-450) o momento positivo vai de 18,0 para 20,0 kN·m/m.

### Notas

- 38 testes automatizados: 21 no core, 11 no projeto, 5 no relatório e 1 doctest.
- Binários de release gerados e verificados em Windows x86_64 e Linux x86_64
  (`cargo build --release --locked`).
- O executável do Windows não é assinado e não carrega metadados de versão; ambos
  dependem de certificado de assinatura e de um recurso de build.
- Bloqueios restantes, nenhum deles com efeito sobre o cálculo: a lista de recursos
  do T-Rüsch (R3) e a semântica dos prefixos `¾`/`(+)`/`-`/`_` nos rótulos das
  tabelas (R1) — ver [Bloqueios](docs/ROADMAP.md#bloqueios).
