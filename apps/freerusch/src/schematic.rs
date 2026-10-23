//! Desenho esquemático da seção da laje, em vetor (`egui::Painter`).
//!
//! Nenhuma imagem ou bitmap: o desenho é construído a partir do estado atual
//! (vão, espessuras, vinculações, direção do tráfego) e das cores do tema. As
//! cotas exibem os valores reais de `lx`, `h` e `a`.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, Pos2, Rect, Shape, Stroke, StrokeKind, TextStyle, Vec2,
    pos2, vec2,
};

use crate::theme;

/// Altura reservada para o desenho.
pub const HEIGHT: f32 = 228.0;

/// Condição de uma borda do painel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EdgeKind {
    /// Vinculação ainda não definida.
    #[default]
    Undefined,
    /// Borda apoiada.
    Supported,
    /// Borda engastada.
    Clamped,
    /// Borda livre.
    Free,
}

impl EdgeKind {
    /// Rótulo em português usado nos seletores.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            EdgeKind::Undefined => "Indefinida",
            EdgeKind::Supported => "Apoiada",
            EdgeKind::Clamped => "Engastada",
            EdgeKind::Free => "Livre",
        }
    }

    /// Todas as opções, na ordem dos seletores.
    pub const ALL: [EdgeKind; 4] = [
        EdgeKind::Undefined,
        EdgeKind::Supported,
        EdgeKind::Clamped,
        EdgeKind::Free,
    ];
}

/// Direção do tráfego sobre o tabuleiro.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TrafficDirection {
    /// Tráfego na direção y (transversal ao vão desenhado).
    #[default]
    Vertical,
    /// Tráfego na direção x (ao longo do vão desenhado).
    Horizontal,
}

impl TrafficDirection {
    /// Rótulo curto para os botões de rádio.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            TrafficDirection::Vertical => "↓ vertical (y)",
            TrafficDirection::Horizontal => "→ horizontal (x)",
        }
    }
}

/// Estado que o desenho reflete.
#[derive(Clone, Copy, Debug)]
pub struct Sketch<'a> {
    /// Vão transversal (m).
    pub lx_m: f64,
    /// Espessura da laje (m).
    pub slab_m: f64,
    /// Espessura da pavimentação (m).
    pub pavement_m: f64,
    /// Distância transversal entre rodas (m).
    pub wheel_spacing_m: f64,
    /// Condições das bordas: superior, inferior, esquerda, direita.
    pub edges: [EdgeKind; 4],
    /// Direção do tráfego.
    pub traffic: TrafficDirection,
    /// Momento em exibição, já formatado (ex.: `"Mxm = 123,45 kN·m/m"`).
    pub moment_label: Option<&'a str>,
}

