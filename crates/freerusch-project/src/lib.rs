//! Arquivo de projeto `.frus` do FreeRusch.
//!
//! O formato é JSON versionado (`schema_version`) e é gravado com escrita
//! atômica via [`craft_atomic::atomic_write`]: uma falha no meio da gravação
//! nunca corrompe o projeto anterior.
//!
//! Este crate é a única camada que conhece o formato em disco. A GUI e a CLI
//! conversam com [`Project`], nunca com o JSON diretamente, de modo que a
//! migração de esquema (Fase 3: `.frus` v2 com armadura) fique contida aqui.

use std::path::Path;

use freerusch_core::{EdgeCondition, TrainType};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Versão do esquema de arquivo entendida por esta build.
pub const SCHEMA_VERSION: u32 = 1;

/// Rótulo de [`EdgeCondition::SimplySupported`] no arquivo de projeto.
pub const EDGE_SIMPLY_SUPPORTED: &str = "simply-supported";
/// Rótulo de [`EdgeCondition::Clamped`] no arquivo de projeto.
pub const EDGE_CLAMPED: &str = "clamped";

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
    /// O rótulo de condição de borda não corresponde a nenhum valor conhecido.
    #[error("condição de borda desconhecida no projeto: \"{0}\"")]
    UnknownEdge(String),
    /// Dimensão ausente, nula, negativa ou não finita.
    #[error("dimensão inválida no projeto: {name} = {value}")]
    InvalidDimension {
        /// Nome do campo (ex.: `"l1_m"`).
        name: &'static str,
        /// Valor recebido.
        value: f64,
    },
    /// Número de vãos fora do que as tabelas de Rüsch cobrem.
    #[error("número de vãos inválido no projeto: {spans} (esperado entre 1 e 4)")]
    InvalidSpans {
        /// Número de vãos recebido.
        spans: usize,
    },
}

/// Um projeto de laje salvo em `.frus`.
///
/// Os campos são públicos para que a GUI possa editá-los diretamente e para
/// que o `Default` sirva de ponto de partida em arquivos incompletos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    /// Versão do esquema gravada no arquivo.
    pub schema_version: u32,
    /// Identificação da obra/painel (texto livre).
    pub title: String,
    /// Trem-tipo da NBR 7188:2013 (`"TB-450"` ou `"TB-240"`).
    pub train: String,
    /// Condição de borda (`"simply-supported"` ou `"clamped"`).
    pub edge: String,
    /// Número de vãos do tabuleiro contínuo.
    pub spans: usize,
    /// Vão considerado `l₁` (m).
    pub l1_m: f64,
    /// Vão vizinho `l₂` (m); `0.0` para painel isolado.
    pub l2_m: f64,
    /// Largura transversal do tabuleiro `b` (m).
    pub deck_width_m: f64,
    /// Classe do concreto (ex.: `"C25/30"`).
    pub concrete_class: String,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            schema_version: SCHEMA_VERSION,
            title: String::new(),
            train: TrainType::TB450.label().to_string(),
            edge: EDGE_SIMPLY_SUPPORTED.to_string(),
            spans: 1,
            l1_m: 6.0,
            l2_m: 0.0,
            deck_width_m: 6.0,
            concrete_class: "C25/30".to_string(),
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

    /// Condição de borda tipada, ou erro se o rótulo gravado for desconhecido.
    pub fn edge_condition(&self) -> Result<EdgeCondition, ProjectError> {
        let label = self.edge.trim();
        if label.eq_ignore_ascii_case(EDGE_SIMPLY_SUPPORTED) || label.eq_ignore_ascii_case("pinned")
        {
            Ok(EdgeCondition::SimplySupported)
        } else if label.eq_ignore_ascii_case(EDGE_CLAMPED) || label.eq_ignore_ascii_case("fixed") {
            Ok(EdgeCondition::Clamped)
        } else {
            Err(ProjectError::UnknownEdge(self.edge.clone()))
        }
    }

    /// Valida o projeto inteiro antes de calcular ou gravar.
    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.schema_version > SCHEMA_VERSION {
            return Err(ProjectError::UnsupportedSchema {
                found: self.schema_version,
                supported: SCHEMA_VERSION,
            });
        }
        if !(self.spans >= 1 && self.spans <= 4) {
            return Err(ProjectError::InvalidSpans { spans: self.spans });
        }
        for (name, value) in [("l1_m", self.l1_m), ("deck_width_m", self.deck_width_m)] {
            if !(value.is_finite() && value > 0.0) {
                return Err(ProjectError::InvalidDimension { name, value });
            }
        }
        if !(self.l2_m.is_finite() && self.l2_m >= 0.0) {
            return Err(ProjectError::InvalidDimension {
                name: "l2_m",
                value: self.l2_m,
            });
        }
        self.train_type()?;
        self.edge_condition()?;
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
