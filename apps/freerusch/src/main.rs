//! FreeRusch — cálculo estrutural de lajes de pontes pelo método de Rüsch.
//!
//! GUI sobre `eframe`/`egui` com três abas: **Inputs** (esquema, parâmetros e
//! tabela de coeficientes), **Results** (momentos característicos e de projeto)
//! e **Report** (memória de cálculo em HTML).
//!
//! A organização da aba Inputs segue o fluxo que um engenheiro de pontes
//! reconhece: desenho esquemático à esquerda, parâmetros à direita, barra de
//! ações, tabela de coeficientes com a interpolação em uso destacada e painel de
//! resultados. O desenho é vetorial (`egui::Painter`) e reage aos campos.
//!
//! Todo valor de engenharia vem de `freerusch-core`; a UI não calcula nada. As
//! cores vêm de [`theme::pal`], derivadas do tema ativo.

mod schematic;
mod theme;

use std::path::Path;

use eframe::egui::{
    self, Align, Align2, CornerRadius, Layout, RichText, Sense, Stroke, TextStyle, Vec2, pos2, vec2,
};
use freerusch_core::{
    DesignMoment, RuschDesignInput, RuschTableData, TrainType, WHEEL_SPACING_M, design_moments,
    impact_coefficient, interpolated_uniform_coefficient, interpolated_wheel_coefficient,
    load_spread_width_m, panel_coefficient, uniform_column, wheel_columns,
};
use freerusch_project::Project;
use freerusch_report::{ReportContext, render_html, write_report};
use schematic::{EdgeKind, Sketch, TrafficDirection};
use serde::{Deserialize, Serialize};

/// Rótulos das abas, na ordem de exibição.
const TABS: [&str; 3] = ["Inputs", "Results", "Report"];
const TAB_INPUTS: usize = 0;
const TAB_RESULTS: usize = 1;
const TAB_REPORT: usize = 2;

/// Largura do painel de resultados.
const RESULTS_WIDTH: f32 = 300.0;
/// Altura de uma linha da tabela de coeficientes.
const TABLE_ROW_HEIGHT: f32 = 18.0;
/// Largura da primeira coluna (rótulo `lx/a`).
const TABLE_LABEL_WIDTH: f32 = 62.0;
/// Largura das colunas de coeficientes.
const TABLE_CELL_WIDTH: f32 = 68.0;

/// Esforço exibido na tabela e no painel de resultados.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Resultant {
    Mxm,
    Mym,
    Mxe,
}

impl Resultant {
    const ALL: [Resultant; 3] = [Resultant::Mxm, Resultant::Mym, Resultant::Mxe];

    fn key(self) -> &'static str {
        match self {
            Resultant::Mxm => "Mxm",
            Resultant::Mym => "Mym",
            Resultant::Mxe => "Mxe",
        }
    }

    fn label(self) -> &'static str {
        self.key()
    }
}

/// Estado transitório da interface (não faz parte do `.frus`).
struct UiState {
    resultant: Resultant,
    edges: [EdgeKind; 4],
    traffic: TrafficDirection,
    /// Entrada usada no último cálculo, para saber se o resultado está atualizado.
    computed_from: Option<RuschDesignInput>,
    show_paths: bool,
    show_about: bool,
}

impl Default for UiState {
    fn default() -> Self {
        UiState {
            resultant: Resultant::Mxm,
            edges: [EdgeKind::Undefined; 4],
            traffic: TrafficDirection::Vertical,
            computed_from: None,
            show_paths: false,
            show_about: false,
        }
    }
}

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
    /// Tema preferido: "system", "dark" ou "light".
    theme: String,
}

