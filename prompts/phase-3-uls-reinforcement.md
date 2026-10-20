# Prompt — Fase 3: Armadura (ELU), combinações, import/export e release

> Pré-requisito: enviar APÓS o merge da Fase 2 (`feat/ruesch-tables-nbr7188-gui`).
> Escopo: dimensionamento de armadura pelo estado limite último, combinações de ações, robustez de projeto e publicação.

## Contexto
FreeRusch em https://github.com/AltamiroAJ/FreeRusch já possui: tabelas de Rüsch completas com interpolação, trens-tipo TB-450/TB-240 (NBR 7188:2013), diagramas de momentos, GUI com abas (Inputs/Results/Report) via craft-ui/egui, memória de cálculo HTML com SVG inline, `craft-settings` para persistência e `.frus` salvo atomicamente via `craft-atomic`. Inspecione a `main` mesclada antes de começar e estenda as interfaces existentes.

## Tarefas

### 1. Dimensionamento de armadura (ELU — NBR 6118:2023) em novo módulo `reinforcement` de `freerusch-core`
- Propriedades de concreto parametrizadas: `fck` (25–50 MPa), `Ecs` estimado pela norma, γc = 2,1; aço CA-50 (fyk = 500 MPa, γs = 1,15) com CA-25/CA-60 selecionáveis.
- Função `fn required_as(m_sd: f64, b: f64, d: f64, concrete: Concrete, steel: Steel) -> Result<Reinforcement, CoreError>`:
  - braço de alavanca z simplificado (z ≤ 0,9d), resolução da equação quadrática clássica de seção retangular;
  - verificação de x/d limites (seção subarmada) e erro claro quando excede;
  - As,mín conforme NBR 6118 (mínimo de flexão + controle de fissuração para lajes de ponte: usar μ mínimo/máximo documentado).
- Espaçamento de barras: `fn bar_spacing(as_m2_per_m, bar_diameter) -> Result<f64, _>`; seleção automática do menor diâmetro (10/12,5/16/20 mm) que atende As e espaçamento ≥ 2 cm e ≤ 20 cm (ou h, o menor) conforme norma.
- Cobertura mínima configurável (padrão 2,5 cm para ambiente agressivo de ponte, CC1/CC2 conforme tabela da norma).
- Testes: caso manual conferido (viga/laje conhecida), seção superarmada → erro, As,mín governando para momentos pequenos, round-trip de unidades (kN·m/cm² ↔ MPa).

### 2. Combinações de ações e fatores de majoração
- Enumeração de combinações ULSS (normais e especiais) com γ_f documentados: carga permanente (peso próprio com γ_c = 25 kN/m³ opcional automático vs. manual), sobrecarga rodoviária (trem-tipo × carga uniforme 20 kN/m² — usar a **mais desfavorável** por ponto, conforme NBR 7188 item de combinações), impacto local (pneus 250 kN em área reduzida — Fase 4 se necessário, deixar TODO rastreado).
- API: `fn design_moments(slab: &SlabDefinition, combo: Combination) -> Result<SlabMoments, CoreError>` devolvendo M_sd por ponto de cálculo.
- Relatório passa a listar M_k (característico) e M_sd (de cálculo) lado a lado.

### 3. Verificações de serviço (opcional, mas desejável)
- Flecha imediata aproximada pelo método de Branson adaptado a laje (coeficientes de Rüsch para carga de serviço) com alerta visual na GUI quando > L/250 (limite típico para pontes; documentar hipótese).
- Fissuração: σ_s com armadura real vs. limite de 300 MPa (controle de fissuração NBR 6118 Tabela p/ ambientes agressivos).

### 4. Arquivo de projeto `.frus` v2
- Migrar formato JSON com campo `schema_version`; manter loader retrocompatível para v1 (inputs sem armadura) com teste de migração.
- Validação de entrada com mensagens claras (erros thiserror tipados: vão inválido, fck fora de faixa, etc.).

### 5. GUI
- Nova aba "Reinforcement": escolha de fck, aço, cobertura, espessura h, distância effective d; botão "Dimensionar" mostrando tabela As por ponto, barra selecionada e espaçamento; destaque visual de pontos onde x/d excede limite.
- Menu Exportar: PDF (via craft-print quando disponível) e HTML; Exportar CSV dos momentos (escrita atômica).

### 6. Release engineering
- CHANGELOG.md (Keep a Changelog) iniciando com 0.2.0 (Fase 2) e 0.3.0 (esta fase).
- Tag `v0.3.0` e GitHub Release com binários das 3 plataformas (adicionar `cargo-dist` OU workflow manual de artifacts no ci.yml — escolher o mais simples que funcione; pure Rust mantido).
- README: capturar screenshot/texto da nova aba, seção "Limitações do método de Rüsch" ampliada com notas de ELU.

## Regras (herdam Fases 1–2)
Pure Rust; não inventar APIs do craft-libs; thiserror; docs ///; fmt/clippy `-D warnings`/test limpos; NÃO copiar dados proprietários de softwares comerciais (T-Rüsch é referência funcional apenas); valores normativos citados da NBR 6118:2023/NBR 7188:2013 com comentário de origem.

## Critérios de aceite
1. Módulo `reinforcement` com testes unitários conferindo cálculo manual em ≥ 3 casos.
2. Combinações ULS aplicadas; relatório mostra M_k e M_sd e armadura por ponto.
3. `.frus` v2 com migração testada de v1.
4. GUI com aba Reinforcement funcional; export HTML/PDF/CSV OK.
5. CI verde nas 3 plataformas; tag v0.3.0 publicada com artifacts.
6. Branch `feat/uls-reinforcement`, PR descrevendo fórmulas usadas e referências normativas.
