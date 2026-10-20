# Prompt — Fase 2: Domínio de engenharia (Rüsch + NBR 7188), GUI real e relatório

> Pré-requisito: enviar este prompt APÓS o merge do PR #1 (`feat/init-freerusch-workspace`) na `main`.
> Status da base ao gerar este prompt: branch 773ed7c compilando, testes passando, CI e README criados.

## Contexto

O FreeRusch (https://github.com/AltamiroAJ/FreeRusch) tem agora um workspace Cargo funcional com:
- `crates/freerusch-core`: motor de cálculo puro com tabelas de Rüsch **parciais** (esqueleto da Fase 1), interpolação linear e uma função pública de momento fletor com teste unitário;
- `crates/freerusch-report`: geração da memória de cálculo em HTML usando `craft-print`/`craft-atomic`;
- `apps/freerusch`: janela GUI mínima via `craft-ui`/`egui` com título "FreeRusch";
- dependências `craft-libs` fixadas no commit `ae2f796b673f29cf3161f36eb6a8933f0a3fe028`;
- CI em `.github/workflows/ci.yml` (fmt, clippy `-D warnings`, test, build nas 3 plataformas).

Agora você deve evoluir o **domínio de engenharia** e a **experiência de uso**, mantendo todas as convenções da família Craft e as regras da Fase 1 (pure Rust, não inventar APIs do craft-libs, thiserror, docs `///`).

Antes de codificar: inspecione o estado atual da `main` mesclada (`git pull`, leia `crates/freerusch-core/src/lib.rs`, `crates/freerusch-report/src/lib.rs` e `apps/freerusch/src/main.rs`) e estenda as interfaces existentes em vez de recriá-las.

## Tarefas

### 1. Tabelas de Rüsch completas em `freerusch-core`
- Transcrever as tabelas clássicas de Rüsch (Helsper/Kerdahl/Blodner, formato DIN/IBSW) para lajes de ponte **articuladas nas bordas livres** e **engastadas nas bordas livres**, parametrizadas por:
  - razão `k = ly/lx` (vão transversal / vão longitudinal), de 1,0 a ∞ (tratar k ≥ 3 como laje armada só no vão principal);
  - posição do ponto de cálculo (médio do vão, quarto pontos dos vãos, apoio intermediário, borda livre etc.).
- Estruturas sugeridas: `enum BoundaryCondition { Pinned, Fixed }`, `struct RueschTable`, `fn coefficient(k: f64, point: CalculationPoint, bc: BoundaryCondition) -> Result<f64, CoreError>` com interpolação linear em `k` entre entradas vizinhas da tabela.
- Convenção de sinal e unidades documentadas: momentos em termos de `m·lx²` (m = carga por área), coeficientes adimensionais.
- Testes: (a) coeficientes exatos nos nós da tabela; (b) interpolação monotônica e limitada entre nós; (c) caso degenerado k=1 (laje quadrada engastada: m_x = m_y ≈ p·l²/24 no centro, conferindo contra valor de tabela publicado); (d) erro claro para k fora do domínio ou ponto inexistente.

### 2. Trens-tipo TB-450 e TB-240 conforme NBR 7188:2013
- Módulo `nbr7188` com `enum VehicleType { Tb450, Tb240 }` e geometria/pesos oficiais:
  - TB-450: caminhão 450 kN (eixo dianteiro 100 kN + dois eios traseiros de 175 kN, espaçamentos 4,00 m e 1,25 m largura de pista 3,00 m ajustável);
  - TB-240: caminhão 240 kN (100 kN + 2×70 kN, mesmos espaçamentos);
  - sobrecarga uniforme de pista: 20 kN/m² (faixa de 3,00 m) conforme NBR 7188 item de trens-tipo.
- Cálculo de **linha de influência simplificada** para a faixa de roda sobre a laje (abordagem de Rüsch: distribuição de pressão 30°/eixo ou largura de influência efetiva `b_eff`), incluindo:
  - largura efetiva de distribuição `b_min` (mínima) e `b_max` conforme regra da norma para engastamento/borda livre;
  - posicionamento transversal crítico da pista (múltiplas pistas quando `k` pequeno, conforme NBR 7188: número de pistas = floor(largura útil / 3,0 m), mínimo 1);
- API pública: `fn moments_from_traffic(...) -> SlabMoments` somando trem-tipo + carga uniforme, aplicando os coeficientes de Rüsch; documentação clara das hipóteses e limitações (não é FEM; conservadorismo típico do método).
- Testes: carga do eixo 175 kN distribuída sobre `b_eff` confere valor manual; comparação sanity TB-450 > TB-240 no mesmo ponto; erro para geometria impossível (ex.: vão < 2·b_min tratado com fallback documentado).

### 3. Diagramas de momentos
- Em `freerusch-core`, gerar `MomentDiagram { points: Vec<(f64, f64)> }` ao longo de uma faixa de 1 m nas duas direções (x e y), para cada combinação (articulada/engastada, TB-450/TB-240/uniforme), com `fn diagram_m_x(...)` e `fn diagram_m_y(...)`.
- Sem renderização aqui: apenas dados numéricos poligonais prontos para SVG/UI.

### 4. GUI real em `apps/freerusch` (sobre `craft-ui`/`egui`)
- Abas ("Inputs", "Results", "Report") usando o que o `craft-ui` já oferece; se abas/containers específicos não existirem, criar `crates/freerusch-ui` com componentes mínimos sobre `egui` (sem duplicar o craft-ui).
- Formulário de entrada: vão lx, vão ly, condição de borda (combo), tipo de trem (TB-450/TB-240/nenhum), carga uniforme (padrão 20 kN/m², editável), concreto/aço apenas informativos (armadura fica para Fase 3).
- Botão "Calcular" → exibe tabela de momentos por ponto de cálculo e preview dos diagramas (desenho simples com `egui::Painter`, polilinha dos `MomentDiagram`s).
- Persistir último projeto/inputs via `craft-settings` (`config.toml`) e salvar/abrir arquivo de projeto `.frus` (JSON serializado com `serde`, escrito atomicamente com `craft-atomic`).
- Título da janela: "FreeRusch".

### 5. Memória de cálculo aprimorada em `freerusch-report`
- HTML final com: premissas (normas citadas: NBR 7188:2013, método de Rüsch), inputs, coeficientes usados (com fonte da tabela), resultados por ponto, **diagramas embutidos como SVG inline** gerados a partir dos `MomentDiagram`s (função pura `fn diagram_svg(&MomentDiagram) -> String`), data/hora e versão do software.
- Exportação/impressão via `craft-print`; escrita do `.html` via `craft-atomic`.
- Teste: snapshot-like (asserções de conteúdo: presença de "TB-450", valores calculados formatados, `<svg` presente).

### 6. Infra e qualidade
- Atualizar `README.md` (uso da GUI, exemplo de biblioteca do core, limitações do método).
- Manter CI verde nas 3 plataformas; se alguma plataforma quebrar por causa do GUI (ex.: headless), registrar no PR e isolar com feature flag em vez de remover dependência.
- `cargo fmt`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` limpos.

## Regras (herdam a Fase 1)
- Pure Rust; sem C/C++ ou binários externos.
- Não inventar APIs do craft-libs — inspecionar o repo `storytold/craft-libs` no rev fixado e usar a API real.
- Erros com `thiserror` (`CoreError`, `ReportError`); docs `///` em todo item público.
- Valores das tabelas/NBR transcritos de fontes públicas de engenharia (literatura Rüsch/DIN, texto público da NBR 7188); citar origem em comentário de módulo; NÃO copiar código ou dados proprietários do T-Rüsch (referência funcional apenas).

## Critérios de aceite
1. Tabelas de Rüsch completas com interpolação e ≥ 4 testes novos em `freerusch-core`.
2. TB-450/TB-240 + carga uniforme calculando momentos com testes de sanidade.
3. `cargo run -p freerusch` abre a GUI com abas, formulário, botão Calcular e preview de diagramas.
4. Relatório HTML contém premissas, resultados e SVGs dos diagramas; teste em `freerusch-report` cobre isso.
5. CI verde em ubuntu/windows/macos; fmt/clippy/test/build limpos.
6. Branch `feat/ruesch-tables-nbr7188-gui`, commits lógicos, PR descrevendo hipóteses de cálculo e limitações.
