//! Motor de cálculo puro do FreeRusch: lajes de pontes pelo método de Rüsch.
//!
//! Este crate não depende de UI nem de E/S. Ele fornece:
//!
//! - [RuschTable]: tabelas de Rüsch embutidas (coeficientes adimensionais `alpha`
//!   para momentos positivos e `beta` para momentos negativos em vãos contínuos),
//!   indexadas por razão `l2/l1` e condição de borda;
//! - interpolação bilinear dos coeficientes para razões intermediárias
//!   ([`RuschTable::coefficients_at`]);
//! - trens-tipo da NBR 7188:2013 ([`TrainType`], [`load_per_lane_knm`]) com a
//!   largura-mínima efetiva `b_min = b + 3,00 m` e o fator de distribuição
//!   transversal `θ = b_min / n · λ_v`;
//! - cálculo do momento fletor de projeto a partir dos coeficientes
//!   ([`design_moment`] / [`bending_moment`]) e geração de envelopes/diagramas
//!   discretizados ([`moment_diagram`]).
//!
//! # Exemplo
//!
//! ```
//! use freerusch_core::{EdgeCondition, RuschTable, TrainType, bending_moment};
//!
//! // Laje biapoiada de vão l1 = 8,00 m, concreto c25/20, trem-tipo TB-450.
//! let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
//! let m = bending_moment(&table, &TrainType::TB450, 8.0, 0.0, 25.0)
//!     .expect("cálculo válido");
//! assert!(m.max_positive > 0.0);
//! ```

use std::sync::OnceLock;

use serde::Deserialize;
use thiserror::Error;

/// Natureza de um coeficiente numa tabela de Rüsch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuschColumnKind {
    /// Coeficiente `L`: parcela do trem-tipo, em função de `t/a`.
    Wheel,
    /// Coeficiente `p`/`p'`: parcela da carga uniforme, válido para todo `t/a`.
    Uniform,
}

/// Uma coluna de coeficientes de uma tabela de Rüsch.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RuschColumn {
    /// Rótulo do grupo, preservado cru da planilha (ex.: `"Mxm in Plattenmitte"`).
    pub group: String,
    /// Letra do coeficiente: `"L"`, `"p"` ou `"p'"`.
    pub letter: String,
    /// Relação `t/a` da coluna; `None` quando vale para todo `t/a`.
    pub ta: Option<f64>,
    /// Natureza do coeficiente.
    pub kind: RuschColumnKind,
    /// Valores ao longo de [`RuschTableData::rows`].
    ///
    /// `None` representa uma ausência: a célula `-` da planilha (coeficiente não
    /// se aplica) ou uma célula que a fonte traz como erro/typo. Nunca é zero
    /// inventado — as ausências anômalas estão listadas em `warnings` no JSON.
    pub values: Vec<Option<f64>>,
}

/// Coeficiente fixo do painel para carga uniforme em toda a laje.
///
/// Vem do cabeçalho da tabela (ex.: `Mxm = 0,063`) e vale para qualquer carga
/// uniformemente distribuída: `M = k · q · lx²`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PanelCoefficient {
    /// Nome da resultante (`"Mxm"`, `"Mye"`, ...).
    pub name: String,
    /// Coeficiente adimensional.
    pub value: f64,
}

/// Uma tabela de Rüsch extraída do workbook de referência do IME.
///
/// Os coeficientes são dados numéricos crus, sem interpretação de sinal: os
/// prefixos dos rótulos (`¾`, `(+)`, `-`, `_`) ainda não foram confirmados
/// contra a fonte original e por isso são preservados como texto.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RuschTableData {
    /// Nome da planilha de origem (ex.: `"Nr 65.1"`).
    pub sheet: String,
    /// Número da tabela.
    pub number: u32,
    /// Variante da tabela (o `.1`/`.2` do nome), quando existir.
    pub variant: Option<u8>,
    /// Parâmetro do cabeçalho: `"ly/lx"` ou `"l/a"`.
    pub param: Option<String>,
    /// Valor do parâmetro como está no cabeçalho (ex.: `"2,25"`, `"∞"`).
    pub ratio_raw: Option<String>,
    /// Valor numérico do parâmetro; `None` para o infinito ou quando ausente.
    pub ratio: Option<f64>,
    /// `true` quando o parâmetro é infinito.
    pub ratio_is_infinite: bool,
    /// Eixo das linhas (`lx/a`).
    pub rows: Vec<f64>,
    /// Colunas de coeficientes.
    pub columns: Vec<RuschColumn>,
    /// Coeficientes da carga uniforme em todo o painel (cabeçalho da tabela).
    pub panel_coefficients: Vec<PanelCoefficient>,
    /// Classe de ponte declarada no cabeçalho (ex.: `"Brückenklasse 30t bis 60t"`).
    pub bridge_class: Option<String>,
    /// Nota do cabeçalho sobre a carga uniforme de referência
    /// (ex.: `"Gleichlast um SLW von 1 t.m2"`).
    pub uniform_reference: Option<String>,
}

#[derive(Deserialize)]
struct RuschTablesPayload {
    tables: Vec<RuschTableData>,
}

