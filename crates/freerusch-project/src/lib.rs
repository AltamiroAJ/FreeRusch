//! Arquivo de projeto `.frus` do FreeRusch.
//!
//! O formato é JSON versionado (`schema_version`) e é gravado com escrita
//! atômica via [`craft_atomic::atomic_write`]: uma falha no meio da gravação
//! nunca corrompe o projeto anterior.
//!
//! Este crate é a única camada que conhece o formato em disco. A GUI e a CLI
//! conversam com [`Project`], nunca com o JSON diretamente.
//!
//! # Esquema v2
//!
//! A v2 trocou o modelo antigo (vãos/viga/trem-tipo genéricos) pelo
//! dimensionamento efetivo por **tabela de Rüsch**: número da tabela, espessuras,
//! pavimento, distância entre rodas e as cargas. Campos ausentes de um arquivo v1
//! caem no `Default`, que é o exemplo resolvido do workbook.

use std::path::Path;

use freerusch_core::{RuschDesignInput, RuschTableData, TrainType};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Versão do esquema de arquivo entendida por esta build.
pub const SCHEMA_VERSION: u32 = 2;

/// Erros de leitura, validação e gravação do arquivo de projeto.
#[derive(Debug, Error)]
pub enum ProjectError {
    /// Falha de E/S ao ler ou gravar o arquivo.
    #[error("falha de E/S no arquivo de projeto: {0}")]
    Io(#[from] std::io::Error),
    /// O conteúdo não é JSON válido ou não casa com o esquema.
    #[error("arquivo de projeto inválido: {0}")]
    Json(#[from] serde_json::Error),
    /// O arquivo foi gravado por uma versão mais nova do FreeRusch.
    #[error("versão de esquema {found} não suportada (esta build entende até {supported})")]
    UnsupportedSchema {
        /// Versão encontrada no arquivo.
        found: u32,
        /// Última versão suportada por esta build.
        supported: u32,
    },
    /// O rótulo de trem-tipo não corresponde a nenhum valor conhecido.
    #[error("trem-tipo desconhecido no projeto: \"{0}\"")]
    UnknownTrain(String),
    /// A tabela de Rüsch indicada não existe no acervo embarcado.
    #[error("tabela de Rüsch inexistente no projeto: Nr {number}.{variant:?}")]
    UnknownTable {
        /// Número da tabela.
        number: u32,
        /// Variante informada.
        variant: Option<u8>,
    },
    /// Dimensão ausente, nula, negativa ou não finita.
    #[error("dimensão inválida no projeto: {name} = {value}")]
    InvalidDimension {
        /// Nome do campo (ex.: `"lx_m"`).
        name: &'static str,
        /// Valor recebido.
        value: f64,
    },
}

/// Um projeto de laje salvo em `.frus`.
///
/// Os campos são públicos para que a GUI possa editá-los diretamente e para que
/// o `Default` sirva de ponto de partida em arquivos incompletos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    /// Versão do esquema gravada no arquivo.
    pub schema_version: u32,
    /// Identificação da obra/painel (texto livre).
    pub title: String,
    /// Trem-tipo da NBR 7188:2013 (`"TB-450"` ou `"TB-240"`), informativo.
    pub train: String,
    /// Classe do concreto (ex.: `"C25/30"`).
    pub concrete_class: String,
    /// Número da tabela de Rüsch.
    pub table_number: u32,
    /// Variante da tabela (o `.1`/`.2` do nome), quando existir.
    pub table_variant: Option<u8>,
    /// Vão transversal `lx` (m).
    pub lx_m: f64,
    /// Espessura da laje (m).
    pub slab_m: f64,
    /// Espessura da pavimentação (m).
    pub pavement_m: f64,
    /// Distância transversal entre rodas (m) — entrada `lx/a` da tabela.
    pub wheel_spacing_m: f64,
    /// Carga permanente distribuída (kN/m²).
    pub permanent_load_kn_m2: f64,
    /// Carga por roda do trem-tipo (kN).
    pub wheel_load_kn: f64,
    /// Carga uniforme da pista (kN/m²).
    pub uniform_load_kn_m2: f64,
}

impl Default for Project {
    /// Padrões iguais ao exemplo resolvido do workbook (tabela Nr 65.1).
    fn default() -> Self {
        Project {
            schema_version: SCHEMA_VERSION,
            title: String::new(),
            train: TrainType::TB450.label().to_string(),
            concrete_class: "C25/30".to_string(),
            table_number: 65,
            table_variant: Some(1),
            lx_m: 7.0,
            slab_m: 0.26,
            pavement_m: 0.07,
            wheel_spacing_m: 2.0,
            permanent_load_kn_m2: 10.5209,
            wheel_load_kn: 75.0,
            uniform_load_kn_m2: 5.0,
        }
    }
}

impl Project {
    /// Trem-tipo tipado, ou erro se o rótulo gravado for desconhecido.
    pub fn train_type(&self) -> Result<TrainType, ProjectError> {
        let label = self.train.trim();
        for candidate in [TrainType::TB450, TrainType::TB240] {
            let canonical = candidate.label();
            let compact = canonical.replace('-', "");
            if label.eq_ignore_ascii_case(canonical) || label.eq_ignore_ascii_case(&compact) {
                return Ok(candidate);
            }
        }
        Err(ProjectError::UnknownTrain(self.train.clone()))
    }

