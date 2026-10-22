//! Motor de cálculo do FreeRusch: lajes de pontes pelo método de Rüsch.
//!
//! O crate embarca as **186 tabelas de Rüsch** (DIN/IBSW) em
//! `data/rusch_tables.json` e implementa os dois caminhos de carga:
//!
//! - **carga permanente** — [`uniform_load_moments`]: `M = k · q · lx²`;
//! - **carga móvel** — [`live_load_moments`]:
//!   `M = φ · (P·M_L + q·M_p + q·M_p')`, com [`impact_coefficient`] e
//!   [`load_spread_width_m`];
//! - **combinação** — [`design_moments`], devolvendo `M_d = γg·M_g + γq·M_q`.
//!
//! As tabelas são endereçadas por número ([`RuschTableData::find`]) e os rótulos
//! de coluna são preservados crus, inclusive prefixos ainda não interpretados
//! (`¾`, `(+)`, `-`, `_`).
//!
//! # Exemplo
//!
//! ```
//! use freerusch_core::{RuschDesignInput, design_moments};
//!
//! // Exemplo resolvido do workbook: tabela Nr 65.1, lx = 7,00 m.
//! let input = RuschDesignInput {
//!     table_number: 65,
//!     table_variant: Some(1),
//!     lx_m: 7.0,
//!     slab_m: 0.26,
//!     pavement_m: 0.07,
//!     wheel_spacing_m: 2.0,
//!     permanent_load_kn_m2: 10.5209,
//!     wheel_load_kn: 75.0,
//!     uniform_load_kn_m2: 5.0,
//! };
//! let moments = design_moments(&input).expect("cálculo válido");
//! assert!(moments.iter().any(|m| m.resultant == "Mxm" && m.design > 0.0));
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

/// Coeficiente de impacto vertical da NBR 7188:2013.
///
/// `φ = 1,4 − 0,007 · lx`, com `lx` em metros.
///
/// Validado em dois pontos independentes:
/// - `lx = 7,00 m → 1,351`, o valor que o workbook traz na aba `Main!I32`;
/// - `lx = 2,70 m → 1,38`, o valor das notas de aula do IME (p. 2).
///
/// # Erros
///
/// [`RuschError::InvalidDimension`] se `lx_m` não for finito e positivo.
pub fn impact_coefficient(lx_m: f64) -> Result<f64, RuschError> {
    if !(lx_m.is_finite() && lx_m > 0.0) {
        return Err(RuschError::InvalidDimension {
            name: "lx_m",
            value: lx_m,
        });
    }
    Ok(1.4 - 0.007 * lx_m)
}

/// Largura de espraiamento da carga de roda até a meia-espessura da laje.
///
/// `t = √(0,20 · 0,50) + 2 · (pavimento + laje/2)`, que é a área de contato
/// 0,20 m × 0,50 m convertida em quadrado equivalente e distribuída a 45° até a
/// meia-espessura. Exemplo das notas do IME: pavimento 0,07 m e laje 0,23 m
/// resultam em `t = 0,69 m`.
///
/// # Erros
///
/// [`RuschError::InvalidDimension`] se alguma espessura não for finita e não
/// negativa.
pub fn load_spread_width_m(pavement_m: f64, slab_m: f64) -> Result<f64, RuschError> {
    for (name, value) in [("pavement_m", pavement_m), ("slab_m", slab_m)] {
        if !(value.is_finite() && value >= 0.0) {
            return Err(RuschError::InvalidDimension { name, value });
        }
    }
    Ok((0.20_f64 * 0.50).sqrt() + 2.0 * (pavement_m + slab_m / 2.0))
}

/// Resultantes conhecidas, em ordem de preferência de casamento (mais longa
/// primeiro) para lidar com rótulos sujos como `-Mxe` ou `¾ Myr`.
const RESULTANT_KEYS: [&str; 9] = ["Mxm", "Mym", "Mxr", "Mxe", "Mye", "Myr", "Mx", "My", "Qx"];

/// Remove prefixos/sufixos não alfanuméricos dos rótulos crus da planilha.
fn normalized_label(label: &str) -> String {
    label.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Identifica a resultante de um rótulo cru de grupo.
fn resultant_key(label: &str) -> Option<&'static str> {
    let normalized = normalized_label(label);
    RESULTANT_KEYS
        .iter()
        .find(|key| normalized.starts_with(**key))
        .copied()
}