impl Default for General {
    fn default() -> Self {
        General {
            train: TrainType::TB450.label().to_string(),
            last_project_path: "projeto.frus".to_string(),
            last_report_path: "memoria-de-calculo.html".to_string(),
            theme: "system".to_string(),
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

/// Converte o rótulo persistido em preferência de tema do egui.
fn theme_preference(label: &str) -> egui::ThemePreference {
    match label {
        "dark" => egui::ThemePreference::Dark,
        "light" => egui::ThemePreference::Light,
        _ => egui::ThemePreference::System,
    }
}

fn theme_label(preference: egui::ThemePreference) -> &'static str {
    match preference {
        egui::ThemePreference::Dark => "dark",
        egui::ThemePreference::Light => "light",
        egui::ThemePreference::System => "system",
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
    ui_state: UiState,
}

impl FreeruschApp {
    /// Abre a aplicação carregando as configurações do usuário.
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Em testes/CI sem perfil, mantém as configurações apenas em memória.
        let location = if std::env::var_os("FREERUSCH_NO_PREFS").is_some() {
            craft_settings::Location::Memory
        } else {
            craft_settings::Location::Auto
        };
        let (store, report) = craft_settings::SettingsStore::<FreeruschSettings>::open(location);

        // Tema: variável de ambiente (útil para captura automatizada) > config.
        let env_theme = std::env::var("FREERUSCH_THEME").ok();
        let preference = theme_preference(
            env_theme
                .as_deref()
                .unwrap_or(store.get().general.theme.as_str()),
        );
        cc.egui_ctx.set_theme(preference);
        theme::configure(&cc.egui_ctx);

        let project = Project {
            train: store.get().general.train.clone(),
            wheel_spacing_m: WHEEL_SPACING_M,
            ..Project::default()
        };
        let mut app = FreeruschApp {
            settings: store,
            project,
            tab: TAB_INPUTS,
            moments: Vec::new(),
            report_html: String::new(),
            message: String::new(),
            ui_state: UiState::default(),
        };
        if report.has_problems() {
            app.message =
                "Configurações não puderam ser lidas por completo; usando padrões.".to_string();
        }
        // O exemplo do workbook já vem calculado: a tela abre com números, não
        // com campos vazios, e o usuário vê o efeito de cada edição no estado.
        app.calculate();
        app
    }

    /// Entrada de cálculo correspondente ao estado atual do formulário.
    fn input(&self) -> RuschDesignInput {
        self.project.design_input()
    }

    /// `true` quando algum campo mudou depois do último cálculo.
    fn stale(&self) -> bool {
        match self.ui_state.computed_from.as_ref() {
            Some(previous) => *previous != self.input(),
            None => false,
        }
    }

    /// Recalcula os momentos característicos e de projeto.
    fn calculate(&mut self) {
        self.message.clear();
        if let Err(error) = self.project.validate() {
            self.moments.clear();
            self.report_html.clear();
            self.ui_state.computed_from = None;
            self.message = format!("Projeto inválido: {error}");
            return;
        }

        let input = self.input();
        match design_moments(&input) {
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
                if let Some(selected) = moments
                    .iter()
                    .find(|moment| moment.resultant == self.ui_state.resultant.key())
                {
                    self.message = format!(
                        "{label} · M_d({}) = {:.2} kN·m/m",
                        self.ui_state.resultant.label(),
                        selected.design
                    );
                } else {
                    self.message = format!("Cálculo concluído com {label}.");
                }
                self.moments = moments;
                self.ui_state.computed_from = Some(input);
            }
            Err(error) => {
                self.moments.clear();
                self.report_html.clear();
                self.ui_state.computed_from = None;
                self.message = format!("Erro de cálculo: {error}");
            }
        }
    }

    /// Momento da resultante selecionada, se houver cálculo.
    fn selected_moment(&self) -> Option<&DesignMoment> {
        self.moments
            .iter()
            .find(|moment| moment.resultant == self.ui_state.resultant.key())
    }

    /// Tabela de Rüsch escolhida.
    fn table(&self) -> Option<&'static RuschTableData> {
        RuschTableData::find(self.project.table_number, self.project.table_variant)
    }

    /// Exporta a memória de cálculo em HTML com escrita atômica.
    fn export_report(&mut self) {
        if self.report_html.is_empty() {
            self.calculate();
        }
        if self.report_html.is_empty() {
            self.message = "Nada a exportar: calcule primeiro.".to_string();
            return;
        }
        let path = self.settings.get().general.last_report_path.clone();
        match write_report(Path::new(&path), &self.report_html) {
            Ok(()) => self.message = format!("Memória escrita em {path}"),
            Err(error) => self.message = format!("Falha na exportação: {error}"),
        }
    }

    /// Grava o projeto atual em `.frus`.
    fn save_project(&mut self) {
        let path = self.settings.get().general.last_project_path.clone();
        match self.project.save(Path::new(&path)) {
            Ok(()) => self.message = format!("Projeto salvo em {path}"),
            Err(error) => self.message = format!("Falha ao salvar projeto: {error}"),
        }
    }

    /// Carrega um projeto `.frus`, substituindo o estado atual em caso de êxito.
    fn open_project(&mut self) {
        let path = self.settings.get().general.last_project_path.clone();
        match Project::load(Path::new(&path)) {
            Ok(mut project) => {
                // `a` é a convenção das tabelas, não um campo do usuário.
                project.wheel_spacing_m = WHEEL_SPACING_M;
                self.project = project;
                self.settings
                    .update(|settings| settings.general.train = self.project.train.clone());
                self.calculate();
                self.tab = TAB_INPUTS;
                self.message = format!("Projeto carregado de {path}");
            }
            Err(error) => self.message = format!("Falha ao abrir projeto: {error}"),
        }
    }