/// Todas as tabelas de Rüsch embarcadas, na ordem do arquivo de dados.
///
/// O acervo vem de `data/rusch_tables.json`, embutido no binário em tempo de
/// compilação e desserializado uma única vez, na primeira chamada.
///
/// # Panics
///
/// Se o JSON embutido não desserializar. Isso seria um defeito de build: o
/// arquivo é gerado por `tools/extract_rusch_tables.py` e validado por testes.
#[must_use]
pub fn rusch_tables() -> &'static [RuschTableData] {
    static TABLES: OnceLock<Vec<RuschTableData>> = OnceLock::new();
    TABLES.get_or_init(|| {
        let payload: RuschTablesPayload =
            serde_json::from_str(include_str!("../data/rusch_tables.json"))
                .expect("tabelas de Rüsch embutidas devem desserializar");
        payload.tables
    })
}

impl RuschTableData {
    /// Localiza uma tabela pelo número e variante.
    #[must_use]
    pub fn find(number: u32, variant: Option<u8>) -> Option<&'static Self> {
        rusch_tables()
            .iter()
            .find(|t| t.number == number && t.variant == variant)
    }

    /// Índice da linha cujo `lx/a` é exatamente o valor pedido.
    #[must_use]
    pub fn row_index(&self, lx_over_a: f64) -> Option<usize> {
        self.rows.iter().position(|r| (r - lx_over_a).abs() < 1e-9)
    }

    /// Primeira coluna que casa com grupo e letra.
    #[must_use]
    pub fn column(&self, group: &str, letter: &str) -> Option<&RuschColumn> {
        self.columns
            .iter()
            .find(|c| c.group == group && c.letter == letter)
    }
}

/// Momentos fletores por metro devidos a uma carga uniforme `q` em **todo** o
/// painel.
///
/// Usa os coeficientes fixos do cabeçalho da tabela: `M = k · q · lx²`. É o
/// caminho validado contra o exemplo resolvido do workbook (aba `Main`): tabela
/// Nr 65.1, `lx = 7,00 m`, `q = 10,5209 kN/m²` reproduz os seis momentos de
/// carga permanente célula a célula.
///
/// Retorna pares `(resultante, momento)` na ordem da tabela. O sinal segue a
/// tabela (positivo); a convenção de face tracionada é do chamador.
///
/// # Erros
///
/// [`RuschError::InvalidDimension`] se `q_kn_m2` ou `lx_m` não forem finitos e
/// positivos.
pub fn uniform_load_moments(
    table: &RuschTableData,
    q_kn_m2: f64,
    lx_m: f64,
) -> Result<Vec<(String, f64)>, RuschError> {
    if !(q_kn_m2.is_finite() && q_kn_m2 > 0.0) {
        return Err(RuschError::InvalidDimension {
            name: "q_kn_m2",
            value: q_kn_m2,
        });
    }
    if !(lx_m.is_finite() && lx_m > 0.0) {
        return Err(RuschError::InvalidDimension {
            name: "lx_m",
            value: lx_m,
        });
    }
    let factor = q_kn_m2 * lx_m * lx_m;
    Ok(table
        .panel_coefficients
        .iter()
        .map(|k| (k.name.clone(), k.value * factor))
        .collect())
}

/// Erros do motor de cálculo Rüsch.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RuschError {
    /// Razão de vãos `l2/l1` fora do intervalo da tabela `[r_min, r_max]`.
    #[error("razão l2/l1 = {ratio:.3} fora da tabela [{r_min:.3}, {r_max:.3}]")]
    RatioOutOfRange {
        /// Razão solicitada.
        ratio: f64,
        /// Menor razão coberta pela tabela.
        r_min: f64,
        /// Maior razão coberta pela tabela.
        r_max: f64,
    },
    /// Dimensão nula, negativa ou não finita.
    #[error("dimensão inválida: {name} = {value:?} (exige valor finito > 0)")]
    InvalidDimension {
        /// Nome da grandeza (ex.: `"l1"`).
        name: &'static str,
        /// Valor recebido.
        value: f64,
    },
    /// O concreto informado não existe na lista de resistências conhecidas.
    #[error("classe de concreto desconhecida: \"{name}\"")]
    UnknownConcreteClass {
        /// Nome recebido.
        name: String,
    },
}

/// Condição de contorno do painel contínuo (apoios extremos).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeCondition {
    /// Painel de extremidade biapoiado nas bordas externas (laje contínua).
    SimplySupported,
    /// Painel de extremidade com borda externa engastada (continuidade total).
    Clamped,
}

/// Um ponto `(l2/l1, alfa)` ou `(l2/l1, beta)` de uma linha da tabela de Rüsch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuschPoint {
    /// Razão de vãos `r = l2/l1`.
    pub ratio: f64,
    /// Coeficiente adimensional associado à razão.
    pub coeff: f64,
}

/// Linha da tabela para um número fixo de vãos.
#[derive(Debug, Clone, PartialEq)]
pub struct RuschRow {
    /// Número de vãos do tabuleiro contínuo.
    pub spans: usize,
    /// Pontos de `alfa` (momentos positivos), ordenados por `ratio` crescente.
    pub alpha: Vec<RuschPoint>,
    /// Pontos de `beta` (momentos negativos nos apoios internos), ordenados
    /// por `ratio` crescente. Vazio quando a configuração não possui apoio
    /// interno (painel isolado).
    pub beta: Vec<RuschPoint>,
}