/// Interpolação linear na coluna `rows` (`lx/a`).
///
/// Células `-` entram como contribuição nula, que é o significado da ausência
/// nessas colunas.
fn interpolate_rows(values: &[Option<f64>], rows: &[f64], target: f64) -> Option<f64> {
    let point = |index: usize| values.get(index).copied().flatten().unwrap_or(0.0);
    for index in 0..rows.len().saturating_sub(1) {
        let (x0, x1) = (rows[index], rows[index + 1]);
        if (x0..=x1).contains(&target) {
            let span = x1 - x0;
            let weight = if span.abs() < f64::EPSILON {
                0.0
            } else {
                (target - x0) / span
            };
            return Some(point(index) + weight * (point(index + 1) - point(index)));
        }
    }
    None
}

/// Coeficiente da parcela de roda (`L`), interpolado em `lx/a` e depois em `t/a`.
fn wheel_coefficient(
    table: &RuschTableData,
    resultant: &str,
    lx_over_a: f64,
    t_over_a: f64,
) -> Option<f64> {
    let mut columns: Vec<&RuschColumn> = table
        .columns
        .iter()
        .filter(|column| {
            column.letter == "L"
                && column.ta.is_some()
                && resultant_key(&column.group) == Some(resultant)
        })
        .collect();
    if columns.is_empty() {
        return None;
    }
    columns.sort_by(|a, b| a.ta.partial_cmp(&b.ta).unwrap_or(std::cmp::Ordering::Equal));

    let ta_values: Vec<f64> = columns.iter().filter_map(|c| c.ta).collect();
    let per_column: Vec<f64> = columns
        .iter()
        .map(|c| interpolate_rows(&c.values, &table.rows, lx_over_a))
        .collect::<Option<Vec<_>>>()?;

    for index in 0..ta_values.len().saturating_sub(1) {
        let (t0, t1) = (ta_values[index], ta_values[index + 1]);
        if (t0..=t1).contains(&t_over_a) {
            let span = t1 - t0;
            let weight = if span.abs() < f64::EPSILON {
                0.0
            } else {
                (t_over_a - t0) / span
            };
            return Some(per_column[index] + weight * (per_column[index + 1] - per_column[index]));
        }
    }
    None
}

/// Coeficiente da parcela de carga uniforme (`p` ou `p'`), válido para todo `t/a`.
fn uniform_coefficient(
    table: &RuschTableData,
    resultant: &str,
    letter: &str,
    lx_over_a: f64,
) -> Option<f64> {
    table
        .columns
        .iter()
        .filter(|column| column.letter == letter && resultant_key(&column.group) == Some(resultant))
        .find_map(|column| interpolate_rows(&column.values, &table.rows, lx_over_a))
}

