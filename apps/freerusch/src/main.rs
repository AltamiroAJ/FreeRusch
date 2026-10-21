//! FreeRusch — cálculo estrutural de lajes de pontes pelo método de Rüsch.
//!
//! GUI sobre `eframe`/`egui` com três abas: **Inputs** (entrada do painel),
//! **Results** (coeficientes, momentos e diagrama) e **Report** (memória de
//! cálculo em HTML). A barra de abas usa os primitivos de `craft_ui::tabs`
//! (`fit` para larguras, `Tab::show_at` para interação e acessibilidade), já
//! que a `craft-ui` não fornece um contêiner de abas pronto.
//!
//! Persistência: preferências em `config.toml` via `craft-settings`; o projeto
//! completo em `.frus` via `freerusch-project` (JSON versionado, escrita
//! atômica); a memória de cálculo em HTML via `freerusch-report`.

use std::path::Path;

use eframe::egui;
use freerusch_core::{
    EdgeCondition, MomentDiagram, MomentResult, RuschTable, TrainType,
    concrete_characteristic_strength, design_moment,
};
use freerusch_project::{EDGE_CLAMPED, EDGE_SIMPLY_SUPPORTED, Project};
use serde::{Deserialize, Serialize};

/// Rótulos das abas, na ordem de exibição.
const TABS: [&str; 3] = ["Inputs", "Results", "Report"];
const TAB_INPUTS: usize = 0;
const TAB_RESULTS: usize = 1;
const TAB_REPORT: usize = 2;

/// Número de pontos usados no diagrama exibido na GUI.
const DIAGRAM_SAMPLES: usize = 81;

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

/// Estado da aplicação.
struct FreeruschApp {
    settings: craft_settings::SettingsStore<FreeruschSettings>,
    project: Project,
    tab: usize,
    result: Option<MomentResult>,
    diagram: MomentDiagram,
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
            result: None,
            diagram: MomentDiagram { points: Vec::new() },
            report_html: String::new(),
            message: String::new(),
        };
        if report.has_problems() {
            app.message =
                "Configurações não puderam ser lidas por completo; usando padrões.".into();
        }
        app
    }

    /// Recalcula coeficientes, momentos, diagrama e a memória de cálculo.
    fn calculate(&mut self) {
        self.result = None;
        let train = match self.project.train_type() {
            Ok(t) => t,
            Err(e) => {
                self.message = format!("Trem-tipo inválido: {e}");
                return;
            }
        };
        let edge = match self.project.edge_condition() {
            Ok(e) => e,
            Err(e) => {
                self.message = format!("Condição de borda inválida: {e}");
                return;
            }
        };
        let fck = match concrete_characteristic_strength(&self.project.concrete_class) {
            Ok(f) => f,
            Err(e) => {
                self.message = format!("Concreto inválido: {e}");
                return;
            }
        };
        let table = RuschTable::for_edges(edge);
        match design_moment(
            &table,
            train,
            self.project.spans,
            self.project.l1_m,
            self.project.l2_m,
            self.project.deck_width_m,
            fck,
        ) {
            Ok(moment) => {
                self.diagram =
                    MomentDiagram::for_result(&moment, self.project.l1_m, DIAGRAM_SAMPLES);
                self.report_html =
                    freerusch_report::render_html(&self.slab_input(train, edge), &moment);
                self.result = Some(moment);
                self.message = format!(
                    "{} · {:.2} m → M⁺ = {:.2} kN·m/m (α = {:.4})",
                    train.label(),
                    self.project.l1_m,
                    moment.max_positive,
                    moment.alpha
                );
                self.tab = TAB_RESULTS;
            }
            Err(e) => self.message = format!("Erro de cálculo: {e}"),
        }
    }

    /// Monta o `SlabInput` do relatório a partir do estado atual.
    fn slab_input(&self, train: TrainType, _edge: EdgeCondition) -> freerusch_report::SlabInput {
        freerusch_report::SlabInput::new(
            if self.project.title.is_empty() {
                "FreeRusch — painel corrente"
            } else {
                &self.project.title
            },
            train,
            self.project.l1_m,
            self.project.l2_m,
            self.project.deck_width_m,
            &self.project.concrete_class,
        )
    }

    /// Exporta a memória de cálculo em HTML com escrita atômica.
    fn export_report(&mut self) {
        if self.result.is_none() {
            self.calculate();
        }
        if self.report_html.is_empty() {
            self.message = "Nada a exportar: calcule primeiro.".into();
            return;
        }
        let path = self.settings.get().general.last_report_path.clone();
        match freerusch_report::write_report(Path::new(&path), &self.report_html) {
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

/// Desenha o diagrama de momentos no espaço disponível.
fn draw_diagram(ui: &mut egui::Ui, diagram: &MomentDiagram) {
    let size = egui::vec2(ui.available_width(), 180.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        rect,
        egui::CornerRadius::same(4),
        ui.visuals().extreme_bg_color,
    );

    let mid_y = rect.center().y;
    painter.line_segment(
        [
            egui::pos2(rect.left(), mid_y),
            egui::pos2(rect.right(), mid_y),
        ],
        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.fg_stroke.color),
    );

    let span = diagram.max_abs_moment();
    let x_last = diagram.points.last().map(|p| p.0).unwrap_or(0.0);
    if !span.is_finite() || span <= 0.0 || !x_last.is_finite() || x_last <= 0.0 {
        return;
    }

    let half_height = rect.height() * 0.45;
    let mut points: Vec<egui::Pos2> = Vec::with_capacity(diagram.points.len());
    for &(x, m) in &diagram.points {
        if !(x.is_finite() && m.is_finite()) {
            continue;
        }
        points.push(egui::pos2(
            rect.left() + (x / x_last) as f32 * rect.width(),
            mid_y - (m / span) as f32 * half_height,
        ));
    }
    if points.len() >= 2 {
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 60, 60)),
        ));
    }
}