/// Tabela de Rüsch: coeficientes adimensionais `M = α · p · l₁²` (vão positivo)
/// e `M = β · p · l₁²` (apoio interno negativo).
#[derive(Debug, Clone, PartialEq)]
pub struct RuschTable {
    /// Condição de contorno dos apoios externos.
    pub edge: EdgeCondition,
    /// Linhas, uma por número de vãos.
    pub rows: Vec<RuschRow>,
}

impl RuschTable {
    /// Tabela embutida para a condição de contorno escolhida.
    pub fn for_edges(edge: EdgeCondition) -> Self {
        match edge {
            EdgeCondition::SimplySupported => simply_supported_table(),
            EdgeCondition::Clamped => clamped_table(),
        }
    }

    /// Encontra a linha com o número de vãos pedido.
    pub fn row(&self, spans: usize) -> Option<&RuschRow> {
        self.rows.iter().find(|r| r.spans == spans)
    }

    /// Interpola os coeficientes `alfa`/`beta` na razão `ratio` para o número
    /// de vãos pedido.
    ///
    /// Retorna `(alfa, beta)`; `beta` é `None` quando a configuração não tem
    /// apoio interno. Erro [`RuschError::RatioOutOfRange`] se `ratio` ultrapassar
    /// o intervalo da linha em mais de `1e-9`.
    pub fn coefficients_at(
        &self,
        spans: usize,
        ratio: f64,
    ) -> Result<(f64, Option<f64>), RuschError> {
        let row = self.row(spans).ok_or(RuschError::InvalidDimension {
            name: "spans",
            value: spans as f64,
        })?;
        let alpha = interpolate_row(&row.alpha, ratio)?;
        let beta = if row.beta.is_empty() {
            None
        } else {
            Some(interpolate_row(&row.beta, ratio)?)
        };
        Ok((alpha, beta))
    }
}

/// Sinaliza que os coeficientes embutidos ainda **não** são as tabelas
/// homologadas de Rüsch.
///
/// A base atual é didática e coerente com a formulação clássica (`alfa → 1/8`
/// no painel isolado, `|beta| → 1/12` no limite igualmente contínuo), mas as
/// tabelas completas (Helsper/Kerdahl/Blodner, formato DIN/IBSW) ainda não
/// foram transcritas de uma fonte verificada. Enquanto este valor for `true`,
/// a GUI e a memória de cálculo devem exibir o aviso de resultados não
/// homologados.
pub const RUSCH_TABLES_PRELIMINARY: bool = true;

/// Tabela simplificada para painéis com apoios externos biapoiados.
///
/// Os valores são uma base didática coerente com a formulação clássica de
/// Rüsch: `alfa → 1/8` no painel isolado (`r = 0`) e cresce com a continuidade;
/// `|beta| → 1/12` no limite igualmente contínuo. Tabelas completas homologadas
/// serão incorporadas em PRs seguintes mantendo esta interface.
fn simply_supported_table() -> RuschTable {
    let ratios: [f64; 6] = [0.5, 0.6, 0.8, 1.0, 1.2, 1.4];
    let mk = |pts: &[f64]| -> Vec<RuschPoint> {
        ratios
            .iter()
            .zip(pts)
            .map(|(&x, &y)| RuschPoint { ratio: x, coeff: y })
            .collect()
    };
    RuschTable {
        edge: EdgeCondition::SimplySupported,
        rows: vec![
            RuschRow {
                spans: 1,
                alpha: mk(&[0.1250, 0.1250, 0.1250, 0.1250, 0.1250, 0.1250]),
                beta: Vec::new(),
            },
            RuschRow {
                spans: 2,
                alpha: mk(&[0.1020, 0.1060, 0.1140, 0.1250, 0.1210, 0.1180]),
                beta: mk(&[-0.0580, -0.0640, -0.0760, -0.0833, -0.0790, -0.0750]),
            },
            RuschRow {
                spans: 3,
                alpha: mk(&[0.0960, 0.1000, 0.1080, 0.1200, 0.1160, 0.1130]),
                beta: mk(&[-0.0620, -0.0680, -0.0780, -0.0860, -0.0820, -0.0780]),
            },
            RuschRow {
                spans: 4,
                alpha: mk(&[0.0940, 0.0980, 0.1060, 0.1180, 0.1140, 0.1110]),
                beta: mk(&[-0.0640, -0.0700, -0.0800, -0.0880, -0.0840, -0.0800]),
            },
        ],
    }
}

