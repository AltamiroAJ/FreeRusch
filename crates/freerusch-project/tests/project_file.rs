//! Aceitação do arquivo de projeto `.frus` (esquema v2).
//!
//! Estes testes usam apenas a API pública, como um consumidor real (GUI ou CLI).

use std::path::PathBuf;

use freerusch_core::TrainType;
use freerusch_project::{Project, ProjectError, SCHEMA_VERSION};

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("freerusch-project-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("criar diretório temporário");
    dir
}

fn sample() -> Project {
    Project {
        title: "Viaduto Centro — Painel P1".to_string(),
        table_number: 65,
        table_variant: Some(1),
        lx_m: 7.0,
        slab_m: 0.26,
        pavement_m: 0.07,
        wheel_spacing_m: 2.0,
        permanent_load_kn_m2: 10.5209,
        wheel_load_kn: 75.0,
        uniform_load_kn_m2: 5.0,
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
    assert_eq!(back.table_label(), Some("Nr 65.1"));
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
fn missing_fields_fall_back_to_the_workbook_example() {
    let project = Project::from_json(r#"{"lx_m": 8.0}"#).expect("campos ausentes usam default");
    assert_eq!(project.lx_m, 8.0);
    assert_eq!(project.schema_version, SCHEMA_VERSION);
    assert_eq!(project.table_number, 65);
    assert_eq!(project.table_variant, Some(1));
    assert_eq!(project.slab_m, 0.26);
}

#[test]
fn v1_project_still_loads() {
    // Arquivo do esquema v1: os campos novos caem no default e os antigos são
    // ignorados. A validação não pode rejeitar um arquivo legítimo.
    let v1 = r#"{
      "schema_version": 1,
      "title": "projeto antigo",
      "train": "TB-240",
      "edge": "clamped",
      "spans": 2,
      "l1_m": 8.0,
      "deck_width_m": 6.0
    }"#;
    let project = Project::from_json(v1).expect("v1 deve carregar");
    assert_eq!(project.title, "projeto antigo");
    assert_eq!(project.train, "TB-240");
    assert_eq!(project.table_number, 65);
    assert_eq!(project.lx_m, 7.0);
}

#[test]
fn rejects_unknown_table() {
    let mut project = sample();
    project.table_number = 4242;
    assert!(matches!(
        project.validate(),
        Err(ProjectError::UnknownTable { number: 4242, .. })
    ));
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
    project.lx_m = 0.0;
    assert!(matches!(
        project.validate(),
        Err(ProjectError::InvalidDimension { name: "lx_m", .. })
    ));
}

#[test]
fn design_input_carries_the_project_values() {
    let project = sample();
    let input = project.design_input();
    assert_eq!(input.table_number, 65);
    assert_eq!(input.table_variant, Some(1));
    assert_eq!(input.lx_m, 7.0);
    assert_eq!(input.slab_m, 0.26);
    assert_eq!(input.pavement_m, 0.07);
    assert_eq!(input.wheel_spacing_m, 2.0);
    assert_eq!(input.permanent_load_kn_m2, 10.5209);
    assert_eq!(input.wheel_load_kn, 75.0);
    assert_eq!(input.uniform_load_kn_m2, 5.0);
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
