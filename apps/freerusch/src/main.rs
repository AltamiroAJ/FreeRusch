//! FreeRusch — cálculo estrutural de lajes de pontes pelo método de Rüsch.
//!
//! Esqueleto mínimo da GUI: janela `eframe` com título "FreeRusch", componentes
//! compartilhados da família via `craft-ui`, configurações persistidas em
//! `config.toml` via `craft-settings` e exportação da memória de cálculo com
//! escrita atômica (`craft-atomic`) e impressão (`craft-print`).

use std::path::PathBuf;

use eframe::egui;
use freerusch_core::{EdgeCondition, RuschTable, TrainType, bending_moment};
use serde::{Deserialize, Serialize};

/// Configurações do FreeRusch persistidas em `config.toml` pela `craft-settings`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
struct FreeruschSettings {
    general: General,
}

/// Seção `[general]` das configurações.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct General {
    /// Último trem-tipo selecionado ("TB-450" ou "TB-240").
    train: String,
}

impl Default for General {
    fn default() -> Self {
        General {
            train: TrainType::TB450.label().to_string(),
        }
    }
}

impl craft_settings::Settings for FreeruschSettings {
    const APP_ID: &'static str = "freerusch";
    const SCHEMA_VERSION: u32 = 1;
}

/// Estado da aplicação.
struct FreeruschApp {
    settings: craft_settings::SettingsStore<FreeruschSettings>,
    l1: f64,
    deck_width: f64,
    last_message: String,
    /// Último momento positivo calculado (kN·m/m), usado na exportação.
    last_moment: Option<f64>,
}

impl FreeruschApp {
    /// Abre a aplicação carregando as configurações do usuário.
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Em testes/CI sem perfil, mantém as configurações apenas em memória.
        let location = if std::env::var_os("FREERUSCH_NO_PREFS").is_some() {
            craft_settings::Location::Memory
        } else {
            craft_settings::Location::Auto
        };
        let (store, _report) = craft_settings::SettingsStore::<FreeruschSettings>::open(location);
        FreeruschApp {
            settings: store,
            l1: 8.0,
            deck_width: 6.0,
            last_message: String::new(),
            last_moment: None,
        }
    }

    /// Recalcula o momento fletor máximo com os valores atuais.
    fn recalculate(&mut self) {
        let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
        let train = if self.settings.get().general.train == TrainType::TB240.label() {
            TrainType::TB240
        } else {
            TrainType::TB450
        };
        match bending_moment(&table, &train, self.l1, 0.0, 25.0) {
            Ok(m) => {
                self.last_moment = Some(m.max_positive);
                self.last_message = format!(
                    "{} · vão {} m → M⁺ = {:.2} kN·m/m (α = {:.4})",
                    train.label(),
                    self.l1,
                    m.max_positive,
                    m.alpha
                );
            }
            Err(e) => {
                self.last_moment = None;
                self.last_message = format!("Erro: {e}");
            }
        }
    }

    /// Exporta a memória de cálculo em HTML com escrita atômica ao lado do
    /// executável (pasta padrão do sistema em PRs seguintes).
    fn export_report(&mut self) {
        if self.last_moment.is_none() {
            self.recalculate();
        }
        let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
        let train = if self.settings.get().general.train == TrainType::TB240.label() {
            TrainType::TB240
        } else {
            TrainType::TB450
        };
        let out = PathBuf::from("memoria-de-calculo.html");
        match bending_moment(&table, &train, self.l1, 0.0, 25.0) {
            Ok(m) => {
                let input = freerusch_report::SlabInput::new(
                    "FreeRusch — painel corrente",
                    train,
                    self.l1,
                    0.0,
                    self.deck_width,
                    "C25/30",
                );
                let html = freerusch_report::render_html(&input, &m);
                match freerusch_report::write_report(&out, &html) {
                    Ok(()) => self.last_message = format!("Memória escrita em {}", out.display()),
                    Err(e) => self.last_message = format!("Falha na exportação: {e}"),
                }
            }
            Err(e) => self.last_message = format!("Erro: {e}"),
        }
    }
}

impl eframe::App for FreeruschApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("FreeRusch");
        ui.label("Cálculo de lajes de pontes pelo método de Rüsch (NBR 7188:2013).");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Vão l₁ (m):");
            ui.add(
                egui::DragValue::new(&mut self.l1)
                    .range(1.0..=40.0)
                    .speed(0.1),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Largura do tabuleiro (m):");
            ui.add(
                egui::DragValue::new(&mut self.deck_width)
                    .range(3.0..=30.0)
                    .speed(0.1),
            );
        });
        let mut train_idx = if self.settings.get().general.train == TrainType::TB240.label() {
            1
        } else {
            0
        };
        egui::ComboBox::from_label("Trem-tipo")
            .selected_text(if train_idx == 0 { "TB-450" } else { "TB-240" })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut train_idx, 0, "TB-450");
                ui.selectable_value(&mut train_idx, 1, "TB-240");
            });
        let label = if train_idx == 0 {
            TrainType::TB450.label()
        } else {
            TrainType::TB240.label()
        };
        self.settings
            .update(|s| s.general.train = label.to_string());
        if ui.button("Calcular").clicked() {
            self.recalculate();
        }
        if ui.button("Exportar memória de cálculo (HTML)").clicked() {
            self.export_report();
        }
        ui.separator();
        ui.label(&self.last_message);

        // Grava as configurações alteradas (debounce da própria craft-settings).
        let now = std::time::Instant::now();
        let tick = self.settings.tick(now);
        if let Some(deadline) = tick.next_deadline {
            ui.ctx()
                .request_repaint_after(deadline.saturating_duration_since(now));
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 420.0])
            .with_title("FreeRusch"),
        ..Default::default()
    };
    eframe::run_native(
        "FreeRusch",
        options,
        Box::new(|cc| Ok(Box::new(FreeruschApp::new(cc)))),
    )
}
