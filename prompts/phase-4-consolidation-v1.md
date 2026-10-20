# Prompt — Fase 4: Consolidação, i18n, propriedades locais e 1.0.0

> Pré-requisito: enviar APÓS o merge da Fase 3 (`feat/uls-reinforcement`).
> Escopo: refinamentos finais de domínio, internacionalização, robustez e preparação do release 1.0.0.

## Contexto
FreeRusch em https://github.com/AltamiroAJ/FreeRusch já possui: tabelas de Rüsch completas com interpolação, trens-tipo TB-450/TB-240 (NBR 7188:2013), diagramas de momentos, GUI multi-aba (Inputs/Results/Report/Reinforcement) via craft-ui/egui, dimensionamento ELU (NBR 6118:2023), combinações ULS, memória de cálculo HTML/PDF/CSV, `.frus` v2 retrocompatível, CI verde nas 3 plataformas e tag v0.3.0 publicada. Inspecione a `main` mesclada antes de começar e estenda as interfaces existentes.

## Tarefas

### 1. Carga nos pneus (impacto local) — NBR 7188 item de carga isolada
- Pressão de contato dos pneus (250 kN distribuídos em área de contato conforme norma, com ângulo de dispersão 30° pela espessura do pavimento/concreto).
- Momento local no topo da laje (fórmula clássica de Pucher/Young para placa sobre meio elástico ou simplificação Ingólfsson documentada) e verificação de punching se h pequeno.
- API: `fn tire_local_moment(p_kn: f64, h: f64, pave: f64, ...) -> Result<f64, CoreError>`; entra na combinação ULSS como ação especial.
- Testes: valor conferido com exemplo manual; efeito da espessura de pavimento reduzindo momento.

### 2. Propriedades locais e pré-dimensionamento
- Espessura mínima da laje (h ≥ lx/16 articulado, h ≥ lx/20 contínuo — regras práticas citadas em comentário de origem) com sugestão automática na GUI e alerta quando violada.
- Largura efetiva de distribuição de carga concentrada (b_min/b_max) revisada contra a Fase 2; unificar em um único módulo `distribution` usado por trem-tipo e pneu.

### 3. i18n (pt-BR / en-US)
- Strings da GUI e do relatório externalizadas (enum `Lang` + tabela estática ou crate leve pure-Rust como `rust-i18n`/`fluent` — escolher mantendo pure Rust; se houver dependência com C, descartar).
- Idioma persistido em `config.toml` via `craft-settings`; padrão pt-BR; relatório usa idioma selecionado (unidades permanecem SI).
- Teste: render do relatório em ambos os idiomas contém termos esperados ("Memória de cálculo"/"Calculation memo").

### 4. Qualidade e DX
- `#![warn(missing_docs)]` em freerusch-core e freerusch-report; fechar todos os warnings.
- Exemplos executáveis: `crates/freerusch-core/examples/slab_tb450.rs` (biblioteca sem GUI) rodando em CI (`cargo run --example`).
- Documentação de arquitetura em `docs/architecture.md` (diagrama ASCII dos crates + fluxo inputs→coeficientes→momentos→ELS/ELU→relatório).
- Benchmarks leves com criterion OU teste de performance simples behind feature `bench` (interpolação < 1 µs/op); opcional, registrar escolha no PR.

### 5. Release 1.0.0
- CHANGELOG consolidado; version bump 1.0.0 em todos os crates do workspace.
- GitHub Release v1.0.0 com binários ubuntu/windows/macos + checksums SHA-256 publicados.
- Adicionar `SECURITY.md` (política de report) e `CONTRIBUTING.md` (setup, fmt/clippy/test, convenção de branches `feat/*`, `fix/*`).
- Revisar README final: badges de CI, captura/descritivo da GUI, exemplo de biblioteca, seção "Por que FreeRusch?" (open source, pure Rust, família Craft).

## Regras (herdam Fases 1–3)
Pure Rust; não inventar APIs do craft-libs; thiserror; docs ///; fmt/clippy `-D warnings`/test limpos; valores normativos com comentário de origem; NÃO copiar código/dados proprietários de softwares comerciais.

## Critérios de aceite
1. Carga de pneu implementada e integrada às combinações, com testes.
2. Módulo `distribution` unificado (trem-tipo + pneu) sem duplicação.
3. GUI e relatório em pt-BR/en-US alternáveis via config, com teste.
4. `missing_docs` limpo; exemplo roda em CI; docs/architecture.md presente.
5. Tag v1.0.0 publicada com artifacts + checksums; CI verde nas 3 plataformas.
6. Branch `feat/i18n-tire-load-v1`, PR resumindo mudanças normativas e de produto.