/// Momentos fletores por metro devidos à **carga móvel** de um trem-tipo.
///
/// Implementa o método das tabelas de Rüsch como descrito nas notas de aula do
/// IME (Prof. Eduardo C. S. Thomaz, "Lajes do tabuleiro — Método de Rüsch"):
///
/// ```text
/// M = φ · ( P · M_L + q · M_p + q · M_p' )
/// ```
///
/// com `φ` de [`impact_coefficient`], `P` a carga por roda, `q` a carga
/// uniforme da pista e `M_L`, `M_p`, `M_p'` interpolados na tabela em
/// `(lx/a, t/a)`, onde `a` é a distância transversal entre rodas e `t` vem de
/// [`load_spread_width_m`].
///
/// A interpolação é bilinear: primeiro em `lx/a`, depois em `t/a`.
///
/// # Validação
///
/// Reproduz o exemplo resolvido do workbook (`Main!I34..I39`) com a tabela
/// Nr 65.1 dentro de 0,2 % nos seis momentos: `76,00 · 38,68 · 144,40 ·
/// 131,55 · 48,25 · 50,79 kN·m/m`.
///
/// # Erros
///
/// Dimensão inválida, ou ausência dos coeficientes da resultante pedida na
/// tabela (retorna erro em vez de zero silencioso).
pub fn live_load_moments(
    table: &RuschTableData,
    wheel_load_kn: f64,
    uniform_load_kn_m2: f64,
    lx_m: f64,
    pavement_m: f64,
    slab_m: f64,
    wheel_spacing_m: f64,
) -> Result<Vec<(String, f64)>, RuschError> {
    for (name, value) in [
        ("wheel_load_kn", wheel_load_kn),
        ("uniform_load_kn_m2", uniform_load_kn_m2),
        ("lx_m", lx_m),
        ("wheel_spacing_m", wheel_spacing_m),
    ] {
        if !(value.is_finite() && value > 0.0) {
            return Err(RuschError::InvalidDimension { name, value });
        }
    }
    let impact = impact_coefficient(lx_m)?;
    let spread = load_spread_width_m(pavement_m, slab_m)?;
    let lx_over_a = lx_m / wheel_spacing_m;
    let t_over_a = spread / wheel_spacing_m;

    // Resultantes presentes como coeficiente de roda, na ordem do arquivo.
    let mut resultants: Vec<&'static str> = Vec::new();
    for column in &table.columns {
        if column.letter != "L" || column.ta.is_none() {
            continue;
        }
        if let Some(key) = resultant_key(&column.group)
            && !resultants.contains(&key)
        {
            resultants.push(key);
        }
    }

    let mut moments = Vec::with_capacity(resultants.len());
    for resultant in resultants {
        let wheel = wheel_coefficient(table, resultant, lx_over_a, t_over_a).ok_or_else(|| {
            RuschError::TableLookup {
                sheet: table.sheet.clone(),
                detail: format!("{resultant}: lx/a = {lx_over_a:.3}, t/a = {t_over_a:.3}"),
            }
        })?;
        let positive = uniform_coefficient(table, resultant, "p", lx_over_a).unwrap_or(0.0);
        let negative = uniform_coefficient(table, resultant, "p'", lx_over_a).unwrap_or(0.0);
        let moment = impact * (wheel_load_kn * wheel + uniform_load_kn_m2 * (positive + negative));
        moments.push((resultant.to_string(), moment));
    }
    Ok(moments)
}

/// Coeficiente de ponderação das ações permanentes (ELU, combinação normal).
pub const GAMMA_G: f64 = 1.35;

/// Coeficiente de ponderação das ações variáveis (carga móvel).
pub const GAMMA_Q: f64 = 1.5;

/// Entrada de um dimensionamento pelas tabelas de Rüsch.
///
/// Reúne a identificação da tabela e as grandezas físicas que alimentam os dois
/// caminhos validados: carga permanente ([`uniform_load_moments`]) e carga móvel
/// ([`live_load_moments`]).
#[derive(Debug, Clone, PartialEq)]
pub struct RuschDesignInput {
    /// Número da tabela (`Nr N`).
    pub table_number: u32,
    /// Variante da tabela (o `.1`/`.2` do nome), quando existir.
    pub table_variant: Option<u8>,
    /// Vão transversal `lx` (m).
    pub lx_m: f64,
    /// Espessura da laje (m).
    pub slab_m: f64,
    /// Espessura da pavimentação (m).
    pub pavement_m: f64,
    /// Distância transversal entre rodas (m), entrada `lx/a` da tabela.
    pub wheel_spacing_m: f64,
    /// Carga permanente distribuída (kN/m²).
    pub permanent_load_kn_m2: f64,
    /// Carga por roda do trem-tipo (kN).
    pub wheel_load_kn: f64,
    /// Carga uniforme da pista (kN/m²).
    pub uniform_load_kn_m2: f64,
}

/// Momentos de uma resultante: característicos e de projeto.
#[derive(Debug, Clone, PartialEq)]
pub struct DesignMoment {
    /// Resultante (`"Mxm"`, `"Mxe"`, ...).
    pub resultant: String,
    /// Momento devido à carga permanente (kN·m/m).
    pub permanent: f64,
    /// Momento devido à carga móvel, já com impacto (kN·m/m).
    pub live: f64,
    /// Momento de projeto: `γg · permanente + γq · móvel` (kN·m/m).
    pub design: f64,
}