    /// Volta o formulário ao exemplo resolvido do workbook.
    fn reset_project(&mut self) {
        self.project = Project::default();
        self.moments.clear();
        self.report_html.clear();
        self.ui_state = UiState {
            resultant: self.ui_state.resultant,
            edges: [EdgeKind::Undefined; 4],
            traffic: self.ui_state.traffic,
            computed_from: None,
            show_paths: self.ui_state.show_paths,
            show_about: self.ui_state.show_about,
        };
        self.message.clear();
    }

    /// Barra de menus: Arquivo, Opções, Resultados, Sobre.
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let mut saved_theme = None;
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Arquivo", |ui| {
                if ui.button("Novo (exemplo do workbook)").clicked() {
                    self.reset_project();
                    ui.close();
                }
                if ui.button("Abrir").clicked() {
                    self.open_project();
                    ui.close();
                }
                if ui.button("Salvar").clicked() {
                    self.save_project();
                    ui.close();
                }
                if ui.button("Exportar memória de cálculo").clicked() {
                    self.export_report();
                    ui.close();
                }
                ui.separator();
                if ui.button("Caminhos…").clicked() {
                    self.ui_state.show_paths = true;
                    ui.close();
                }
                ui.separator();
                if ui.button("Sair").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close();
                }
            });
            ui.menu_button("Opções", |ui| {
                ui.label("Tema");
                let current = theme_preference(self.settings.get().general.theme.as_str());
                for preference in [
                    egui::ThemePreference::System,
                    egui::ThemePreference::Light,
                    egui::ThemePreference::Dark,
                ] {
                    let label = match preference {
                        egui::ThemePreference::System => "Sistema",
                        egui::ThemePreference::Light => "Claro",
                        egui::ThemePreference::Dark => "Escuro",
                    };
                    if ui.selectable_label(current == preference, label).clicked() {
                        saved_theme = Some(preference);
                        ui.close();
                    }
                }
            });
            ui.menu_button("Resultados", |ui| {
                for resultant in Resultant::ALL {
                    if ui
                        .selectable_label(self.ui_state.resultant == resultant, resultant.label())
                        .clicked()
                    {
                        self.ui_state.resultant = resultant;
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Ir para a aba Results").clicked() {
                    self.tab = TAB_RESULTS;
                    ui.close();
                }
            });
            ui.menu_button("Sobre", |ui| {
                if ui.button("Sobre o FreeRusch").clicked() {
                    self.ui_state.show_about = true;
                    ui.close();
                }
            });
        });

        if let Some(preference) = saved_theme {
            ui.ctx().set_theme(preference);
            self.settings
                .update(|settings| settings.general.theme = theme_label(preference).to_string());
        }
    }

    /// Aba "Inputs": esquema, parâmetros, tabela de coeficientes e resultados.
    fn inputs_tab(&mut self, ui: &mut egui::Ui) {
        let pal = theme::pal(ui);
        // A largura é fixada antes da área rolável: dentro dela `available_width`
        // passa a depender do próprio conteúdo e as colunas encolhem a cada frame.
        let full_width = ui.available_width();
        egui::ScrollArea::vertical()
            .id_salt("inputs-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(full_width);
                self.inputs_content(ui, pal);
            });
    }

    /// Conteúdo rolável da aba Inputs.
    fn inputs_content(&mut self, ui: &mut egui::Ui, pal: theme::Pal) {
        let width = ui.available_width();
        let gap = 14.0;
        let left_width = (width * 0.40).clamp(340.0, 620.0);
        let right_width = (width - left_width - gap).max(360.0);
        let top_height = schematic::HEIGHT + 44.0;

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                vec2(left_width, top_height),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.label(RichText::new("Seção transversal").strong());
                    self.schematic_panel(ui, pal);
                },
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                vec2(right_width, top_height),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.label(RichText::new("Parâmetros").strong());
                    self.parameters_panel(ui, pal);
                },
            );
        });

        ui.add_space(6.0);
        self.action_bar(ui);

        ui.add_space(6.0);
        let table_width = (ui.available_width() - RESULTS_WIDTH - gap).max(360.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                vec2(table_width, 360.0),
                Layout::top_down(Align::Min),
                |ui| {
                    self.coefficient_table(ui, pal);
                },
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                vec2(RESULTS_WIDTH, 360.0),
                Layout::top_down(Align::Min),
                |ui| {
                    self.results_panel(ui, pal);
                },
            );
        });

        ui.add_space(6.0);
    }

    /// Desenho esquemático da seção.
    fn schematic_panel(&self, ui: &mut egui::Ui, pal: theme::Pal) {
        let moment_label = self
            .selected_moment()
            .map(|moment| format!("{} = {:.2} kN·m/m", moment.resultant, moment.design));
        let sketch = Sketch {
            lx_m: self.project.lx_m,
            slab_m: self.project.slab_m,
            pavement_m: self.project.pavement_m,
            wheel_spacing_m: self.project.wheel_spacing_m,
            edges: self.ui_state.edges,
            traffic: self.ui_state.traffic,
            moment_label: moment_label.as_deref(),
        };
        egui::Frame::NONE
            .fill(pal.SURFACE)
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| schematic::show(ui, &sketch));
    }

    /// Painel de parâmetros: tabela, dimensões, vinculações, tráfego e φ.
    fn parameters_panel(&mut self, ui: &mut egui::Ui, pal: theme::Pal) {
        ui.spacing_mut().item_spacing.y = 4.0;

        // Tabela de Rüsch.
        ui.horizontal(|ui| {
            ui.label("Tabela Nr");
            ui.add(egui::DragValue::new(&mut self.project.table_number).range(1..=102));
            let mut variant_index = match self.project.table_variant {
                Some(1) => 0,
                Some(2) => 1,
                _ => 2,
            };
            egui::ComboBox::from_id_salt("variant")
                .selected_text(match variant_index {
                    0 => ".1",
                    1 => ".2",
                    _ => "—",
                })
                .width(56.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut variant_index, 0, ".1");
                    ui.selectable_value(&mut variant_index, 1, ".2");
                    ui.selectable_value(&mut variant_index, 2, "sem variante");
                });
            self.project.table_variant = match variant_index {
                0 => Some(1),
                1 => Some(2),
                _ => None,
            };
            match self.table() {
                Some(table) => {
                    ui.label(
                        RichText::new(table.sheet.clone())
                            .monospace()
                            .color(pal.ACCENT),
                    );
                }
                None => {
                    ui.colored_label(pal.DANGER, "tabela inexistente");
                }
            }
        });

        ui.add_space(2.0);
        ui.label(RichText::new("Dimensões").strong());
        ui.horizontal(|ui| {
            ui.label("lx (m)");
            ui.add(
                egui::DragValue::new(&mut self.project.lx_m)
                    .range(1.0..=40.0)
                    .speed(0.05)
                    .fixed_decimals(2),
            );
            ui.label("ly");
            let ly_text = match self.table() {
                Some(table) if table.ratio_is_infinite => "∞".to_string(),
                Some(table) => match table.ratio {
                    Some(ratio) => format!("{:.2} m", ratio * self.project.lx_m),
                    None => "—".to_string(),
                },
                None => "—".to_string(),
            };
            read_only(ui, pal, &ly_text, 96.0);
        });
        ui.horizontal(|ui| {
            ui.label("h (m)");
            ui.add(
                egui::DragValue::new(&mut self.project.slab_m)
                    .range(0.10..=1.00)
                    .speed(0.005)
                    .fixed_decimals(3),
            );
            ui.label("a (m)").on_hover_text(
                "Distância transversal entre rodas: convenção das tabelas de Rüsch.",
            );
            read_only(ui, pal, &format!("{WHEEL_SPACING_M:.2}"), 96.0);
        });

        ui.add_space(2.0);
        ui.label(RichText::new("Vinculações").strong())
            .on_hover_text(
                "Descrevem o desenho: a configuração estrutural vem da tabela Nr escolhida.",
            );
        let names = ["Superior", "Inferior", "Esquerda", "Direita"];
        egui::Grid::new("edges-grid")
            .num_columns(2)
            .spacing([10.0, 2.0])
            .show(ui, |ui| {
                for (index, name) in names.iter().enumerate() {
                    ui.label(*name);
                    let mut kind = self.ui_state.edges[index];
                    egui::ComboBox::from_id_salt(("edge", index))
                        .selected_text(kind.label())
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            for option in EdgeKind::ALL {
                                ui.selectable_value(&mut kind, option, option.label());
                            }
                        });
                    self.ui_state.edges[index] = kind;
                    if index % 2 == 1 {
                        ui.end_row();
                    }
                }
            });

        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Tráfego").strong());
            for direction in [TrafficDirection::Vertical, TrafficDirection::Horizontal] {
                ui.radio_value(&mut self.ui_state.traffic, direction, direction.label());
            }
            ui.separator();
            ui.label(RichText::new("φ").strong())
                .on_hover_text("Coeficiente de impacto vertical, calculado pelo motor.");
            let impact = impact_coefficient(self.project.lx_m)
                .map(|value| format!("{value:.4}"))
                .unwrap_or_else(|_| "—".to_string());
            read_only(ui, pal, &impact, 80.0);
        });

        ui.add_space(2.0);
        ui.label(RichText::new("Cargas").strong());
        ui.horizontal(|ui| {
            ui.label("g");
            ui.add(
                egui::DragValue::new(&mut self.project.permanent_load_kn_m2)
                    .range(0.5..=100.0)
                    .speed(0.1)
                    .fixed_decimals(2),
            );
            ui.label("P");
            ui.add(
                egui::DragValue::new(&mut self.project.wheel_load_kn)
                    .range(5.0..=250.0)
                    .speed(1.0)
                    .fixed_decimals(1),
            );
            ui.label("q");
            ui.add(
                egui::DragValue::new(&mut self.project.uniform_load_kn_m2)
                    .range(0.5..=30.0)
                    .speed(0.1)
                    .fixed_decimals(1),
            );
        });
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("train")
                .selected_text(self.project.train.as_str())
                .width(96.0)
                .show_ui(ui, |ui| {
                    for train in [TrainType::TB450, TrainType::TB240] {
                        ui.selectable_value(
                            &mut self.project.train,
                            train.label().to_string(),
                            train.label(),
                        );
                    }
                });
            if ui.button("Padrões do trem-tipo").clicked()
                && let Ok(train) = self.project.train_type()
            {
                let (wheel, uniform) = train_defaults(train);
                self.project.wheel_load_kn = wheel;
                self.project.uniform_load_kn_m2 = uniform;
            }
        });

        // Mantém o rótulo do trem-tipo persistido em sincronia.
        if let Ok(train) = self.project.train_type() {
            self.settings
                .update(|settings| settings.general.train = train.label().to_string());
        }
    }

    /// Barra de ações: Calcular e alternadores de esforço.
    fn action_bar(&mut self, ui: &mut egui::Ui) {
        let pal = theme::pal(ui);
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [120.0, 26.0],
                    egui::Button::new(RichText::new("Calcular").strong()),
                )
                .clicked()
            {
                self.calculate();
            }
            ui.separator();
            ui.label("Esforço:");
            for resultant in Resultant::ALL {
                let selected = self.ui_state.resultant == resultant;
                if ui
                    .selectable_label(
                        selected,
                        RichText::new(resultant.label())
                            .monospace()
                            .color(if selected { pal.ACCENT } else { pal.TEXT }),
                    )
                    .clicked()
                {
                    self.ui_state.resultant = resultant;
                }
            }
        });
    }

    /// Tabela de coeficientes com a interpolação em uso destacada.
    fn coefficient_table(&self, ui: &mut egui::Ui, pal: theme::Pal) {
        // `allocate_ui_with_layout` aloca só a largura usada: sem isto a coluna
        // da tabela encolhe e o painel de resultados sobe para o meio da tela.
        ui.set_min_width(ui.available_width());
        let Some(table) = self.table() else {
            ui.colored_label(pal.DANGER, "Selecione uma tabela de Rüsch existente.");
            return;
        };
        let resultant = self.ui_state.resultant.key();
        let wheels = wheel_columns(table, resultant);
        if wheels.is_empty() {
            ui.colored_label(
                pal.DANGER,
                format!(
                    "A tabela {} não traz coeficientes de {resultant}.",
                    table.sheet
                ),
            );
            return;
        }
        let positive = uniform_column(table, resultant, "p");
        let negative = uniform_column(table, resultant, "p'");

        let total_columns = wheels.len() + 2;
        let total_width = TABLE_LABEL_WIDTH + total_columns as f32 * TABLE_CELL_WIDTH;
        let table_info = format!(
            "Tabela {} · {} = {}",
            table.sheet,
            table.param.clone().unwrap_or_else(|| "ly/lx".to_string()),
            table.ratio_raw.clone().unwrap_or_else(|| "—".to_string())
        );

        let lx_over_a = self.project.lx_m / self.project.wheel_spacing_m.max(0.1);
        let t_over_a = load_spread_width_m(self.project.pavement_m, self.project.slab_m)
            .map(|spread| spread / self.project.wheel_spacing_m.max(0.1))
            .unwrap_or(0.0);
        let t_values: Vec<f64> = wheels.iter().filter_map(|column| column.ta).collect();
        let row_marks = bracket_indices(&table.rows, lx_over_a);
        let column_marks = bracket_indices(&t_values, t_over_a);

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Coeficientes — {resultant}")).strong())
                .on_hover_text(table_info);
            ui.label(
                RichText::new(format!(
                    "· interpolação: lx/a = {lx_over_a:.3} · t/a = {t_over_a:.3}"
                ))
                .small()
                .color(pal.TEXT_WEAK),
            );
        });
        ui.add_space(2.0);

        let mono = TextStyle::Monospace.resolve(ui.style());
        let small = TextStyle::Small.resolve(ui.style());

        // Todas as linhas no fluxo normal: a aba inteira já é rolável, então uma
        // segunda área de rolagem aqui só esconderia linhas da tabela.
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            ui.set_width(total_width);

            // Cabeçalho, nível 1: grupos.
            ui.horizontal(|ui| {
                matrix_header(ui, TABLE_LABEL_WIDTH, pal, "");
                matrix_header(ui, wheels.len() as f32 * TABLE_CELL_WIDTH, pal, resultant);
                matrix_header(ui, 2.0 * TABLE_CELL_WIDTH, pal, "para todo t/a");
            });
            // Cabeçalho, nível 2: t/a e p/p'.
            ui.horizontal(|ui| {
                matrix_cell(
                    ui,
                    TABLE_LABEL_WIDTH,
                    pal,
                    "lx/a",
                    &small,
                    pal.TEXT_WEAK,
                    None,
                );
                for column in &wheels {
                    let text = column
                        .ta
                        .map(|value| format!("{value:.3}"))
                        .unwrap_or_else(|| "—".to_string());
                    matrix_cell(ui, TABLE_CELL_WIDTH, pal, &text, &small, pal.TEXT, None);
                }
                matrix_cell(ui, TABLE_CELL_WIDTH, pal, "p", &small, pal.TEXT, None);
                matrix_cell(ui, TABLE_CELL_WIDTH, pal, "p'", &small, pal.TEXT, None);
            });

            // Linhas de coeficientes.
            for (row_index, row_value) in table.rows.iter().enumerate() {
                let row_highlighted = row_marks.contains(&row_index);
                ui.horizontal(|ui| {
                    matrix_cell(
                        ui,
                        TABLE_LABEL_WIDTH,
                        pal,
                        &format!("{row_value:.2}"),
                        &mono,
                        pal.TEXT,
                        row_highlighted.then_some(pal.HIGHLIGHT),
                    );
                    for (column_index, column) in wheels.iter().enumerate() {
                        let highlighted = row_highlighted && column_marks.contains(&column_index);
                        let background = if highlighted {
                            Some(pal.HIGHLIGHT_STRONG)
                        } else if row_highlighted || column_marks.contains(&column_index) {
                            Some(pal.HIGHLIGHT)
                        } else {
                            None
                        };
                        let text = column
                            .values
                            .get(row_index)
                            .copied()
                            .flatten()
                            .map(|value| format!("{value:.3}"))
                            .unwrap_or_else(|| "–".to_string());
                        matrix_cell(
                            ui,
                            TABLE_CELL_WIDTH,
                            pal,
                            &text,
                            &mono,
                            pal.TEXT,
                            background,
                        );
                    }
                    for column in [positive, negative].into_iter().flatten() {
                        let text = column
                            .values
                            .get(row_index)
                            .copied()
                            .flatten()
                            .map(|value| format!("{value:.3}"))
                            .unwrap_or_else(|| "–".to_string());
                        matrix_cell(
                            ui,
                            TABLE_CELL_WIDTH,
                            pal,
                            &text,
                            &mono,
                            pal.TEXT,
                            row_highlighted.then_some(pal.HIGHLIGHT),
                        );
                    }
                });
            }
        });

        ui.add_space(4.0);
    }

    /// Painel de resultados com os valores do motor.
    fn results_panel(&self, ui: &mut egui::Ui, pal: theme::Pal) {
        let mono = TextStyle::Monospace.resolve(ui.style());
        egui::Frame::NONE
            .fill(pal.SURFACE)
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.set_width(RESULTS_WIDTH - 20.0);
                ui.label(RichText::new("Resultados").strong());
                ui.add_space(4.0);

                let table_text = self.project.table_label().unwrap_or("—").to_string();
                row_value(ui, pal, &mono, "Tabela utilizada", &table_text);

                let Some(table) = self.table() else {
                    return;
                };
                let resultant = self.ui_state.resultant.key();
                let lx_over_a = self.project.lx_m / self.project.wheel_spacing_m.max(0.1);
                let t_over_a = load_spread_width_m(self.project.pavement_m, self.project.slab_m)
                    .map(|spread| spread / self.project.wheel_spacing_m.max(0.1))
                    .unwrap_or(0.0);

                let ml = interpolated_wheel_coefficient(table, resultant, lx_over_a, t_over_a);
                let mp = interpolated_uniform_coefficient(table, resultant, "p", lx_over_a);
                let mpp = interpolated_uniform_coefficient(table, resultant, "p'", lx_over_a);

                ui.add_space(6.0);
                ui.label(RichText::new("Cargas móveis").color(pal.TEXT_WEAK));
                row_value(ui, pal, &mono, "M_L", &format_optional(ml));
                row_value(ui, pal, &mono, "M_p", &format_optional(mp));
                row_value(ui, pal, &mono, "M_p'", &format_optional(mpp));
                let live = self.selected_moment().map(|moment| moment.live);
                row_value(ui, pal, &mono, "M_q", &kilonewton_metre(live));

                ui.add_space(6.0);
                ui.label(RichText::new("Cargas permanentes").color(pal.TEXT_WEAK));
                let k = panel_coefficient(table, resultant);
                row_value(ui, pal, &mono, "k", &format_optional(k));
                let permanent = self.selected_moment().map(|moment| moment.permanent);
                row_value(ui, pal, &mono, "M_g", &kilonewton_metre(permanent));

                ui.add_space(6.0);
                let design = self.selected_moment().map(|moment| moment.design);
                row_value(ui, pal, &mono, "M_d", &kilonewton_metre(design));

                if self.selected_moment().is_none() {
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Clique em Calcular para obter os momentos.")
                            .small()
                            .color(pal.TEXT_WEAK),
                    );
                }
            });
    }

    /// Rodapé: unidade em uso e estado do cálculo.
    fn status_bar(&self, ui: &mut egui::Ui, pal: theme::Pal) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("kN · m/m").monospace().color(pal.TEXT_WEAK));
            ui.separator();
            if self.ui_state.computed_from.is_none() {
                ui.label(RichText::new("Sem cálculo").color(pal.TEXT_WEAK));
            } else if self.stale() {
                ui.label(RichText::new("Alterado após o último cálculo").color(pal.DANGER));
            } else {
                ui.label(RichText::new("Calculado").color(pal.ACCENT));
            }
            if !self.message.is_empty() {
                ui.separator();
                ui.label(RichText::new(self.message.as_str()).color(pal.TEXT_WEAK));
            }
        });
    }

    /// Aba "Results": momentos característicos e de projeto.
    fn results_tab(&mut self, ui: &mut egui::Ui) {
        let pal = theme::pal(ui);
        if self.moments.is_empty() {
            ui.label("Ainda sem resultados — preencha a aba Inputs e clique em Calcular.");
            return;
        }
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
                    ui.label(RichText::new(&moment.resultant).monospace());
                    ui.label(format!("{:.3}", moment.permanent));
                    ui.label(format!("{:.3}", moment.live));
                    ui.strong(format!("{:.3}", moment.design));
                    ui.end_row();
                }
            });
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "Coeficientes de ponderação: γg = 1,35 e γq = 1,5. \
                 Nas resultantes de borda (Mxe/Mye) a tração é na face superior.",
            )
            .small()
            .color(pal.TEXT_WEAK),
        );
    }

    /// Aba "Report": exportação da memória de cálculo.
    fn report_tab(&mut self, ui: &mut egui::Ui) {
        let pal = theme::pal(ui);
        ui.horizontal(|ui| {
            if ui.button("Exportar HTML").clicked() {
                self.export_report();
            }
            ui.label(
                RichText::new(self.settings.get().general.last_report_path.as_str())
                    .monospace()
                    .color(pal.TEXT_WEAK),
            );
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
                egui::Label::new(
                    RichText::new(self.report_html.as_str())
                        .monospace()
                        .color(pal.TEXT),
                )
                .wrap(),
            );
        });
    }

    /// Diálogos de caminhos e "Sobre".
    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.ui_state.show_paths {
            let mut open = true;
            egui::Window::new("Caminhos")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    let mut project_path = self.settings.get().general.last_project_path.clone();
                    let mut report_path = self.settings.get().general.last_report_path.clone();
                    ui.horizontal(|ui| {
                        ui.label("Projeto .frus");
                        ui.add(egui::TextEdit::singleline(&mut project_path).desired_width(240.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Memória HTML");
                        ui.add(egui::TextEdit::singleline(&mut report_path).desired_width(240.0));
                    });
                    self.settings.update(|settings| {
                        settings.general.last_project_path = project_path;
                        settings.general.last_report_path = report_path;
                    });
                });
            self.ui_state.show_paths = open;
        }

        if self.ui_state.show_about {
            let mut open = true;
            egui::Window::new("Sobre o FreeRusch")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    let pal = theme::pal(ui);
                    ui.label(RichText::new("FreeRusch").strong());
                    ui.label(format!("Versão {}", env!("CARGO_PKG_VERSION")));
                    ui.label("Cálculo de lajes de pontes pelo método de Rüsch.");
                    ui.label(
                        RichText::new(
                            "Método e coeficientes: tabelas DIN/IBSW e notas de aula do IME \
                             (Prof. Eduardo C. S. Thomaz). Licença MIT ou Apache-2.0.",
                        )
                        .small()
                        .color(pal.TEXT_WEAK),
                    );
                });
            self.ui_state.show_about = open;
        }
    }
}

