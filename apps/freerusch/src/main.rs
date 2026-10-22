//! FreeRusch — cálculo estrutural de lajes de pontes pelo método de Rüsch.
//!
//! GUI sobre `eframe`/`egui` com três abas: **Inputs** (tabela de Rüsch e
//! cargas), **Results** (momentos característicos e de projeto) e **Report**
//! (memória de cálculo em HTML). A barra de abas usa os primitivos de
//! `craft_ui::tabs` (`fit` para larguras, `Tab::show_at` para interação e
//! acessibilidade), já que a `craft-ui` não fornece um contêiner de abas pronto.
//!
//! O cálculo usa as 186 tabelas de Rüsch embarcadas em `freerusch-core`, pelos
//! caminhos validados contra o exemplo resolvido do workbook e contra as notas
//! de aula do IME. Persistência: preferências em `config.toml` via
//! `craft-settings`; projeto em `.frus` via `freerusch-project`; memória de
//! cálculo em HTML via `freerusch-report`.

use std::path::Path;

use eframe::egui;
use freerusch_core::{DesignMoment, TrainType};
use freerusch_project::Project;
use freerusch_report::{ReportContext, render_html, write_report};
use serde::{Deserialize, Serialize};

/// Rótulos das abas, na ordem de exibição.
const TABS: [&str; 3] = ["Inputs", "Results", "Report"];
const TAB_INPUTS: usize = 0;
const TAB_RESULTS: usize = 1;
const TAB_REPORT: usize = 2;

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
    /// Último caminho de projeto `.frus` utilizado.
    last_project_path: String,
    /// Último caminho de memória de cálculo exportada.
    last_report_path: String,
}

impl Default for General {
    fn default() -> Self {
        General {
            train: TrainType::TB450.label().to_string(),
            last_project_path: "projeto.frus".to_string(),
            last_report_path: "memoria-de-calculo.html".to_string(),
        }
    }
}

impl craft_settings::Settings for FreeruschSettings {
    const APP_ID: &'static str = "freerusch";
    const SCHEMA_VERSION: u32 = 1;
}

/// Carga por roda e sobrecarga uniforme convencionais de cada trem-tipo.
///
/// `TB-450`: 450 kN em 6 rodas = 75 kN; `TB-240`: 240 kN em 6 rodas = 40 kN
/// (média por roda — as cargas por eixo da NBR 7188 não são iguais).
/// Sobrecargas: 5,0 e 4,0 kN/m² conforme NBR 7188:2013.
fn train_defaults(train: TrainType) -> (f64, f64) {
    match train {
        TrainType::TB450 => (75.0, 5.0),
        TrainType::TB240 => (40.0, 4.0),
    }
}

