//! Aceitação do arquivo de projeto `.frus` (Fase 2, tarefa 4).
//!
//! Estes testes usam apenas a API pública, como um consumidor real (GUI ou CLI).

use std::path::PathBuf;

use freerusch_core::{EdgeCondition, TrainType};
use freerusch_project::{Project, ProjectError, SCHEMA_VERSION};

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("freerusch-project-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("criar diretório temporário");
    dir
}

fn sample() -> Project {
    Project {
        title: "Viaduto Centro — Painel P1".to_string(),
        train: TrainType::TB450.label().to_string(),
        edge: "simply-supported".to_string(),
        spans: 2,
        l1_m: 8.0,
        l2_m: 8.0,
        deck_width_m: 6.0,
        concrete_class: "C25/30".to_string(),
        ..Default::default()
    }
}

#[test]
fn json_round_trip_preserves_fields() {
    let project = sample();
    let json = project.to_json().expect("serializar");
    assert!(json.contains("\"schema_version\""));

    let back = Project::from_json(&json).expect("desserializar");
    assert_eq!(back, project);
    assert_eq!(back.train_type().expect("trem"), TrainType::TB450);
    assert_eq!(
        back.edge_condition().expect("borda"),
        EdgeCondition::SimplySupported
    );
}

#[test]
fn save_and_load_round_trip_on_disk() {
    let dir = temp_dir("save");
    let path = dir.join("projeto.frus");
    let project = sample();

    project.save(&path).expect("salvar");
    assert!(path.exists(), "o arquivo .frus deve existir após salvar");

    let back = Project::load(&path).expect("carregar");
    assert_eq!(back, project);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn missing_fields_fall_back_to_defaults() {
    let project = Project::from_json(r#"{"l1_m": 12.0}"#).expect("campos ausentes usam default");
    assert_eq!(project.l1_m, 12.0);
    assert_eq!(project.schema_version, SCHEMA_VERSION);
    assert_eq!(project.train, TrainType::TB450.label());
}

#[test]
fn rejects_unknown_train() {
    let mut project = sample();
    project.train = "TB-999".to_string();
    assert!(matches!(
        project.train_type(),
        Err(ProjectError::UnknownTrain(_))
    ));
}

#[test]
fn rejects_unknown_edge_condition() {
    let mut project = sample();
    project.edge = "welded".to_string();
    assert!(matches!(
        project.edge_condition(),
        Err(ProjectError::UnknownEdge(_))
    ));
}

#[test]
fn rejects_newer_schema_version() {
    let mut project = sample();
    project.schema_version = SCHEMA_VERSION + 1;
    assert!(matches!(
        project.validate(),
        Err(ProjectError::UnsupportedSchema { .. })
    ));
}

#[test]
fn rejects_non_positive_dimension() {
    let mut project = sample();
    project.l1_m = 0.0;
    assert!(matches!(
        project.validate(),
        Err(ProjectError::InvalidDimension { name: "l1_m", .. })
    ));
}

#[test]
fn load_rejects_malformed_json() {
    let dir = temp_dir("bad");
    let path = dir.join("quebrado.frus");
    std::fs::write(&path, "{ isto não é json").expect("escrever lixo");

    assert!(matches!(Project::load(&path), Err(ProjectError::Json(_))));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn save_leaves_no_staging_files_behind() {
    let dir = temp_dir("staging");
    let path = dir.join("projeto.frus");
    sample().save(&path).expect("salvar");

    let leftovers: Vec<String> = std::fs::read_dir(&dir)
        .expect("listar diretório")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name != "projeto.frus")
        .collect();
    assert!(
        leftovers.is_empty(),
        "escrita atômica não deve deixar arquivos temporários: {leftovers:?}"
    );

    std::fs::remove_dir_all(&dir).ok();
}
