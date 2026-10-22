# Roadmap do FreeRusch

Este é o documento **único** de planejamento do FreeRusch. O `README.md` resume e
aponta para cá; os arquivos em `prompts/` são o **registro histórico** de como cada
fase foi especificada e não são reescritos.

- **Referência funcional do domínio:** [T-Rüsch (TQS)](https://www.tqs.com.br/apps/t-rusch/a6q52fh86z).
  O FreeRusch é uma reimplementação limpa e independente: não usa nem copia esse
  software, seus dados ou seu código. Paridade é objetivo de produto, não de código.
- **Regra de integridade técnica:** nenhum valor normativo entra no código sem
  fonte citável. Item que dependa de texto de norma fica em [Bloqueios](#bloqueios)
  até a fonte ser fornecida.

## Como ler

| Marca | Significado |
|---|---|
| `[x]` | concluído e verificado por teste |
| `[ ]` | pendente |
| **BLOQUEADO** | depende de fonte externa que não temos (ver [Bloqueios](#bloqueios)) |
| **A VALIDAR** | inferido do domínio; precisa ser conferido contra a referência comercial |

Prefixos de ID: `R` bloqueio/risco · `D` domínio de engenharia · `I` infraestrutura ·
`P` produto e DX · `X` release.

## Estado atual (`0.1.0-dev`)

| Item | Estado |
|---|---|
| Versão no workspace | `0.1.0`, sem tag publicada |
| Crates | `freerusch-core`, `freerusch-report`, `freerusch-project`, `apps/freerusch` |
| GUI | abas Inputs / Results / Report |
| Relatório | HTML autocontido com diagrama SVG embutido |
| Projeto | `.frus` JSON versionado, escrita atômica |
| Testes | 30 (13 core · 9 project · 7 report · 1 doctest) |
| CI | fmt · clippy `-D warnings` · test · build em Linux/Windows/macOS |
| Tabelas de Rüsch | 186 de 186 embarcadas (workbook do IME) — ver [R1](#r1--extração-concluída-integração-pendente) |
| Confiabilidade do resultado | **preliminar** — ver aviso abaixo |

> **Aviso.** Os coeficientes de Rüsch embutidos são uma base didática, não as
> tabelas homologadas. Enquanto `freerusch_core::RUSCH_TABLES_PRELIMINARY` for
> `true`, a GUI e a memória de cálculo exibem esse aviso. Não use para projeto
> executivo.

---

# v0.1.0 — Domínio confiável

Objetivo: o primeiro release em que **o número que sai na tela pode ser levado a
um projeto**. Isso exige as tabelas homologadas, o carregamento normativo completo
e a memória de cálculo declarando suas próprias hipóteses.

## Bloqueadores

| ID | Item | Precisa de |
|---|---|---|
| **R2** | Regra normativa de largura efetiva `b_min`/`b_max` e do fator de distribuição transversal θ por faixa de influência | cláusula da NBR 7188:2013 |
| **R3** | Conferir a lista de recursos do T-Rüsch contra este roadmap | página do fornecedor — na última verificação retornou HTTP 500 |

### R1 — extração concluída, integração pendente

A fonte das tabelas foi obtida: `Tabelas Rusch Final - IME.xls`, com **186 tabelas**
(DIN/IBSW) em 188 planilhas, 13 linhas de `lx/a` (0,5 a 10) e 29 layouts de coluna
distintos. As resultantes vão de `Mxm`/`Mym` (centro da laje) a `Mxr`/`Myr`
(meio da borda livre), `Mxe`/`Mye` (meio da borda) e `Qx` (cortante).

Pipeline implementado e executado para o conjunto completo:

```
.xls  --(LibreOffice)-->  .xlsx  --tools/extract_rusch_tables.py --all-->
      crates/freerusch-core/data/rusch_tables.json   (único artefato, versionado)
```

Decisões registradas: a fonte **não** é versionada (o `.xls` tem SHA-256 gravado no
JSON); endereçamento por **número da tabela** (`Nr N.M`), já que não há legenda;
rótulos de coluna preservados **crus**; célula `-` vira ausência, nunca `0,0`.
O JSON é embutido com `include_str!` e desserializado uma vez em `rusch_tables()`.

**Estado:** **186 de 186 tabelas** embarcadas — 56.693 coeficientes, 13 linhas por
tabela, índices `(número, variante)` únicos e verificados por teste. 181 das 186
tabelas também trazem os **coeficientes fixos do painel** do cabeçalho
(`Mxm = 0,063`, …), que antes ficavam de fora da extração.

**Validado contra o exemplo resolvido do workbook (aba `Main`):** o caminho de
**carga permanente** foi reproduzido célula a célula. Para a tabela Nr 65.1 com
`lx = 7,00 m` e `q = 10,5209 kN/m²`, `uniform_load_moments()` devolve exatamente
`32,478 · 5,67077 · 32,478 · 64,4406 · 45,8817 · 10,3105` — os seis valores de
`Main!I24..I29` — o que confirma que os coeficientes de cabeçalho multiplicam
`q · lx²`. Esse caminho **não** depende de R2.

**Tentativa registrada (negativa) para a carga móvel:** testou-se a hipótese
`M = j · (L·P + c·q)` (com `c` = `p` ou `p'`) por mínimos quadrados sobre as 13
linhas × 4 colunas `t/a` × 64 combinações de `p`/`p'`. O melhor ajuste deixa
resíduo de **39 kN·m/m** e `q` negativo — ou seja, **não** é esse o modelo. A
carga móvel exige a descrição do método (R2/legenda), não engenharia reversa.

**Lacunas registradas na fonte (11, em `warnings` no JSON):** a planilha traz
células com erro (`=#REF!`, `#VALUE!`), números ilegíveis (`2,.82`, `3,.08`,
`1,.26`) e cabeçalhos cujo rótulo não casa com o nome da planilha
(`Nr 15.2`, `28.2`, `41.2`, `97.2`). Todas viram ausência declarada, nenhuma vira
número inventado; conferir contra o documento original antes de usar essas células.

Falta para fechar R1:

- decidir a semântica dos prefixos de rótulo `¾`, `(+)`, `-`, `_`;
- o modelo da **carga móvel** (roda + multidão) — ver tentativa negativa acima;
- **integrar** ao motor: `design_moment` ainda calcula com os coeficientes
  simplificados de `RuschTable`, e `RUSCH_TABLES_PRELIMINARY` continua `true` de
  propósito.

**Consequência:** com R1 extraído, o único bloqueio externo restante para a v0.1.0
confiável é **R2**.

## Domínio de engenharia

| ID | Item | Aceite | Depende |
|---|---|---|---|
| D1 | Coeficiente de impacto vertical φ (NBR 7188) aplicado aos esforços | teste com φ explícito e citação da origem no código | — |
| D2 | Cargas de passeio e passarela: 3,0 kN/m² (geral) e 5,0 kN/m² (elemento direto/passarela) como entradas opcionais | teste de combinação com e sem passeio | — |
| D3 | `design_moment` passa a usar `VehicleGeometry::lane_count`; remove o cálculo de faixas duplicado hoje em `lib.rs` | teste garantindo uma única regra de contagem de faixas | R2 |
| D4 | Eliminar `bending_moment`: ele ignora `_l2_over_l1` e fixa `spans=1` e `b=6,0 m`, parecendo genérico sem ser | função removida ou renomeada para exemplo; doctest e README ajustados | — |
| D5 | Diagrama de momentos: usar a distribuição real por seção **ou** renomear para `envelope_diagram` e rotular como ilustrativo na GUI e no HTML | teste de forma + texto explícito ao usuário | R1 |
| D6 | `SlabInput` ganha `spans` e `edge`; o HTML declara vãos e condição de borda usados | asserção no HTML contendo vãos e "biapoiada"/"engastada" | — |
| D7 | Validar `spans` na borda (tabela cobre 1–4 biapoiada e 1–3 engastada) com erro específico | teste para 4 vãos engastados | — |
| D8 | Roda-paleta: área de contato do pneu e dispersão a 30° na espessura | teste contra exemplo manual | R2 |
| D9 | Revisar `load_per_lane_knm` à luz de R2 (hoje é uma simplificação de largura efetiva) | coeficientes documentados com origem | R2 |
| D10 | Vãos múltiplos: avaliar posicionamento crítico do trem-tipo por vão | estudo registrado + teste | R1 |

## Infraestrutura e reprodutibilidade

| ID | Item | Aceite |
|---|---|---|
| I1 | Commitar `Cargo.lock` (hoje ignorado) — binário com dependências git fixadas por `rev` | `Cargo.lock` versionado; CI usa `--locked` |
| I2 | Resolver `craft-core`: declarado no workspace mas inexistente no `craft-libs` — TODO rastreável ou remoção | `cargo metadata` sem dependência fantasma |
| I3 | `eframe` habilita `wayland`/`x11` em todas as plataformas; restringir a Linux | build Windows/macOS inalterado |
| I4 | Cache do CI por `Cargo.lock` (a chave atual nunca casa porque o arquivo não existe) | chave de cache muda quando o lock muda |

## Produto e DX

| ID | Item | Aceite |
|---|---|---|
| P1 | `CHANGELOG.md` no formato Keep a Changelog e bump `0.1.0` nos quatro crates | entrada com D1–D7 |
| P2 | Casos de validação da literatura (Rüsch × FEM), com tolerância declarada | ≥3 casos em teste |
| P3 | README: corrigir a árvore de arquitetura e apontar para este roadmap | link válido |
| P4 | CLI headless `freerusch --calc projeto.frus --html saida.html` | roda em CI sem display |

## Release

| ID | Item | Aceite |
|---|---|---|
| X1 | Tag `v0.1.0` + GitHub Release com binários das 3 plataformas e SHA-256 | `cargo-dist` ou artifacts no `ci.yml` |
| X2 | `RUSCH_TABLES_PRELIMINARY = false` — **somente** depois de R1 | avisos somem da GUI e do HTML |

### Critério de saída da v0.1.0

1. R1 e R2 fechados; D3, D8 e D9 implementados a partir deles.
2. D1, D2, D4, D6, D7 concluídos com testes.
3. D5 resolvido (real ou renomeado e rotulado).
4. I1 e I4 concluídos; CI verde nas 3 plataformas com `--locked`.
5. CHANGELOG publicado e tag `v0.1.0` com binários.
6. Nenhum aviso de "preliminar" visível na GUI nem no HTML.

---

# v0.2.0 — Fase 3: armadura, ELU e `.frus` v2

Origem: [phase-3-uls-reinforcement](prompts/phase-3-uls-reinforcement.md).

| ID | Item |
|---|---|
| E1 | Módulo `reinforcement` em `freerusch-core`: `required_as`, z ≤ 0,9d, verificação x/d, As,mín (NBR 6118:2023) |
| E2 | Concreto parametrizado (fck 25–50 MPa, γc = 2,1) e aço CA-50/CA-25/CA-60 (γs = 1,15) |
| E3 | `bar_spacing`: seleção automática entre 10 / 12,5 / 16 / 20 mm, espaçamento 2–20 cm, cobrimento 2,5 cm |
| E4 | Combinações ULS com γf; `design_moments` devolvendo M_k e M_sd lado a lado |
| E5 | ELS: flecha aproximada (alerta > L/250) e fissuração (σ_s ≤ 300 MPa) |
| E6 | `.frus` v2 com `schema_version`, loader retrocompatível v1 e teste de migração |
| E7 | Aba **Reinforcement** na GUI, com destaque de pontos onde x/d excede o limite |
| E8 | Exportação CSV dos momentos e PDF via `craft-print` |
| E9 | CHANGELOG 0.2.0 e tag `v0.2.0` |

# v1.0.0 — Fase 4: consolidação

Origem: [phase-4-consolidation-v1](prompts/phase-4-consolidation-v1.md).

| ID | Item |
|---|---|
| F1 | Carga nos pneus (impacto local): pressão de contato, dispersão 30°, momento local e punção |
| F2 | Módulo `distribution` unificado (trem-tipo + pneu) e espessura mínima h ≥ lx/16 ou lx/20 |
| F3 | i18n pt-BR/en-US com `craft-i18n`, idioma persistido em `config.toml` |
| F4 | `#![warn(missing_docs)]` limpo em core e report; `docs/architecture.md` |
| F5 | Exemplos executáveis (`crates/freerusch-core/examples/`) rodando em CI |
| F6 | `SECURITY.md`, `CONTRIBUTING.md`, `CHANGELOG` consolidado, release `v1.0.0` com checksums |

# Backlog (sem versão comprometida)

- Posicionamento crítico automático do trem-tipo (longitudinal e transversal).
- Tabuleiro curvo / laje esconsa.
- Verificação de fadiga.
- Comparação automatizada com modelo FEM.
- Benchmarks com `criterion` (interpolação < 1 µs/op).
- Exportação DXF dos diagramas.
- Tema escuro e preferências de UI.
- Múltiplos trens simultâneos por faixa.

---

# Bloqueios

| ID | Bloqueio | O que destrava |
|---|---|---|
| R2 | `b_min`/`b_max` e θ | trecho da NBR 7188:2013 |
| R3 | Lista de recursos do T-Rüsch | página do fornecedor acessível, ou sua descrição |
| R1 | Semântica dos prefixos `¾`/`(+)`/`-`/`_` nos rótulos das tabelas | confirmação no documento original (não bloqueia a extração) |

Enquanto R2 estiver aberto, o restante da v0.1.0 pode avançar — mas o release não
fecha.

# Histórico de fases

| Fase | Prompt | PR | Estado |
|---|---|---|---|
| 1 | [phase-1-init-workspace](prompts/phase-1-init-workspace.md) | #1 | concluída |
| 2 | [phase-2-engineering-domain](prompts/phase-2-engineering-domain.md) | #11 + branches | em andamento |
| 3 | [phase-3-uls-reinforcement](prompts/phase-3-uls-reinforcement.md) | — | não iniciada |
| 4 | [phase-4-consolidation-v1](prompts/phase-4-consolidation-v1.md) | — | não iniciada |

> **Nota sobre numeração.** Os prompts chamam a Fase 2 de `0.2.0` e a Fase 3 de
> `0.3.0`. Como não existe nenhuma tag e o workspace já está em `0.1.0`, este
> roadmap numera os releases de forma monotônica a partir de `0.1.0`:
> v0.1.0 = Fase 2 · v0.2.0 = Fase 3 · v1.0.0 = Fase 4.

# Manutenção deste documento

- Todo item novo entra com ID, critério de aceite e dependência declarada.
- Item concluído é marcado `[x]` **e** referenciado no `CHANGELOG.md`.
- Item que dependa de fonte externa vai para [Bloqueios](#bloqueios), nunca é
  resolvido por estimativa.
- Este arquivo substitui a lista solta que existia no README.