impl eframe::App for FreeruschApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // O `App::ui` do eframe não pinta o fundo: sem isso a janela mostra a cor
        // de limpeza do backend, que é escura, e o tema claro fica ilegível.
        let background = ui.visuals().panel_fill;
        ui.painter()
            .rect_filled(ui.max_rect(), CornerRadius::ZERO, background);

        self.menu_bar(ui);
        ui.separator();

        // Rodapé fixo: unidade em uso e estado do cálculo, sempre visível.
        let pal = theme::pal(ui);
        egui::Panel::bottom("freerusch-status").show(ui, |ui| {
            ui.add_space(3.0);
            self.status_bar(ui, pal);
            ui.add_space(3.0);
        });

        if tab_bar(ui, &mut self.tab) {
            self.message.clear();
        }
        ui.separator();

        egui::Frame::NONE
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| match self.tab {
                TAB_INPUTS => self.inputs_tab(ui),
                TAB_RESULTS => self.results_tab(ui),
                TAB_REPORT => self.report_tab(ui),
                _ => {}
            });

        self.dialogs(ui.ctx());

        // Grava as configurações alteradas (debounce da própria craft-settings).
        let now = std::time::Instant::now();
        let tick = self.settings.tick(now);
        if let Some(deadline) = tick.next_deadline {
            ui.ctx()
                .request_repaint_after(deadline.saturating_duration_since(now));
        }
        if ui.ctx().input(|i| i.viewport().close_requested()) {
            let _ = self.settings.flush();
        }
    }
}