/// Tabela simplificada para painéis com borda externa engastada.
fn clamped_table() -> RuschTable {
    let ratios: [f64; 6] = [0.5, 0.6, 0.8, 1.0, 1.2, 1.4];
    let mk = |pts: &[f64]| -> Vec<RuschPoint> {
        ratios
            .iter()
            .zip(pts)
            .map(|(&x, &y)| RuschPoint { ratio: x, coeff: y })
            .collect()
    };
    RuschTable {
        edge: EdgeCondition::Clamped,
        rows: vec![
            RuschRow {
                spans: 1,
                alpha: mk(&[0.0833, 0.0833, 0.0833, 0.0833, 0.0833, 0.0833]),
                beta: mk(&[-0.0833, -0.0833, -0.0833, -0.0833, -0.0833, -0.0833]),
            },
            RuschRow {
                spans: 2,
                alpha: mk(&[0.0780, 0.0820, 0.0900, 0.1000, 0.0960, 0.0930]),
                beta: mk(&[-0.0900, -0.0960, -0.1040, -0.1110, -0.1070, -0.1030]),
            },
            RuschRow {
                spans: 3,
                alpha: mk(&[0.0750, 0.0790, 0.0870, 0.0970, 0.0930, 0.0900]),
                beta: mk(&[-0.0940, -0.1000, -0.1080, -0.1160, -0.1120, -0.1080]),
            },
        ],
    }
}

/// Interpola linearmente uma linha da tabela na razão `ratio`, com extrapolação
/// linear limitada ao primeiro/último segmento quando `ratio` está fora da grade
/// mas dentro de tolerância mínima; erro caso contrário.
fn interpolate_row(points: &[RuschPoint], ratio: f64) -> Result<f64, RuschError> {
    if points.is_empty() || !ratio.is_finite() {
        return Err(RuschError::InvalidDimension {
            name: "ratio",
            value: ratio,
        });
    }
    let r_min = points.first().map(|p| p.ratio).unwrap_or(0.0);
    let r_max = points.last().map(|p| p.ratio).unwrap_or(0.0);
    const EPS: f64 = 1e-9;
    if ratio < r_min - EPS || ratio > r_max + EPS {
        return Err(RuschError::RatioOutOfRange {
            ratio,
            r_min,
            r_max,
        });
    }
    let ratio = ratio.clamp(r_min, r_max);
    // Busca o segmento que contém `ratio`.
    for window in points.windows(2) {
        let (a, b) = (window[0], window[1]);
        if ratio <= b.ratio + EPS {
            let span = b.ratio - a.ratio;
            if span.abs() < f64::EPSILON {
                return Ok(a.coeff);
            }
            let t = (ratio - a.ratio) / span;
            return Ok(a.coeff + t * (b.coeff - a.coeff));
        }
    }
    Ok(points.last().map(|p| p.coeff).unwrap_or(0.0))
}

/// Trem-tipo da NBR 7188:2013 (ABNT NBR 7188:2013, tabela de trens-tipo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainType {
    /// TB-450: carga uniformemente distribuída de 4,5 kN/m² por faixa de 3,0 m.
    TB450,
    /// TB-240: carga uniformemente distribuída de 2,4 kN/m² por faixa de 3,0 m.
    TB240,
}

impl TrainType {
    /// Intensidade da carga uniformemente distribuída `p` do trem-tipo (kN/m²).
    ///
    /// NBR 7188:2013 — carga de multidão na pista, atuando em conjunto com o
    /// veículo-tipo na região externa a ele: **5,0 kN/m²** para o TB-450 e
    /// **4,0 kN/m²** para o TB-240.
    pub fn ud_load_knm2(self) -> f64 {
        match self {
            TrainType::TB450 => 5.0,
            TrainType::TB240 => 4.0,
        }
    }

    /// Largura da faixa carregada do trem-tipo (m).
    pub fn lane_width_m(self) -> f64 {
        3.0
    }

    /// Identificador normalizado ("TB-450" / "TB-240").
    pub fn label(self) -> &'static str {
        match self {
            TrainType::TB450 => "TB-450",
            TrainType::TB240 => "TB-240",
        }
    }

    /// Geometria do veículo conforme a NBR 7188:2013.
    ///
    /// Os dois trens-tipo usam o mesmo chassi: três eixos (um dianteiro e dois
    /// traseiros), 4,00 m do dianteiro ao primeiro traseiro, 1,25 m entre os
    /// traseiros e 3,00 m de faixa por veículo. O que muda é a carga por eixo:
    /// TB-450 → 100 + 175 + 175 kN (450 kN); TB-240 → 100 + 70 + 70 kN (240 kN).
    pub fn geometry(self) -> VehicleGeometry {
        VehicleGeometry {
            axles_kn: match self {
                TrainType::TB450 => vec![100.0, 175.0, 175.0],
                TrainType::TB240 => vec![100.0, 70.0, 70.0],
            },
            front_to_rear_m: 4.0,
            rear_spacing_m: 1.25,
            lane_width_m: self.lane_width_m(),
        }
    }
}

/// Geometria do veículo de um trem-tipo da NBR 7188:2013.
///
/// Descreve a disposição dos eixos e a faixa ocupada, sem pressupor como as
/// cargas se distribuem na laje: essa etapa (largura efetiva `b_min`/`b_max` e
/// fator de distribuição transversal) é aplicada por quem consome a geometria.
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleGeometry {
    /// Cargas por eixo (kN), na ordem dianteiro → traseiros.
    pub axles_kn: Vec<f64>,
    /// Distância entre o eixo dianteiro e o primeiro eixo traseiro (m).
    pub front_to_rear_m: f64,
    /// Distância entre os dois eixos traseiros (m).
    pub rear_spacing_m: f64,
    /// Largura da faixa ocupada por um veículo (m).
    pub lane_width_m: f64,
}