/// Desenha o esquema ocupando a largura disponível.
pub fn show(ui: &mut egui::Ui, sketch: &Sketch<'_>) {
    let width = ui.available_width().max(260.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, HEIGHT), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let pal = theme::pal(ui);
    let small = TextStyle::Small.resolve(ui.style());
    let mono = TextStyle::Monospace.resolve(ui.style());

    let left = rect.left() + 40.0;
    let right = rect.right() - 104.0;
    let span = (right - left).max(60.0);
    let middle = rect.center().y + 14.0;

    let thickness = lerp(24.0, 72.0, norm(sketch.slab_m, 0.10, 1.00));
    let pavement = lerp(2.0, 12.0, norm(sketch.pavement_m, 0.0, 0.20));

    let slab = Rect::from_min_max(
        pos2(left, middle - thickness / 2.0),
        pos2(right, middle + thickness / 2.0),
    );
    let pavement_rect = Rect::from_min_max(
        pos2(left, slab.top() - pavement),
        pos2(left + span, slab.top()),
    );

    // Laje e pavimentação.
    painter.rect_filled(slab, CornerRadius::same(2), pal.SURFACE);
    painter.rect_stroke(
        slab,
        CornerRadius::same(2),
        Stroke::new(1.4, pal.TEXT),
        StrokeKind::Inside,
    );
    if pavement > 1.0 {
        painter.rect_filled(pavement_rect, CornerRadius::ZERO, pal.GRID);
        painter.rect_stroke(
            pavement_rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, pal.TEXT_WEAK),
            StrokeKind::Inside,
        );
    }

    // Vinculações nas quatro bordas.
    edge_glyph(&painter, &pal, sketch.edges[0], slab, Side::Top, span);
    edge_glyph(&painter, &pal, sketch.edges[1], slab, Side::Bottom, span);
    edge_glyph(&painter, &pal, sketch.edges[2], slab, Side::Left, span);
    edge_glyph(&painter, &pal, sketch.edges[3], slab, Side::Right, span);

    // Carga em trapézio sobre a faixa de tráfego.
    let pixels_per_metre = span / sketch.lx_m.max(0.1) as f32;
    let half_bottom =
        ((sketch.wheel_spacing_m / 2.0) as f32 * pixels_per_metre).clamp(6.0, span * 0.45);
    let half_top = (half_bottom * 0.45).max(3.0);
    let load_height = 44.0;
    let baseline = pavement_rect.top();
    let centre = slab.center().x;
    let trapezoid = vec![
        pos2(centre - half_top, baseline - load_height),
        pos2(centre + half_top, baseline - load_height),
        pos2(centre + half_bottom, baseline - 2.0),
        pos2(centre - half_bottom, baseline - 2.0),
    ];
    painter.add(Shape::convex_polygon(
        trapezoid.clone(),
        pal.HIGHLIGHT_STRONG,
        Stroke::new(1.2, pal.ACCENT),
    ));

    // Momento em exibição sobre a área de carga.
    if let Some(label) = sketch.moment_label {
        painter.text(
            pos2(centre, baseline - load_height - 12.0),
            Align2::CENTER_CENTER,
            label,
            mono.clone(),
            pal.TEXT,
        );
    }

    // Direção do tráfego.
    let arrow_origin = pos2(centre - half_bottom - 16.0, baseline - load_height / 2.0);
    match sketch.traffic {
        TrafficDirection::Vertical => {
            arrow(&painter, &pal, arrow_origin, vec2(0.0, 22.0));
            painter.text(
                arrow_origin + vec2(0.0, -10.0),
                Align2::CENTER_CENTER,
                "tráfego y",
                small.clone(),
                pal.TEXT_WEAK,
            );
        }
        TrafficDirection::Horizontal => {
            arrow(&painter, &pal, arrow_origin, vec2(24.0, 0.0));
            painter.text(
                arrow_origin + vec2(12.0, 16.0),
                Align2::CENTER_CENTER,
                "tráfego x",
                small.clone(),
                pal.TEXT_WEAK,
            );
        }
    }

    // Cota do vão lx.
    let dimension_y = slab.bottom() + 34.0;
    dimension_line(
        &painter,
        &pal,
        pos2(left, dimension_y),
        pos2(right, dimension_y),
        true,
    );
    painter.text(
        pos2(centre, dimension_y + 14.0),
        Align2::CENTER_CENTER,
        format!("lx = {:.2} m", sketch.lx_m),
        mono.clone(),
        pal.TEXT,
    );

    // Cota da espessura h, acima da borda superior para não sair do painel.
    let thickness_x = right + 20.0;
    dimension_line(
        &painter,
        &pal,
        pos2(thickness_x, slab.top()),
        pos2(thickness_x, slab.bottom()),
        false,
    );
    painter.text(
        pos2(thickness_x + 4.0, slab.top() - 12.0),
        Align2::LEFT_CENTER,
        format!("h = {:.2} m", sketch.slab_m),
        small.clone(),
        pal.TEXT,
    );

    // Cota da largura de distribuição a, anotada no pé do trapézio.
    let a_left = centre - half_bottom;
    let a_right = centre + half_bottom;
    for x in [a_left, a_right] {
        painter.line_segment(
            [pos2(x, baseline - 9.0), pos2(x, baseline - 1.0)],
            Stroke::new(1.0, pal.TEXT_WEAK),
        );
    }
    painter.text(
        pos2(a_right + 8.0, baseline - load_height * 0.5),
        Align2::LEFT_CENTER,
        format!("a = {:.2} m", sketch.wheel_spacing_m),
        small,
        pal.TEXT_WEAK,
    );
}

/// Lado da seção onde a vinculação é desenhada.
#[derive(Clone, Copy)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