/// Momentos por resultante pelo método das tabelas de Rüsch.
///
/// Combina os dois caminhos validados do crate:
///
/// ```text
/// M_k,g = k · (g + revestimento) · lx²                  -- carga permanente
/// M_k,q = φ · (P·M_L + q·M_p + q·M_p')                  -- carga móvel
/// M_d   = γg · M_k,g + γq · M_k,q
/// ```
///
/// # Validação
///
/// Para a tabela Nr 65.1 com `lx = 7,00 m` (exemplo do workbook), os campos
/// `permanent` e `live` reproduzem `Main!I24..I29` e `Main!I34..I39`.
///
/// # Erros
///
/// [`RuschError::TableLookup`] se a tabela pedida não existir, e
/// [`RuschError::InvalidDimension`] para entrada fora de domínio.
pub fn design_moments(input: &RuschDesignInput) -> Result<Vec<DesignMoment>, RuschError> {
    let table = RuschTableData::find(input.table_number, input.table_variant).ok_or_else(|| {
        RuschError::TableLookup {
            sheet: match input.table_variant {
                Some(variant) => format!("Nr {}.{variant}", input.table_number),
                None => format!("Nr {}", input.table_number),
            },
            detail: "tabela não encontrada no acervo embarcado".to_string(),
        }
    })?;

    let permanent = uniform_load_moments(table, input.permanent_load_kn_m2, input.lx_m)?;
    let live = live_load_moments(
        table,
        input.wheel_load_kn,
        input.uniform_load_kn_m2,
        input.lx_m,
        input.pavement_m,
        input.slab_m,
        input.wheel_spacing_m,
    )?;

    let mut moments: Vec<DesignMoment> = Vec::new();
    for (name, value) in permanent {
        if let Some(entry) = moments.iter_mut().find(|m| m.resultant == name) {
            entry.permanent = value;
        } else {
            moments.push(DesignMoment {
                resultant: name,
                permanent: value,
                live: 0.0,
                design: 0.0,
            });
        }
    }
    for (name, value) in live {
        if let Some(entry) = moments.iter_mut().find(|m| m.resultant == name) {
            entry.live = value;
        } else {
            moments.push(DesignMoment {
                resultant: name,
                permanent: 0.0,
                live: value,
                design: 0.0,
            });
        }
    }
    for entry in &mut moments {
        entry.design = GAMMA_G * entry.permanent + GAMMA_Q * entry.live;
    }
    Ok(moments)
}

/// Erros do motor de cálculo Rüsch.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RuschError {
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
    /// A tabela de Rüsch não cobre o ponto pedido, ou não traz a resultante.
    #[error("tabela {sheet} não cobre o ponto pedido: {detail}")]
    TableLookup {
        /// Planilha de origem da tabela.
        sheet: String,
        /// Resultante e relações `lx/a`, `t/a` solicitadas.
        detail: String,
    },
}