/// Desenha a barra de abas usando os primitivos de `craft_ui::tabs`.
fn tab_bar(ui: &mut egui::Ui, selected: &mut usize) -> bool {
    const TAB_HEIGHT: f32 = 26.0;
    const MIN_TAB_WIDTH: f32 = 70.0;
    const OVERFLOW_WIDTH: f32 = 28.0;

    let pal = theme::pal(ui);
    let available = ui.available_width();
    let font = egui::TextStyle::Button.resolve(ui.style());
    let natural: Vec<f32> = TABS
        .iter()
        .map(|label| {
            ui.painter()
                .layout_no_wrap((*label).to_owned(), font.clone(), pal.TEXT)
                .size()
                .x
                + 32.0
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
                    let fill = if is_selected {
                        pal.HIGHLIGHT_STRONG
                    } else if response.hovered() {
                        pal.HIGHLIGHT
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::same(4), fill);
                    let text_color = if is_selected {
                        pal.ACCENT
                    } else {
                        pal.TEXT_WEAK
                    };
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

/// Cabeçalho de grupo da tabela, centralizado sobre as colunas que agrupa.
fn matrix_header(ui: &mut egui::Ui, width: f32, pal: theme::Pal, text: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, TABLE_ROW_HEIGHT), Sense::hover());
    let painter = ui.painter();
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, pal.GRID),
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        TextStyle::Small.resolve(ui.style()),
        pal.TEXT,
    );
}