impl eframe::App for FreeruschApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("FreeRusch");
        ui.label("Cálculo de lajes de pontes pelo método de Rüsch (NBR 7188:2013).");
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
    /// Aba "Inputs": formulário do painel.
    fn inputs_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Título:");
            ui.add(
                egui::TextEdit::singleline(&mut self.project.title)
                    .desired_width(280.0)
                    .hint_text("Viaduto Centro — Painel P1"),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Vão l₁ (m):");
            ui.add(
                egui::DragValue::new(&mut self.project.l1_m)
                    .range(1.0..=40.0)
                    .speed(0.1),
            );
            ui.label("Vão vizinho l₂ (m):");
            ui.add(
                egui::DragValue::new(&mut self.project.l2_m)
                    .range(0.0..=40.0)
                    .speed(0.1),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Largura do tabuleiro b (m):");
            ui.add(
                egui::DragValue::new(&mut self.project.deck_width_m)
                    .range(3.0..=30.0)
                    .speed(0.1),
            );
            ui.label("Nº de vãos:");
            ui.add(egui::DragValue::new(&mut self.project.spans).range(1..=4));
        });
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
            egui::ComboBox::from_label("Condição de borda")
                .selected_text(edge_label(&self.project.edge))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.project.edge,
                        EDGE_SIMPLY_SUPPORTED.to_string(),
                        "Biapoiada",
                    );
                    ui.selectable_value(
                        &mut self.project.edge,
                        EDGE_CLAMPED.to_string(),
                        "Engastada",
                    );
                });
        });
        ui.horizontal(|ui| {
            ui.label("Concreto:");
            ui.add(
                egui::TextEdit::singleline(&mut self.project.concrete_class).desired_width(90.0),
            );
            ui.label("(ex.: C25/30, C30/37, C40/50)");
        });
        ui.add_space(8.0);
        if ui.button("Calcular").clicked() {
            self.calculate();
        }
        if let Ok(tab) = self.project.train_type() {
            self.settings
                .update(|s| s.general.train = tab.label().to_string());
        }
    }

    /// Aba "Results": coeficientes, momentos e diagrama.
    fn results_tab(&mut self, ui: &mut egui::Ui) {
        let Some(moment) = self.result else {
            ui.label("Ainda sem resultados — preencha a aba Inputs e clique em Calcular.");
            return;
        };
        if freerusch_core::RUSCH_TABLES_PRELIMINARY {
            ui.colored_label(
                egui::Color32::from_rgb(160, 60, 60),
                "Tabelas de Rüsch preliminares — resultados não homologados para projeto executivo.",
            );
        }
        egui::Grid::new("freerusch-results")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("α (positivo)");
                ui.label(format!("{:.4}", moment.alpha));
                ui.end_row();
                ui.label("β (negativo)");
                ui.label(
                    moment
                        .beta
                        .map(|b| format!("{b:.4}"))
                        .unwrap_or_else(|| "—".to_string()),
                );
                ui.end_row();
                ui.label("M⁺ vão (kN·m/m)");
                ui.label(format!("{:.3}", moment.max_positive));
                ui.end_row();
                ui.label("M⁻ apoio (kN·m/m)");
                ui.label(format!("{:.3}", moment.max_negative));
                ui.end_row();
            });
        ui.add_space(8.0);
        draw_diagram(ui, &self.diagram);
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
            "Documento autocontido, {} bytes, com diagrama SVG embutido.",
            self.report_html.len()
        ));
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::Label::new(egui::RichText::new(self.report_html.as_str()).monospace()).wrap(),
            );
        });
    }
}

/// Rótulo em português para a condição de borda gravada no projeto.
fn edge_label(edge: &str) -> &'static str {
    if edge.eq_ignore_ascii_case(EDGE_CLAMPED) {
        "Engastada"
    } else if edge.eq_ignore_ascii_case(EDGE_SIMPLY_SUPPORTED) {
        "Biapoiada"
    } else {
        "Desconhecida"
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([760.0, 560.0])
            .with_title("FreeRusch"),
        ..Default::default()
    };
    eframe::run_native(
        "FreeRusch",
        options,
        Box::new(|cc| Ok(Box::new(FreeruschApp::new(cc)))),
    )
}
