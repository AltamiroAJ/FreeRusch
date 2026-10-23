//! Paleta e tipografia do FreeRusch, derivadas do tema ativo do egui.
//!
//! Regra do projeto: **a UI não escolhe cores**. Todo código de interface lê
//! [`pal`] e usa os tokens daqui; realce, grades e texto saem do tema ativo, não
//! de constantes fixas. O contraste é calculado (WCAG) a partir do fundo do
//! painel, para que tema claro e escuro passem o mesmo critério de leitura.

use eframe::egui::{self, Color32, FontFamily, FontId, TextStyle};

/// Tokens de cor derivados do tema ativo.
///
/// Os nomes em caixa alta são deliberados: a especificação da demanda usa
/// `theme::pal().TEXT` e `theme::pal().TEXT_WEAK`.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pal {
    /// Texto de corpo. Contraste ≥ 7:1 contra o fundo do painel.
    pub TEXT: Color32,
    /// Texto secundário. Contraste ≥ 4,5:1 contra o fundo do painel.
    pub TEXT_WEAK: Color32,
    /// Fundo de caixas e painéis internos.
    pub SURFACE: Color32,
    /// Linhas de grade, visíveis porém discretas.
    pub GRID: Color32,
    /// Faixa de realce de linha/coluna em uso.
    pub HIGHLIGHT: Color32,
    /// Realce forte da célula de interseção.
    pub HIGHLIGHT_STRONG: Color32,
    /// Fundo de campo somente leitura (visualmente distinto dos editáveis).
    pub READ_ONLY: Color32,
    /// Cor de destaque para rótulos ativos.
    pub ACCENT: Color32,
    /// Erro ou aviso.
    pub DANGER: Color32,
}

/// Tokens para o tema ativo deste `Ui`.
#[must_use]
pub fn pal(ui: &egui::Ui) -> Pal {
    from_visuals(ui.visuals())
}

/// Deriva a paleta de um conjunto de `Visuals` (testável sem interface).
#[must_use]
pub fn from_visuals(visuals: &egui::Visuals) -> Pal {
    let background = visuals.panel_fill;
    let text = highest_contrast(background);
    let text_weak = fade_until_contrast(text, background, 5.0);

    let accent_source = if visuals.selection.bg_fill == Color32::TRANSPARENT {
        visuals.hyperlink_color
    } else {
        visuals.selection.bg_fill
    };

    Pal {
        TEXT: text,
        TEXT_WEAK: text_weak,
        // Superfícies derivadas do próprio texto: dão a mesma definição de
        // painel nos dois temas, sem depender dos cinzas internos do egui.
        SURFACE: blend(background, text, 0.08),
        GRID: blend(text, background, 0.72),
        HIGHLIGHT: blend(background, accent_source, 0.28),
        HIGHLIGHT_STRONG: blend(background, accent_source, 0.52),
        READ_ONLY: blend(background, text, 0.14),
        ACCENT: visuals.selection.stroke.color,
        DANGER: visuals.error_fg_color,
    }
}

/// Aplica a tipografia base: 14 pt para corpo, 16 pt para títulos.
pub fn configure(ctx: &egui::Context) {
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(16.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(13.5, FontFamily::Monospace),
            ),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size.y = 22.0;
    });
}

/// Escolhe entre branco e preto o que tiver mais contraste com o fundo.
fn highest_contrast(background: Color32) -> Color32 {
    let light = Color32::from_rgb(246, 246, 246);
    let dark = Color32::from_rgb(18, 18, 18);
    if contrast(light, background) >= contrast(dark, background) {
        light
    } else {
        dark
    }
}

/// Escurece/clareia `foreground` em direção ao fundo até o contraste cair ao alvo.
///
/// Usado para o texto secundário: parte do texto de corpo (já no máximo de
/// contraste) e recua até `target`, garantindo piso legível em ambos os temas.
fn fade_until_contrast(foreground: Color32, background: Color32, target: f32) -> Color32 {
    let mut best = foreground;
    for step in 1..=20 {
        let ratio = step as f32 / 20.0;
        let candidate = blend(foreground, background, ratio);
        if contrast(candidate, background) < target {
            break;
        }
        best = candidate;
    }
    best
}

/// Interpola duas cores: `amount` é a fração de `to`.
fn blend(from: Color32, to: Color32, amount: f32) -> Color32 {
    let amount = amount.clamp(0.0, 1.0);
    let mix = |a: u8, b: u8| -> u8 {
        let value = f32::from(a) + (f32::from(b) - f32::from(a)) * amount;
        value.round().clamp(0.0, 255.0) as u8
    };
    Color32::from_rgb(
        mix(from.r(), to.r()),
        mix(from.g(), to.g()),
        mix(from.b(), to.b()),
    )
}

/// Razão de contraste WCAG entre duas cores.
#[must_use]
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// Luminância relativa WCAG de uma cor.
#[must_use]
pub fn luminance(color: Color32) -> f32 {
    let channel = |value: u8| -> f32 {
        let value = f32::from(value) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_theme_meets_contrast_targets() {
        let visuals = egui::Visuals::dark();
        let palette = from_visuals(&visuals);
        assert!(contrast(palette.TEXT, visuals.panel_fill) >= 7.0);
        assert!(contrast(palette.TEXT_WEAK, visuals.panel_fill) >= 4.5);
        assert!(contrast(palette.TEXT, palette.HIGHLIGHT) >= 4.5);
    }

    #[test]
    fn light_theme_meets_contrast_targets() {
        let visuals = egui::Visuals::light();
        let palette = from_visuals(&visuals);
        assert!(contrast(palette.TEXT, visuals.panel_fill) >= 7.0);
        assert!(contrast(palette.TEXT_WEAK, visuals.panel_fill) >= 4.5);
        assert!(contrast(palette.TEXT, palette.HIGHLIGHT) >= 4.5);
    }

    #[test]
    fn highlight_is_distinct_from_the_panel_in_both_themes() {
        for visuals in [egui::Visuals::dark(), egui::Visuals::light()] {
            let palette = from_visuals(&visuals);
            assert_ne!(palette.HIGHLIGHT, visuals.panel_fill);
            assert_ne!(palette.HIGHLIGHT_STRONG, palette.HIGHLIGHT);
            assert!(contrast(palette.HIGHLIGHT_STRONG, visuals.panel_fill) > 1.1);
        }
    }

    #[test]
    fn read_only_fields_stay_legible_and_distinct() {
        for visuals in [egui::Visuals::dark(), egui::Visuals::light()] {
            let palette = from_visuals(&visuals);
            assert!(
                contrast(palette.TEXT, palette.READ_ONLY) >= 4.5,
                "texto do campo somente leitura ilegível no tema"
            );
            assert_ne!(palette.READ_ONLY, visuals.panel_fill);
            assert_ne!(palette.READ_ONLY, palette.SURFACE);
            assert!(contrast(palette.TEXT, palette.SURFACE) >= 4.5);
        }
    }

    #[test]
    fn grid_is_visible_but_weaker_than_text() {
        for visuals in [egui::Visuals::dark(), egui::Visuals::light()] {
            let palette = from_visuals(&visuals);
            let grid = contrast(palette.GRID, visuals.panel_fill);
            let text = contrast(palette.TEXT, visuals.panel_fill);
            assert!(grid > 1.2, "grade invisível");
            assert!(grid < text, "grade mais forte que o texto");
        }
    }
}