/// Estado da aplicação.
struct FreeruschApp {
    settings: craft_settings::SettingsStore<FreeruschSettings>,
    project: Project,
    tab: usize,
    moments: Vec<DesignMoment>,
    report_html: String,
    message: String,
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
        let (store, report) = craft_settings::SettingsStore::<FreeruschSettings>::open(location);
        let project = Project {
            train: store.get().general.train.clone(),
            ..Project::default()
        };
        let mut app = FreeruschApp {
            settings: store,
            project,
            tab: TAB_INPUTS,
            moments: Vec::new(),
            report_html: String::new(),
            message: String::new(),
        };
        if report.has_problems() {
            app.message =
                "Configurações não puderam ser lidas por completo; usando padrões.".into();
        }
        app
    }

    /// Recalcula os momentos característicos e de projeto.
    fn calculate(&mut self) {
        self.message.clear();
        if let Err(error) = self.project.validate() {
            self.moments.clear();
            self.report_html.clear();
            self.message = format!("Projeto inválido: {error}");
            return;
        }

        let input = self.project.design_input();
        match freerusch_core::design_moments(&input) {
            Ok(moments) => {
                let title = self.project.title.clone();
                let concrete = self.project.concrete_class.clone();
                let label = self.project.table_label().unwrap_or("—").to_string();
                self.report_html = render_html(
                    &input,
                    &ReportContext {
                        title: &title,
                        concrete_class: &concrete,
                        table_label: &label,
                    },
                    &moments,
                );
                if let Some(mxm) = moments.iter().find(|moment| moment.resultant == "Mxm") {
                    self.message = format!(
                        "{label} · lx {:.2} m → M_d(Mxm) = {:.2} kN·m/m \
                         (M_g {:.2} + M_q {:.2})",
                        input.lx_m, mxm.design, mxm.permanent, mxm.live
                    );
                } else {
                    self.message = format!("Cálculo concluído com {label}.");
                }
                self.moments = moments;
                self.tab = TAB_RESULTS;
            }
            Err(error) => {
                self.moments.clear();
                self.report_html.clear();
                self.message = format!("Erro de cálculo: {error}");
            }
        }
    }

    /// Exporta a memória de cálculo em HTML com escrita atômica.
    fn export_report(&mut self) {
        if self.moments.is_empty() {
            self.calculate();
        }
        if self.report_html.is_empty() {
            self.message = "Nada a exportar: calcule primeiro.".into();
            return;
        }
        let path = self.settings.get().general.last_report_path.clone();
        match write_report(Path::new(&path), &self.report_html) {
            Ok(()) => self.message = format!("Memória escrita em {path}"),
            Err(e) => self.message = format!("Falha na exportação: {e}"),
        }
    }

    /// Grava o projeto atual em `.frus`.
    fn save_project(&mut self) {
        let path = self.settings.get().general.last_project_path.clone();
        match self.project.save(Path::new(&path)) {
            Ok(()) => self.message = format!("Projeto salvo em {path}"),
            Err(e) => self.message = format!("Falha ao salvar projeto: {e}"),
        }
    }

    /// Carrega um projeto `.frus`, substituindo o estado atual em caso de êxito.
    fn open_project(&mut self) {
        let path = self.settings.get().general.last_project_path.clone();
        match Project::load(Path::new(&path)) {
            Ok(project) => {
                self.project = project;
                self.settings
                    .update(|s| s.general.train = self.project.train.clone());
                self.calculate();
                self.tab = TAB_INPUTS;
                self.message = format!("Projeto carregado de {path}");
            }
            Err(e) => self.message = format!("Falha ao abrir projeto: {e}"),
        }
    }
}

/// Desenha a barra de abas usando os primitivos de `craft_ui::tabs`.
///
/// Retorna `true` quando a seleção mudou nesta passagem.
fn tab_bar(ui: &mut egui::Ui, selected: &mut usize) -> bool {
    const TAB_HEIGHT: f32 = 28.0;
    const MIN_TAB_WIDTH: f32 = 64.0;
    const OVERFLOW_WIDTH: f32 = 28.0;

    let available = ui.available_width();
    let font = egui::TextStyle::Button.resolve(ui.style());
    let natural: Vec<f32> = TABS
        .iter()
        .map(|label| {
            ui.painter()
                .layout_no_wrap((*label).to_owned(), font.clone(), egui::Color32::WHITE)
                .size()
                .x
                + 28.0
        })
        .collect();

    let fitted = craft_ui::tabs::fit(
        &natural,
        *selected,
        available,
        MIN_TAB_WIDTH,
        OVERFLOW_WIDTH,
    );
    let (strip, _) =
        ui.allocate_exact_size(egui::vec2(available, TAB_HEIGHT), egui::Sense::hover());

    let current = *selected;
    let mut changed = false;
    let mut x = strip.left();

    for &(index, width) in &fitted.shown {
        if width <= 0.0 {
            continue;
        }
        let rect =
            egui::Rect::from_min_size(egui::pos2(x, strip.top()), egui::vec2(width, TAB_HEIGHT));
        let is_selected = index == current;
        let label = TABS.get(index).copied().unwrap_or_default();
        let response =
            craft_ui::tabs::Tab::new(egui::Id::new(("freerusch-tab", index)), label, is_selected)
                .show_at(ui, rect, |ui, response| {
                    let visuals = ui.visuals();
                    let fill = if is_selected {
                        visuals.selection.bg_fill
                    } else if response.hovered() {
                        visuals.widgets.hovered.bg_fill
                    } else {
                        visuals.widgets.inactive.bg_fill
                    };
                    let text_color = if is_selected {
                        visuals.selection.stroke.color
                    } else {
                        visuals.widgets.inactive.fg_stroke.color
                    };
                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::same(4), fill);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        font.clone(),
                        text_color,
                    );
                });
        if response.clicked() && !is_selected {
            *selected = index;
            changed = true;
        }
        x += width;
    }

    changed
}