/// Desenha o símbolo da vinculação no lado indicado.
fn edge_glyph(
    painter: &egui::Painter,
    pal: &theme::Pal,
    kind: EdgeKind,
    slab: Rect,
    side: Side,
    span: f32,
) {
    let stroke = Stroke::new(1.4, pal.TEXT);
    match kind {
        EdgeKind::Undefined => {
            let (from, to) = match side {
                Side::Top => (slab.left_top(), slab.right_top()),
                Side::Bottom => (slab.left_bottom(), slab.right_bottom()),
                Side::Left => (slab.left_top(), slab.left_bottom()),
                Side::Right => (slab.right_top(), slab.right_bottom()),
            };
            painter.extend(Shape::dashed_line(&[from, to], stroke, 4.0, 4.0));
        }
        EdgeKind::Supported => {
            // Triângulo de apoio apontando para fora do elemento.
            let size = 10.0;
            let points = match side {
                Side::Top => vec![
                    pos2(slab.left() + span * 0.5 - size, slab.top() - size),
                    pos2(slab.left() + span * 0.5 + size, slab.top() - size),
                    pos2(slab.left() + span * 0.5, slab.top()),
                ],
                Side::Bottom => vec![
                    pos2(slab.left() + span * 0.5 - size, slab.bottom() + size),
                    pos2(slab.left() + span * 0.5 + size, slab.bottom() + size),
                    pos2(slab.left() + span * 0.5, slab.bottom()),
                ],
                Side::Left => vec![
                    pos2(slab.left() - size, slab.center().y - size),
                    pos2(slab.left() - size, slab.center().y + size),
                    pos2(slab.left(), slab.center().y),
                ],
                Side::Right => vec![
                    pos2(slab.right() + size, slab.center().y - size),
                    pos2(slab.right() + size, slab.center().y + size),
                    pos2(slab.right(), slab.center().y),
                ],
            };
            painter.add(Shape::convex_polygon(points, Color32::TRANSPARENT, stroke));
        }
        EdgeKind::Clamped => {
            // Hachura curta perpendicular à borda.
            let count = 7;
            for index in 0..count {
                let fraction = (index as f32 + 1.0) / (count as f32 + 1.0);
                let segment = match side {
                    Side::Top => {
                        let x = slab.left() + span * fraction;
                        (pos2(x, slab.top() - 9.0), pos2(x - 6.0, slab.top()))
                    }
                    Side::Bottom => {
                        let x = slab.left() + span * fraction;
                        (pos2(x, slab.bottom() + 9.0), pos2(x - 6.0, slab.bottom()))
                    }
                    Side::Left => {
                        let y = slab.top() + slab.height() * fraction;
                        (pos2(slab.left() - 9.0, y), pos2(slab.left(), y - 6.0))
                    }
                    Side::Right => {
                        let y = slab.top() + slab.height() * fraction;
                        (pos2(slab.right() + 9.0, y), pos2(slab.right(), y - 6.0))
                    }
                };
                painter.line_segment([segment.0, segment.1], stroke);
            }
        }
        EdgeKind::Free => {
            // Traço solto, afastado da borda.
            let segment = match side {
                Side::Top => (
                    pos2(slab.left() + span * 0.5 - 14.0, slab.top() - 8.0),
                    pos2(slab.left() + span * 0.5 + 14.0, slab.top() - 12.0),
                ),
                Side::Bottom => (
                    pos2(slab.left() + span * 0.5 - 14.0, slab.bottom() + 8.0),
                    pos2(slab.left() + span * 0.5 + 14.0, slab.bottom() + 12.0),
                ),
                Side::Left => (
                    pos2(slab.left() - 8.0, slab.center().y - 12.0),
                    pos2(slab.left() - 12.0, slab.center().y + 12.0),
                ),
                Side::Right => (
                    pos2(slab.right() + 8.0, slab.center().y - 12.0),
                    pos2(slab.right() + 12.0, slab.center().y + 12.0),
                ),
            };
            painter.line_segment([segment.0, segment.1], stroke);
        }
    }
}

/// Linha de cota com traços nas extremidades.
fn dimension_line(
    painter: &egui::Painter,
    pal: &theme::Pal,
    from: Pos2,
    to: Pos2,
    horizontal: bool,
) {
    let stroke = Stroke::new(1.0, pal.TEXT_WEAK);
    painter.line_segment([from, to], stroke);
    let tick = 4.0;
    for (point, direction) in [(from, -1.0_f32), (to, 1.0_f32)] {
        let offset = if horizontal {
            vec2(0.0, tick * direction)
        } else {
            vec2(tick * direction, 0.0)
        };
        painter.line_segment([point - offset, point + offset], stroke);
    }
}

/// Seta simples usada para indicar a direção do tráfego.
fn arrow(painter: &egui::Painter, pal: &theme::Pal, origin: Pos2, delta: Vec2) {
    let stroke = Stroke::new(1.4, pal.ACCENT);
    let tip = origin + delta;
    painter.line_segment([origin, tip], stroke);
    let normal = vec2(-delta.y, delta.x).normalized() * 4.0;
    painter.line_segment([tip, tip - delta.normalized() * 7.0 + normal], stroke);
    painter.line_segment([tip, tip - delta.normalized() * 7.0 - normal], stroke);
}

fn lerp(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount.clamp(0.0, 1.0)
}

fn norm(value: f64, min: f64, max: f64) -> f32 {
    if (max - min).abs() < f64::EPSILON {
        return 0.0;
    }
    (((value - min) / (max - min)) as f32).clamp(0.0, 1.0)
}