impl VehicleGeometry {
    /// Carga total do veículo (kN), soma das cargas por eixo.
    pub fn total_kn(&self) -> f64 {
        self.axles_kn.iter().sum()
    }

    /// Número de faixas que cabem na largura útil do tabuleiro.
    ///
    /// Regra da NBR 7188: `floor(b / 3,00)`, no mínimo uma faixa. Larguras
    /// inválidas (não finitas ou não positivas) também resultam em uma faixa,
    /// em vez de zero faixas carregadas.
    pub fn lane_count(&self, deck_width_m: f64) -> usize {
        if !deck_width_m.is_finite() || deck_width_m <= 0.0 {
            return 1;
        }
        let lanes = (deck_width_m / self.lane_width_m).floor();
        if lanes < 1.0 { 1 } else { lanes as usize }
    }
}

/// Carga total por faixa de rodagem (kN/m) sobre a laje, já considerando a
/// largura-mínima efetiva `b_min = b + 3,00 m` (NBR 7188:2013) distribuída em
/// `n_faixas` faixas.
///
/// A regra clássica de largura-mínima efetiva diz que cada faixa carrega como
/// se tivesse `lane_width + 3,00 m` de influência sobre a laje; o resultado é
/// a intensidade média `p_eff` aplicada ao vão (kN/m por metro de vão).
pub fn load_per_lane_knm(train: TrainType, deck_width_m: f64, n_lanes: usize) -> f64 {
    let p_ud = train.ud_load_knm2();
    let w_lane = train.lane_width_m();
    let effective = (deck_width_m).max(w_lane);
    let n = n_lanes.max(1) as f64;
    // Área efetiva por faixa: b_min/n × vão unitário; com b_min = b + 3,00 m
    // limitado pela largura física do tabuleiro.
    let b_min = (effective + 3.0).min(effective * 2.0);
    p_ud * (w_lane.min(b_min / n))
}

/// Resultado do cálculo de esforços de um painel de laje pelo método de Rüsch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MomentResult {
    /// Momento fletor positivo máximo no vão `l1` (kN·m/m).
    pub max_positive: f64,
    /// Momento fletor negativo máximo nos apoios internos (kN·m/m); `0.0` quando
    /// o painel não possui continuidade interna.
    pub max_negative: f64,
    /// Coeficiente `alfa` interpolado utilizado.
    pub alpha: f64,
    /// Coeficiente `beta` interpolado utilizado; `None` sem apoio interno.
    pub beta: Option<f64>,
}

/// Calcula o momento fletor de projeto (por metro de largura) a partir de uma
/// tabela de Rüsch, de um trem-tipo e das dimensões do painel.
///
/// - `table`: tabela de coeficientes conforme a condição de borda;
/// - `train`: trem-tipo NBR 7188;
/// - `l1`: vão considerado (m), deve ser > 0;
/// - `l2`: vão vizinho (m); `0.0` representa painel isolado (`r = 0` ⇒ usa-se
///   o menor ponto da grade da tabela);
/// - `deck_width_m`: largura transversal do tabuleiro (m);
/// - `concrete_class_strength_mpa`: resistência característica `f_ck` (MPa);
///   usada apenas como validação de entrada neste esqueleto.
pub fn design_moment(
    table: &RuschTable,
    train: TrainType,
    spans: usize,
    l1: f64,
    l2: f64,
    deck_width_m: f64,
    concrete_class_strength_mpa: f64,
) -> Result<MomentResult, RuschError> {
    if !(l1.is_finite() && l1 > 0.0) {
        return Err(RuschError::InvalidDimension {
            name: "l1",
            value: l1,
        });
    }
    if !(deck_width_m.is_finite() && deck_width_m > 0.0) {
        return Err(RuschError::InvalidDimension {
            name: "deck_width_m",
            value: deck_width_m,
        });
    }
    if !(concrete_class_strength_mpa.is_finite() && concrete_class_strength_mpa > 0.0) {
        return Err(RuschError::UnknownConcreteClass {
            name: format!("{concrete_class_strength_mpa}"),
        });
    }
    let ratio = if l2 == 0.0 { 0.0 } else { l2 / l1 };
    // Para painel isolado usamos o menor ponto da grade (alfa mínimo clássico).
    let ratio_for_lookup = if l2 == 0.0 { 0.5 } else { ratio };
    if ratio_for_lookup.is_nan() || ratio_for_lookup < 0.0 {
        return Err(RuschError::InvalidDimension {
            name: "l2",
            value: l2,
        });
    }
    let (alpha, beta) = table.coefficients_at(spans, ratio_for_lookup)?;
    let n_lanes = ((deck_width_m / train.lane_width_m()).floor().max(1.0)) as usize;
    let p_line = load_per_lane_knm(train, deck_width_m, n_lanes);
    let per_meter = p_line / deck_width_m;
    let max_positive = alpha * per_meter * l1 * l1;
    let max_negative = beta.unwrap_or(0.0) * per_meter * l1 * l1;
    Ok(MomentResult {
        max_positive,
        max_negative,
        alpha,
        beta,
    })
}