impl eframe::App for FreeruschApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("FreeRusch");
        ui.label("Lajes de pontes pelo método de Rüsch — tabelas DIN/IBSW (NBR 7188:2013).");
        ui.separator();

        // Barra de ferramentas do projeto (.frus).
        ui.horizontal(|ui| {
            ui.label("Projeto:");
            let mut path = self.settings.get().general.last_project_path.clone();
            let response = ui.add(
                egui::TextEdit::singleline(&mut path)
                    .desired_width(220.0)
                    .hint_text("projeto.frus"),
            );
            if response.changed() {
                self.settings
                    .update(|s| s.general.last_project_path = path.clone());
            }
            if ui.button("Abrir").clicked() {
                self.open_project();
            }
            if ui.button("Salvar").clicked() {
                self.save_project();
            }
        });

        if tab_bar(ui, &mut self.tab) {
            self.message.clear();
        }
        ui.separator();

        match self.tab {
            TAB_INPUTS => self.inputs_tab(ui),
            TAB_RESULTS => self.results_tab(ui),
            TAB_REPORT => self.report_tab(ui),
            _ => {}
        }

        ui.separator();
        if !self.message.is_empty() {
            ui.label(&self.message);
        }

        // Grava as configurações alteradas (debounce da própria craft-settings).
        let now = std::time::Instant::now();
        let tick = self.settings.tick(now);
        if let Some(deadline) = tick.next_deadline {
            ui.ctx()
                .request_repaint_after(deadline.saturating_duration_since(now));
        }
        // Última gravação antes de a janela fechar.
        if ui.ctx().input(|i| i.viewport().close_requested()) {
            let _ = self.settings.flush();
        }
    }
}