    /// Entrada de cálculo do `freerusch-core`, pronta para [`freerusch_core::design_moments`].
    #[must_use]
    pub fn design_input(&self) -> RuschDesignInput {
        RuschDesignInput {
            table_number: self.table_number,
            table_variant: self.table_variant,
            lx_m: self.lx_m,
            slab_m: self.slab_m,
            pavement_m: self.pavement_m,
            wheel_spacing_m: self.wheel_spacing_m,
            permanent_load_kn_m2: self.permanent_load_kn_m2,
            wheel_load_kn: self.wheel_load_kn,
            uniform_load_kn_m2: self.uniform_load_kn_m2,
        }
    }

    /// Rótulo legível da tabela (`"Nr 65.1"`), quando ela existe no acervo.
    #[must_use]
    pub fn table_label(&self) -> Option<&'static str> {
        RuschTableData::find(self.table_number, self.table_variant)
            .map(|table| table.sheet.as_str())
    }

    /// Valida o projeto inteiro antes de calcular ou gravar.
    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version > SCHEMA_VERSION {
            return Err(ProjectError::UnsupportedSchema {
                found: self.schema_version,
                supported: SCHEMA_VERSION,
            });
        }
        for (name, value) in [
            ("lx_m", self.lx_m),
            ("wheel_spacing_m", self.wheel_spacing_m),
            ("permanent_load_kn_m2", self.permanent_load_kn_m2),
            ("wheel_load_kn", self.wheel_load_kn),
            ("uniform_load_kn_m2", self.uniform_load_kn_m2),
        ] {
            if !(value.is_finite() && value > 0.0) {
                return Err(ProjectError::InvalidDimension { name, value });
            }
        }
        for (name, value) in [("slab_m", self.slab_m), ("pavement_m", self.pavement_m)] {
            if !(value.is_finite() && value > 0.0) {
                return Err(ProjectError::InvalidDimension { name, value });
            }
        }
        if RuschTableData::find(self.table_number, self.table_variant).is_none() {
            return Err(ProjectError::UnknownTable {
                number: self.table_number,
                variant: self.table_variant,
            });
        }
        self.train_type()?;
        Ok(())
    }

    /// Serializa o projeto em JSON legível (identado).
    pub fn to_json(&self) -> Result<String, ProjectError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Lê e valida um projeto a partir de JSON.
    pub fn from_json(text: &str) -> Result<Project, ProjectError> {
        let project: Project = serde_json::from_str(text)?;
        project.validate()?;
        Ok(project)
    }

    /// Grava o projeto em `path` com escrita atômica.
    ///
    /// O projeto é validado antes de tocar o disco: um projeto inválido nunca
    /// substitui um arquivo bom.
    pub fn save(&self, path: &Path) -> Result<(), ProjectError> {
        self.validate()?;
        let json = self.to_json()?;
        craft_atomic::atomic_write(path, json.as_bytes())?;
        Ok(())
    }

    /// Carrega e valida um projeto de `path`.
    pub fn load(path: &Path) -> Result<Project, ProjectError> {
        let text = std::fs::read_to_string(path)?;
        Project::from_json(&text)
    }
}