/// Atalho para o caso clássico de painel biapoiado de vão único (um só vão,
/// razão `r = 0.5` ⇒ `alfa = 1/8`), usado no exemplo do crate.
pub fn bending_moment(
    table: &RuschTable,
    train: &TrainType,
    l1: f64,
    _l2_over_l1: f64,
    concrete_class_strength_mpa: f64,
) -> Result<MomentResult, RuschError> {
    // Largura típica de tabuleiro de 2 faixas (≈ 6 m) para o exemplo mínimo.
    design_moment(table, *train, 1, l1, 0.0, 6.0, concrete_class_strength_mpa)
}

/// Diagrama/envelope de momentos fletores discretizado ao longo do vão `l1`.
///
/// Distribui o momento positivo no meio do vão e os negativos nas extremidades
/// (parábola elementar coerente com a hipótese de Rüsch para lajes maciças),
/// retornando pares `(x, M)` com `x ∈ [0, l1]` em `samples` pontos.
pub fn moment_diagram(result: &MomentResult, l1: f64, samples: usize) -> Vec<(f64, f64)> {
    let samples = samples.max(2);
    let mut out = Vec::with_capacity(samples);
    let m_plus = result.max_positive;
    let m_minus = result.max_negative;
    for i in 0..samples {
        let x = l1 * (i as f64 / (samples - 1) as f64);
        let u = x / l1; // 0..1
        let m = if u <= 0.5 {
            let s = u / 0.5;
            m_minus + (m_plus - m_minus) * s * s
        } else {
            let s = (1.0 - u) / 0.5;
            m_minus + (m_plus - m_minus) * s * s
        };
        out.push((x, m));
    }
    out
}

/// Diagrama de momentos fletores discretizado ao longo de um vão.
///
/// É apenas dado numérico: a renderização (GUI com [`egui`] ou SVG na memória
/// de cálculo) é responsabilidade das camadas de apresentação. Os pontos vêm
/// ordenados por `x` crescente.
#[derive(Debug, Clone, PartialEq)]
pub struct MomentDiagram {
    /// Pares `(x, M)` com `x` em metros e `M` em kN·m/m.
    pub points: Vec<(f64, f64)>,
}

impl MomentDiagram {
    /// Constrói o diagrama de um [`MomentResult`] ao longo do vão `l1`, com
    /// `samples` pontos (mínimo de 2).
    pub fn for_result(result: &MomentResult, l1: f64, samples: usize) -> Self {
        MomentDiagram {
            points: moment_diagram(result, l1, samples),
        }
    }

    /// Maior momento positivo do diagrama (kN·m/m); `0.0` se vazio.
    pub fn max_moment(&self) -> f64 {
        self.points
            .iter()
            .map(|&(_, m)| m)
            .reduce(f64::max)
            .unwrap_or(0.0)
    }

    /// Menor momento (tipicamente negativo nos apoios, kN·m/m); `0.0` se vazio.
    pub fn min_moment(&self) -> f64 {
        self.points
            .iter()
            .map(|&(_, m)| m)
            .reduce(f64::min)
            .unwrap_or(0.0)
    }

    /// Maior valor absoluto de momento, usado para escalar o desenho; `0.0` se
    /// vazio. É o valor que define a amplitude vertical do diagrama.
    pub fn max_abs_moment(&self) -> f64 {
        self.points
            .iter()
            .map(|&(_, m)| m.abs())
            .reduce(f64::max)
            .unwrap_or(0.0)
    }
}