/// Célula da tabela de coeficientes, desenhada com alinhamento tabular.
fn matrix_cell(
    ui: &mut egui::Ui,
    width: f32,
    pal: theme::Pal,
    text: &str,
    font: &egui::FontId,
    color: egui::Color32,
    background: Option<egui::Color32>,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, TABLE_ROW_HEIGHT), Sense::hover());
    let painter = ui.painter();
    if let Some(background) = background {
        painter.rect_filled(rect, CornerRadius::ZERO, background);
    }
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, pal.GRID),
    );
    painter.text(
        pos2(rect.right() - 6.0, rect.center().y),
        Align2::RIGHT_CENTER,
        text,
        font.clone(),
        color,
    );
}

/// Campo somente leitura, com fundo próprio para se distinguir dos editáveis.
fn read_only(ui: &mut egui::Ui, pal: theme::Pal, value: &str, width: f32) {
    egui::Frame::NONE
        .fill(pal.READ_ONLY)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.add(
                egui::Label::new(RichText::new(value).monospace().color(pal.TEXT))
                    .selectable(false),
            );
        });
}

/// Linha chave/valor do painel de resultados.
fn row_value(ui: &mut egui::Ui, pal: theme::Pal, font: &egui::FontId, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).color(pal.TEXT_WEAK));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).font(font.clone()).color(pal.TEXT));
        });
    });
}

/// Índices das linhas/colunas que delimitam o valor alvo, para o realce.
///
/// Retorna um único índice quando o alvo coincide com um nó, dois quando está
/// entre nós, e o vizinho mais próximo quando está fora da faixa.
fn bracket_indices(values: &[f64], target: f64) -> Vec<usize> {
    if values.is_empty() {
        return Vec::new();
    }
    if let Some(index) = values
        .iter()
        .position(|value| (value - target).abs() < 1e-9)
    {
        return vec![index];
    }
    if target < values[0] {
        return vec![0];
    }
    if target > values[values.len() - 1] {
        return vec![values.len() - 1];
    }
    for index in 0..values.len() - 1 {
        if values[index] <= target && target <= values[index + 1] {
            return vec![index, index + 1];
        }
    }
    vec![values.len() - 1]
}

fn format_optional(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|| "—".to_string())
}

fn kilonewton_metre(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "—".to_string())
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1100.0, 720.0])
            .with_title("FreeRusch"),
        ..Default::default()
    };
    eframe::run_native(
        "FreeRusch",
        options,
        Box::new(|cc| Ok(Box::new(FreeruschApp::new(cc)))),
    )
}