/// Condição de contorno do painel contínuo (apoios extremos).
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

    #[test]
    fn impact_coefficient_matches_both_sources() {
        // Workbook, aba "Main": lx = 7,00 m -> phi = 1,351 (Main!I32).
        assert!((impact_coefficient(7.0).expect("cálculo") - 1.351).abs() < 1e-9);
        // Notas do IME, p.2: lx = 2,70 m -> phi = 1,38 (as notas arredondam
        // para duas casas; o valor exato da fórmula é 1,3811).
        assert!((impact_coefficient(2.70).expect("cálculo") - 1.38).abs() < 0.005);
    }

    #[test]
    fn load_spread_width_matches_course_notes() {
        // Notas do IME, p.2: pavimento 7 cm, laje 23 cm -> t = 0,69 m.
        assert!((load_spread_width_m(0.07, 0.23).expect("cálculo") - 0.69).abs() < 0.005);
    }

    #[test]
    fn live_load_moments_reproduce_workbook_example() {
        // Aba "Main": tabela Nr 65.1 (ly/lx = 4), lx = 7,00 m, laje 26 cm,
        // pavimento 7 cm, a = 2,00 m, TB-450 (P = 75 kN/roda, q = 5 kN/m²).
        // Alvos em Main!I34..I39 (kN·m/m, sinal absoluto).
        let table = RuschTableData::find(65, Some(1)).expect("tabela Nr 65.1");
        let moments = live_load_moments(table, 75.0, 5.0, 7.0, 0.07, 0.26, 2.0).expect("cálculo");

        let moment = |name: &str| {
            moments
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, value)| *value)
                .unwrap_or_else(|| panic!("resultante {name} ausente"))
        };

        for (name, expected, cell) in [
            ("Mxm", 76.0005, "Main!I34"),
            ("Mym", 38.6751, "Main!I35"),
            ("Mxr", 144.401, "Main!I36"),
            ("Mxe", 131.548, "Main!I37"),
            ("Mye", 48.2468, "Main!I38"),
            ("Myr", 50.7861, "Main!I39"),
        ] {
            let computed = moment(name);
            let relative = (computed.abs() - expected).abs() / expected;
            assert!(
                relative < 0.005,
                "{name} ({cell}): calculado {computed:.4}, esperado {expected:.4} (erro {:.2}%)",
                relative * 100.0
            );
        }
    }

    #[test]
    fn live_load_moments_reject_invalid_input() {
        let table = RuschTableData::find(65, Some(1)).expect("tabela Nr 65.1");
        assert!(matches!(
            live_load_moments(table, 0.0, 5.0, 7.0, 0.07, 0.26, 2.0),
            Err(RuschError::InvalidDimension {
                name: "wheel_load_kn",
                ..
            })
        ));
        assert!(matches!(
            live_load_moments(table, 75.0, 5.0, 7.0, 0.07, 0.26, 0.0),
            Err(RuschError::InvalidDimension {
                name: "wheel_spacing_m",
                ..
            })
        ));
    }

    #[test]
    fn design_moments_combine_both_validated_paths() {
        // Reproduz o exemplo do workbook completo: tabela Nr 65.1, lx = 7,00 m,
        // laje 26 cm, pavimento 7 cm, a = 2,00 m, g = 10,5209 kN/m², TB-450.
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
        let moments = design_moments(&input).expect("cálculo");

        let find = |name: &str| {
            moments
                .iter()
                .find(|m| m.resultant == name)
                .unwrap_or_else(|| panic!("resultante {name} ausente"))
                .clone()
        };

        // Carga permanente: Main!I24..I29.
        assert!((find("Mxm").permanent - 32.478).abs() < 1e-2, "Main!I24");
        assert!((find("Myr").permanent - 10.3105).abs() < 1e-2, "Main!I29");
        // Carga móvel: Main!I34..I39 (0,2 % de tolerância).
        assert!((find("Mxm").live - 76.0005).abs() < 0.5, "Main!I34");
        assert!((find("Mxr").live - 144.401).abs() < 0.5, "Main!I36");

        // Combinação: M_d = 1,35·M_g + 1,5·M_q, termo a termo.
        for moment in &moments {
            let expected = GAMMA_G * moment.permanent + GAMMA_Q * moment.live;
            assert!(
                (moment.design - expected).abs() < 1e-9,
                "{}",
                moment.resultant
            );
        }
        let mxm = find("Mxm");
        assert!(
            (mxm.design - (1.35 * 32.478 + 1.5 * 76.0005)).abs() < 1.0,
            "M_d(Mxm) = {:.2}",
            mxm.design
        );
    }

    #[test]
    fn design_moments_report_unknown_table() {
        let input = RuschDesignInput {
            table_number: 999,
            table_variant: Some(1),
            lx_m: 7.0,
            slab_m: 0.26,
            pavement_m: 0.07,
            wheel_spacing_m: 2.0,
            permanent_load_kn_m2: 10.0,
            wheel_load_kn: 75.0,
            uniform_load_kn_m2: 5.0,
        };
        assert!(matches!(
            design_moments(&input),
            Err(RuschError::TableLookup { .. })
        ));
    }
    #[test]
    fn concrete_classes_map_to_characteristic_strength() {
        assert_eq!(
            concrete_characteristic_strength("C25/30").expect("C25/30"),
            25.0
        );
        assert!(matches!(
            concrete_characteristic_strength("C99/99"),
            Err(RuschError::UnknownConcreteClass { .. })
        ));
    }
}