/// Classes de concreto usuais da NBR 6118 e suas resistências `f_ck` (MPa).
pub fn concrete_characteristic_strength(class_name: &str) -> Result<f64, RuschError> {
    let map = [
        ("C20/25", 20.0),
        ("C25/30", 25.0),
        ("C30/37", 30.0),
        ("C35/45", 35.0),
        ("C40/50", 40.0),
        ("C45/55", 45.0),
        ("C50/60", 50.0),
    ];
    map.iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(class_name.trim()))
        .map(|(_, v)| *v)
        .ok_or_else(|| RuschError::UnknownConcreteClass {
            name: class_name.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_alpha_between_grid_points() {
        let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
        let (a_10, _) = table.coefficients_at(2, 1.0).expect("r=1.0 na grade");
        let (a_09, _) = table.coefficients_at(2, 0.9).expect("interpolação");
        let (a_08, _) = table.coefficients_at(2, 0.8).expect("r=0.8 na grade");
        assert!((a_10 - 0.1250).abs() < 1e-9);
        assert!((a_09 - (a_08 + a_10) / 2.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_out_of_range_ratio() {
        let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
        let err = table.coefficients_at(2, 2.0).unwrap_err();
        assert!(matches!(err, RuschError::RatioOutOfRange { .. }));
    }

    #[test]
    fn uniform_load_matches_nbr7188() {
        // NBR 7188:2013 — carga de multidão na pista, atuando em conjunto com o
        // trem-tipo na região externa ao veículo.
        assert!(
            (TrainType::TB450.ud_load_knm2() - 5.0).abs() < 1e-9,
            "TB-450: 5,0 kN/m²"
        );
        assert!(
            (TrainType::TB240.ud_load_knm2() - 4.0).abs() < 1e-9,
            "TB-240: 4,0 kN/m²"
        );
    }

    #[test]
    fn bending_moment_is_alpha_p_l2() {
        // Painel isolado: alfa = 1/8; p_eff = 5,0 kN/m² × min(3,0; b_min/n)/b.
        // Com b = 6,0 m e 2 faixas: p_linha = 5,0 × 3,0 = 15,0 kN/m; por metro
        // de largura: 15,0 / 6,0 = 2,50 kN/m².
        let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
        let m = bending_moment(&table, &TrainType::TB450, 8.0, 0.0, 25.0).expect("cálculo");
        let expected = 0.125 * 2.5 * 8.0 * 8.0; // = 20,0 kN·m/m
        assert!((m.max_positive - expected).abs() < 1e-9);
        assert_eq!(m.max_negative, 0.0);
    }

    #[test]
    fn continuous_span_has_negative_moment() {
        let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
        let m = design_moment(&table, TrainType::TB450, 2, 8.0, 8.0, 6.0, 25.0).expect("cálculo");
        assert!(m.max_negative < 0.0);
        assert!(m.max_positive > 0.0);
    }

    #[test]
    fn invalid_inputs_error() {
        let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
        assert!(matches!(
            design_moment(&table, TrainType::TB450, 2, -1.0, 8.0, 6.0, 25.0),
            Err(RuschError::InvalidDimension { name: "l1", .. })
        ));
        assert!(matches!(
            concrete_characteristic_strength("C99/99"),
            Err(RuschError::UnknownConcreteClass { .. })
        ));
    }

    #[test]
    fn diagram_endpoints_match_result() {
        let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
        let m = design_moment(&table, TrainType::TB240, 2, 10.0, 10.0, 6.0, 25.0).expect("cálculo");
        let d = moment_diagram(&m, 10.0, 11);
        assert_eq!(d.len(), 11);
        assert!((d[0].1 - m.max_negative).abs() < 1e-9);
        assert!((d[5].1 - m.max_positive).abs() < 1e-9);
        assert!((d[10].1 - m.max_negative).abs() < 1e-9);
    }

    #[test]
    fn diagram_extremes_match_moment_result() {
        let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
        let m = design_moment(&table, TrainType::TB450, 2, 8.0, 8.0, 6.0, 25.0).expect("cálculo");
        let d = MomentDiagram::for_result(&m, 8.0, 21);
        assert_eq!(d.points.len(), 21);
        assert!((d.max_moment() - m.max_positive).abs() < 1e-9);
        assert!((d.min_moment() - m.max_negative).abs() < 1e-9);
        assert!((d.max_abs_moment() - m.max_positive.max(m.max_negative.abs())).abs() < 1e-9);
    }

    #[test]
    fn empty_diagram_reports_zero_extremes() {
        let d = MomentDiagram { points: Vec::new() };
        assert_eq!(d.max_abs_moment(), 0.0);
        assert_eq!(d.min_moment(), 0.0);
        assert_eq!(d.max_moment(), 0.0);
    }

    #[test]
    fn tb450_geometry_matches_nbr7188() {
        let geometry = TrainType::TB450.geometry();
        assert_eq!(geometry.axles_kn, vec![100.0, 175.0, 175.0]);
        assert!((geometry.total_kn() - 450.0).abs() < 1e-9);
        assert!((geometry.front_to_rear_m - 4.0).abs() < 1e-9);
        assert!((geometry.rear_spacing_m - 1.25).abs() < 1e-9);
        assert!((geometry.lane_width_m - 3.0).abs() < 1e-9);
    }

    #[test]
    fn tb240_geometry_matches_nbr7188() {
        let geometry = TrainType::TB240.geometry();
        assert_eq!(geometry.axles_kn, vec![100.0, 70.0, 70.0]);
        assert!((geometry.total_kn() - 240.0).abs() < 1e-9);
        assert!((geometry.front_to_rear_m - 4.0).abs() < 1e-9);
        assert!((geometry.rear_spacing_m - 1.25).abs() < 1e-9);
    }

    #[test]
    fn lane_count_floors_by_lane_width() {
        let geometry = TrainType::TB450.geometry();
        assert_eq!(geometry.lane_count(2.5), 1, "nunca menos de uma faixa");
        assert_eq!(geometry.lane_count(3.0), 1);
        assert_eq!(geometry.lane_count(6.0), 2);
        assert_eq!(geometry.lane_count(8.9), 2);
        assert_eq!(geometry.lane_count(9.0), 3);
    }

    #[test]
    fn lane_count_tolerates_invalid_width() {
        let geometry = TrainType::TB450.geometry();
        assert_eq!(geometry.lane_count(f64::NAN), 1);
        assert_eq!(geometry.lane_count(-4.0), 1);
        assert_eq!(geometry.lane_count(0.0), 1);
    }

    #[test]
    fn rusch_data_covers_the_whole_workbook() {
        // O workbook do IME tem 186 planilhas de tabela; todas são extraídas.
        let tables = rusch_tables();
        assert_eq!(tables.len(), 186);
        for table in tables {
            assert_eq!(table.rows.len(), 13, "{}", table.sheet);
            assert!(!table.columns.is_empty(), "{}", table.sheet);
            for column in &table.columns {
                assert_eq!(
                    column.values.len(),
                    table.rows.len(),
                    "{} / {}",
                    table.sheet,
                    column.group
                );
            }
        }
    }

    #[test]
    fn rusch_table_index_is_unique() {
        // (number, variant) é a chave pública de busca; precisa ser única.
        let mut seen = std::collections::HashSet::new();
        for table in rusch_tables() {
            assert!(
                seen.insert((table.number, table.variant)),
                "identidade duplicada em {}",
                table.sheet
            );
        }
    }

    #[test]
    fn rusch_data_matches_spreadsheet_cells() {
        // Planilha "Nr 1.1", coluna "Mxm in Plattenmitte" com t/a = 0,125:
        // C12 = 0,185 (lx/a = 0,5) e C24 = 1,25 (lx/a = 10).
        let table = RuschTableData::find(1, Some(1)).expect("tabela Nr 1.1");
        assert_eq!(table.row_index(0.5), Some(0));
        assert_eq!(table.row_index(10.0), Some(12));

        let column = table
            .column("Mxm in Plattenmitte", "L")
            .expect("coluna de Mxm");
        assert_eq!(column.ta, Some(0.125));
        assert_eq!(column.kind, RuschColumnKind::Wheel);
        assert_eq!(column.values[0], Some(0.185));
        assert_eq!(column.values[12], Some(1.25));
    }

    #[test]
    fn rusch_dash_cells_are_absent_not_zero() {
        // Planilha "Nr 1.1": J12 = "-" (coeficiente p de Mxm em lx/a = 0,5).
        let table = RuschTableData::find(1, Some(1)).expect("tabela Nr 1.1");
        let p = table
            .columns
            .iter()
            .find(|c| c.group == "Mxm" && c.letter == "p")
            .expect("coluna p de Mxm");
        assert_eq!(p.values[0], None);
        assert_ne!(p.values[0], Some(0.0));
    }

    #[test]
    fn infinite_ratio_is_recorded_as_such() {
        let table = RuschTableData::find(1, Some(1)).expect("tabela Nr 1.1");
        assert!(table.ratio_is_infinite);
        assert_eq!(table.ratio_raw.as_deref(), Some("∞"));
        assert_eq!(table.param.as_deref(), Some("ly/lx"));
    }

    #[test]
    fn l_over_a_tables_keep_their_own_parameter() {
        // Nr 27.x não é parametrizada por ly/lx e sim por l/a — a distinção é
        // preservada em vez de normalizada silenciosamente.
        let table = RuschTableData::find(27, Some(1)).expect("tabela Nr 27.1");
        assert_eq!(table.param.as_deref(), Some("l/a"));
        assert_eq!(table.ratio_raw.as_deref(), Some("2,25"));
        assert_eq!(table.ratio, Some(2.25));
    }

    #[test]
    fn shear_table_is_addressed_by_number() {
        let shear = RuschTableData::find(99, None).expect("tabela Nr 99");
        assert!(shear.columns.iter().all(|c| c.group.starts_with("Qx")));
        assert!(RuschTableData::find(12_345, None).is_none());
    }

    #[test]
    fn uniform_load_moments_reproduce_workbook_example() {
        // Aba "Main" do workbook, exemplo resolvido: laje central, tabela Nr 65.1
        // (ly/lx = 4), lx = 7,00 m e carga permanente total q = 10,5209 kN/m².
        // Main!I24..I29 = 32,478 · 5,67077 · 32,478 · -64,4406 · -45,8817 · 10,3105
        let table = RuschTableData::find(65, Some(1)).expect("tabela Nr 65.1");
        let moments = uniform_load_moments(table, 10.5209, 7.0).expect("cálculo");

        let moment = |name: &str| {
            moments
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, value)| *value)
                .unwrap_or_else(|| panic!("resultante {name} ausente"))
        };

        assert!((moment("Mxm") - 32.478).abs() < 1e-3, "Main!I24");
        assert!((moment("Mym") - 5.67077).abs() < 1e-3, "Main!I25");
        assert!((moment("Mxr") - 32.478).abs() < 1e-3, "Main!I26");
        assert!((moment("Mxe") - 64.4406).abs() < 1e-3, "Main!I27");
        assert!((moment("Mye") - 45.8817).abs() < 1e-3, "Main!I28");
        assert!((moment("Myr") - 10.3105).abs() < 1e-3, "Main!I29");
    }

    #[test]
    fn uniform_load_moments_reject_invalid_input() {
        let table = RuschTableData::find(65, Some(1)).expect("tabela Nr 65.1");
        assert!(matches!(
            uniform_load_moments(table, 0.0, 7.0),
            Err(RuschError::InvalidDimension {
                name: "q_kn_m2",
                ..
            })
        ));
        assert!(matches!(
            uniform_load_moments(table, 10.0, -1.0),
            Err(RuschError::InvalidDimension { name: "lx_m", .. })
        ));
    }
}