impl FreeruschApp {
    /// Aba "Inputs": tabela de Rüsch e cargas.
    fn inputs_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Título:");
            ui.add(
                egui::TextEdit::singleline(&mut self.project.title)
                    .desired_width(280.0)
                    .hint_text("Viaduto Centro — Painel P1"),
            );
        });

        ui.add_space(4.0);
        ui.label("Tabela de Rüsch");
        ui.horizontal(|ui| {
            ui.label("Nr:");
            ui.add(egui::DragValue::new(&mut self.project.table_number).range(1..=102));
            let mut variant = match self.project.table_variant {
                Some(1) => 0,
                Some(2) => 1,
                _ => 2,
            };
            egui::ComboBox::from_label("Variante")
                .selected_text(match variant {
                    0 => ".1",
                    1 => ".2",
                    _ => "(sem variante)",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut variant, 0, ".1");
                    ui.selectable_value(&mut variant, 1, ".2");
                    ui.selectable_value(&mut variant, 2, "(sem variante)");
                });
            self.project.table_variant = match variant {
                0 => Some(1),
                1 => Some(2),
                _ => None,
            };
            match self.project.table_label() {
                Some(label) => ui.label(format!("→ {label}")),
                None => ui.colored_label(
                    egui::Color32::from_rgb(170, 40, 40),
                    "tabela inexistente no acervo",
                ),
            };
        });

        ui.add_space(4.0);
        ui.label("Geometria");
        ui.horizontal(|ui| {
            ui.label("Vão lx (m):");
            ui.add(
                egui::DragValue::new(&mut self.project.lx_m)
                    .range(1.0..=40.0)
                    .speed(0.05)
                    .fixed_decimals(2),
            );
            ui.label("Laje h (m):");
            ui.add(
                egui::DragValue::new(&mut self.project.slab_m)
                    .range(0.10..=1.00)
                    .speed(0.005)
                    .fixed_decimals(3),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Pavimento (m):");
            ui.add(
                egui::DragValue::new(&mut self.project.pavement_m)
                    .range(0.0..=0.20)
                    .speed(0.005)
                    .fixed_decimals(3),
            );
            ui.label("Distância entre rodas a (m):");
            ui.add(
                egui::DragValue::new(&mut self.project.wheel_spacing_m)
                    .range(0.5..=5.0)
                    .speed(0.05)
                    .fixed_decimals(2),
            );
        });

        ui.add_space(4.0);
        ui.label("Cargas");
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Trem-tipo")
                .selected_text(self.project.train.as_str())
                .show_ui(ui, |ui| {
                    for train in [TrainType::TB450, TrainType::TB240] {
                        ui.selectable_value(
                            &mut self.project.train,
                            train.label().to_string(),
                            train.label(),
                        );
                    }
                });
            if ui.button("Aplicar padrões do trem-tipo").clicked()
                && let Ok(train) = self.project.train_type()
            {
                let (wheel, uniform) = train_defaults(train);
                self.project.wheel_load_kn = wheel;
                self.project.uniform_load_kn_m2 = uniform;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Permanente g (kN/m²):");
            ui.add(
                egui::DragValue::new(&mut self.project.permanent_load_kn_m2)
                    .range(0.5..=100.0)
                    .speed(0.1)
                    .fixed_decimals(2),
            );
            ui.label("Roda P (kN):");
            ui.add(
                egui::DragValue::new(&mut self.project.wheel_load_kn)
                    .range(5.0..=250.0)
                    .speed(1.0)
                    .fixed_decimals(1),
            );
            ui.label("Uniforme q (kN/m²):");
            ui.add(
                egui::DragValue::new(&mut self.project.uniform_load_kn_m2)
                    .range(0.5..=30.0)
                    .speed(0.1)
                    .fixed_decimals(1),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Concreto:");
            ui.add(
                egui::TextEdit::singleline(&mut self.project.concrete_class).desired_width(90.0),
            );
            ui.label("(ex.: C25/30, C30/37)");
        });

        ui.add_space(8.0);
        if ui.button("Calcular").clicked() {
            self.calculate();
        }

        if let Ok(train) = self.project.train_type() {
            self.settings
                .update(|s| s.general.train = train.label().to_string());
        }
    }

    /// Aba "Results": momentos característicos e de projeto.
    fn results_tab(&mut self, ui: &mut egui::Ui) {
        if self.moments.is_empty() {
            ui.label("Ainda sem resultados — preencha a aba Inputs e clique em Calcular.");
            return;
        }
        let impact = freerusch_core::impact_coefficient(self.project.lx_m);
        let spread =
            freerusch_core::load_spread_width_m(self.project.pavement_m, self.project.slab_m);
        ui.horizontal(|ui| {
            if let Ok(value) = impact {
                ui.label(format!("φ = {value:.4}"));
            }
            if let Ok(value) = spread {
                ui.label(format!("t = {value:.4} m"));
            }
            ui.label(format!(
                "lx/a = {:.3}",
                self.project.lx_m / self.project.wheel_spacing_m
            ));
        });
        ui.add_space(4.0);

        egui::Grid::new("freerusch-results")
            .num_columns(4)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Resultante");
                ui.label("M_g (kN·m/m)");
                ui.label("M_q (kN·m/m)");
                ui.label("M_d (kN·m/m)");
                ui.end_row();
                for moment in &self.moments {
                    ui.label(&moment.resultant);
                    ui.label(format!("{:.3}", moment.permanent));
                    ui.label(format!("{:.3}", moment.live));
                    ui.strong(format!("{:.3}", moment.design));
                    ui.end_row();
                }
            });
        ui.add_space(6.0);
        ui.label(
            "Coeficientes de ponderação: γg = 1,35 e γq = 1,5. \
             Nas resultantes de borda (Mxe/Mye) a tração é na face superior.",
        );
    }

    /// Aba "Report": exportação da memória de cálculo.
    fn report_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Arquivo HTML:");
            let mut path = self.settings.get().general.last_report_path.clone();
            let response = ui.add(
                egui::TextEdit::singleline(&mut path)
                    .desired_width(260.0)
                    .hint_text("memoria-de-calculo.html"),
            );
            if response.changed() {
                self.settings
                    .update(|s| s.general.last_report_path = path.clone());
            }
            if ui.button("Exportar HTML").clicked() {
                self.export_report();
            }
        });
        ui.add_space(8.0);
        if self.report_html.is_empty() {
            ui.label("A memória de cálculo é gerada ao clicar em Calcular.");
            return;
        }
        ui.label(format!(
            "Documento autocontido, {} bytes, declarando tabela, hipóteses e coeficientes.",
            self.report_html.len()
        ));
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::Label::new(egui::RichText::new(self.report_html.as_str()).monospace()).wrap(),
            );
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([820.0, 600.0])
            .with_title("FreeRusch"),
        ..Default::default()
    };
    eframe::run_native(
        "FreeRusch",
        options,
        Box::new(|cc| Ok(Box::new(FreeruschApp::new(cc)))),
    )
}
