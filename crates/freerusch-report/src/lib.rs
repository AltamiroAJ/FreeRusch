//! Geração da memória de cálculo do FreeRusch em HTML.
//!
//! O relatório é montado a partir dos resultados de [`freerusch_core`] e escrito
//! com segurança por [`craft_atomic::atomic_write`] (substituição atômica, livre
//! de corrupção). A exportação para papel usa [`craft_print`]: o chamador fornece
//! os bytes PDF já gerados (por exemplo, pelo navegador/WebView da aplicação) e o
//! crate os submete ao spooler do sistema.

use std::path::Path;

use freerusch_core::{MomentDiagram, MomentResult, RuschError, TrainType};
use thiserror::Error;

/// Erros da geração da memória de cálculo.
#[derive(Debug, Error)]
pub enum ReportError {
    /// Falha de E/S durante a escrita do arquivo.
    #[error("falha de E/S ao escrever o relatório: {0}")]
    Io(#[from] std::io::Error),
    /// O cálculo de origem retornou erro.
    #[error("erro de cálculo: {0}")]
    Calc(#[from] RuschError),
}

/// Parâmetros de uma laje descritos na memória de cálculo.
#[derive(Debug, Clone, PartialEq)]
pub struct SlabInput {
    /// Identificação da obra/painel (texto livre).
    pub title: String,
    /// Trem-tipo adotado (NBR 7188:2013).
    pub train: TrainType,
    /// Vão considerado `l1` (m).
    pub l1_m: f64,
    /// Vão vizinho `l2` (m); `0.0` para painel isolado.
    pub l2_m: f64,
    /// Largura transversal do tabuleiro (m).
    pub deck_width_m: f64,
    /// Classe do concreto (ex.: `"C25/30"`).
    pub concrete_class: String,
}

impl SlabInput {
    /// Constrói um jogo de parâmetros válido de exemplo.
    pub fn new(
        title: &str,
        train: TrainType,
        l1_m: f64,
        l2_m: f64,
        deck_width_m: f64,
        concrete_class: &str,
    ) -> Self {
        SlabInput {
            title: title.to_string(),
            train,
            l1_m,
            l2_m,
            deck_width_m,
            concrete_class: concrete_class.to_string(),
        }
    }
}

/// Foge de entidades HTML nos campos textuais do relatório.
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Gera um SVG inline do diagrama de momentos, pronto para embutir na memória
/// de cálculo (documento autocontido, sem arquivos externos).
///
/// A escala vertical usa [`MomentDiagram::max_abs_moment`] e recai em `1.0`
/// quando o diagrama é nulo, evitando divisão por zero. Pontos não finitos são
/// descartados. Retorna string vazia quando não há pontos representáveis.
pub fn diagram_svg(diagram: &MomentDiagram) -> String {
    let pts: Vec<(f64, f64)> = diagram
        .points
        .iter()
        .copied()
        .filter(|&(x, m)| x.is_finite() && m.is_finite())
        .collect();
    if pts.is_empty() {
        return String::new();
    }

    const W: f64 = 680.0;
    const H: f64 = 220.0;
    const LEFT: f64 = 48.0;
    const RIGHT: f64 = 16.0;
    const TOP: f64 = 16.0;
    const BOTTOM: f64 = 28.0;

    let plot_w = W - LEFT - RIGHT;
    let plot_h = H - TOP - BOTTOM;
    let mid_y = TOP + plot_h / 2.0;

    let x_last = pts.last().map(|p| p.0).unwrap_or(0.0);
    let x_max = if x_last > 0.0 { x_last } else { 1.0 };
    let span = diagram.max_abs_moment();
    let span = if span > 0.0 { span } else { 1.0 };

    let mut polyline = String::with_capacity(pts.len() * 16);
    for (i, &(x, m)) in pts.iter().enumerate() {
        let px = LEFT + (x / x_max) * plot_w;
        let py = mid_y - (m / span) * (plot_h / 2.0);
        if i > 0 {
            polyline.push(' ');
        }
        polyline.push_str(&format!("{px:.2},{py:.2}"));
    }

    let m_max = pts
        .iter()
        .map(|&(_, m)| m)
        .fold(f64::NEG_INFINITY, f64::max);
    let m_min = pts.iter().map(|&(_, m)| m).fold(f64::INFINITY, f64::min);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="100%" role="img" aria-label="Diagrama de momentos fletores">
<rect x="0" y="0" width="{W}" height="{H}" fill="#fbfbfb" stroke="#ccc"/>
<line x1="{LEFT}" y1="{mid_y:.2}" x2="{x_right:.2}" y2="{mid_y:.2}" stroke="#888" stroke-dasharray="4 3"/>
<polyline points="{polyline}" fill="none" stroke="#b00" stroke-width="2"/>
<text x="{LEFT}" y="12" font-family="sans-serif" font-size="11" fill="#444">M⁺ = {m_max:.2} kN·m/m</text>
<text x="{LEFT}" y="{h_text:.2}" font-family="sans-serif" font-size="11" fill="#444">M⁻ = {m_min:.2} kN·m/m</text>
<text x="{x_right:.2}" y="{h_text:.2}" font-family="sans-serif" font-size="11" fill="#666" text-anchor="end">x (m)</text>
</svg>"##,
        x_right = W - RIGHT,
        h_text = H - 8.0,
    )
}

/// Gera a memória de cálculo completa em HTML (documento único, autocontido).
///
/// Inclui cabeçalho, premissas de entrada, coeficientes Rüsch interpolados e
/// momentos de projeto por metro de largura.
pub fn render_html(input: &SlabInput, result: &MomentResult) -> String {
    let beta = result
        .beta
        .map(|b| format!("{b:.4}"))
        .unwrap_or_else(|| "—".to_string());
    // Aviso obrigatório enquanto as tabelas de Rüsch não forem homologadas.
    let warning = if freerusch_core::RUSCH_TABLES_PRELIMINARY {
        "<p class=\"warn\">Resultados preliminares: as tabelas completas de Rüsch \
         ainda não foram transcritas de fonte verificada. Não use para projeto \
         executivo.</p>"
    } else {
        ""
    };
    // O diagrama é derivado do próprio resultado: a memória de cálculo não
    // depende de o chamador (GUI) fornecer os pontos.
    let diagrama = {
        let diagram = MomentDiagram::for_result(result, input.l1_m, 41);
        let svg = diagram_svg(&diagram);
        if svg.is_empty() {
            String::new()
        } else {
            format!("<h2>Diagrama de momentos</h2>\n<figure>{svg}</figure>")
        }
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="pt-br">
<head>
<meta charset="utf-8">
<title>Memória de cálculo — {title}</title>
<style>
body {{ font-family: sans-serif; margin: 2em; color: #222; }}
h1 {{ font-size: 1.4em; }}
table {{ border-collapse: collapse; margin-top: 1em; }}
th, td {{ border: 1px solid #999; padding: 4px 10px; text-align: right; }}
th {{ background: #eee; }}
.neg {{ color: #a00; }}
.warn {{ color: #8a1f1f; border: 1px solid #e0b4b4; background: #fdf2f2; padding: 8px; }}
figure {{ margin: 1em 0; }}
footer {{ margin-top: 2em; font-size: 0.85em; color: #666; }}
</style>
</head>
<body>
<h1>FreeRusch — Memória de cálculo (método de Rüsch)</h1>
<p>{title}</p>
{warning}
<h2>Premissas</h2>
<table>
<tr><th>Parâmetro</th><th>Valor</th></tr>
<tr><td>Trem-tipo (NBR 7188:2013)</td><td>{train}</td></tr>
<tr><td>Vão l₁</td><td>{l1:.3} m</td></tr>
<tr><td>Vão vizinho l₂</td><td>{l2:.3} m</td></tr>
<tr><td>Largura do tabuleiro b</td><td>{bw:.3} m</td></tr>
<tr><td>Concreto</td><td>{cc}</td></tr>
</table>
<h2>Coeficientes Rüsch</h2>
<table>
<tr><th>Coeficiente</th><th>Valor</th></tr>
<tr><td>α (momento positivo)</td><td>{alpha:.4}</td></tr>
<tr><td>β (momento negativo)</td><td>{beta}</td></tr>
</table>
<h2>Esforços de projeto (por metro de largura)</h2>
<table>
<tr><th>Seção</th><th>M (kN·m/m)</th></tr>
<tr><td>Positivo no vão</td><td>{mpos:.3}</td></tr>
<tr class="neg"><td>Negativo nos apoios</td><td>{mneg:.3}</td></tr>
</table>
{diagrama}
<footer>Gerado pelo FreeRusch — reimplementação livre do método de Rüsch.
Valores de tabela sujeitos à homologação final das tabelas completas.</footer>
</body>
</html>
"#,
        title = html_escape(&input.title),
        train = input.train.label(),
        l1 = input.l1_m,
        l2 = input.l2_m,
        bw = input.deck_width_m,
        cc = html_escape(&input.concrete_class),
        alpha = result.alpha,
        beta = beta,
        mpos = result.max_positive,
        mneg = result.max_negative,
        diagrama = diagrama,
        warning = warning,
    )
}

/// Escreve a memória de cálculo em HTML no caminho indicado, usando escrita
/// atômica (`craft-atomic`): um falha nunca deixa o arquivo anterior corrompido.
pub fn write_report(path: &Path, html: &str) -> Result<(), ReportError> {
    craft_atomic::atomic_write(path, html.as_bytes())?;
    Ok(())
}

/// Submete um PDF (já renderizado pelo chamador) ao spooler do sistema através
/// do [`craft_print`], com o título da memória de cálculo.
///
/// Retorna o recibo do spooler (mensagem do backend).
pub fn print_pdf(pdf_bytes: &[u8], title: &str) -> Result<String, ReportError> {
    let job = craft_print::Job {
        title: title.to_string(),
        ..Default::default()
    };
    craft_print::submit(pdf_bytes, &job)
        .map_err(|e| ReportError::Io(std::io::Error::other(e.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use freerusch_core::{EdgeCondition, RuschTable, bending_moment};

    fn sample() -> (SlabInput, MomentResult) {
        let table = RuschTable::for_edges(EdgeCondition::SimplySupported);
        let input = SlabInput::new(
            "Viaduto Centro — Painel P1",
            TrainType::TB450,
            8.0,
            0.0,
            6.0,
            "C25/30",
        );
        let result = bending_moment(&table, &TrainType::TB450, 8.0, 0.0, 25.0).expect("cálculo");
        (input, result)
    }

    #[test]
    fn html_contains_inputs_and_results() {
        let (input, result) = sample();
        let html = render_html(&input, &result);
        assert!(html.contains("Memória de cálculo"));
        assert!(html.contains("TB-450"));
        assert!(html.contains("Viaduto Centro"));
        assert!(html.contains("20.000")); // momento positivo esperado (kN·m/m)
    }

    #[test]
    fn html_warns_when_tables_are_preliminary() {
        let (input, result) = sample();
        let html = render_html(&input, &result);
        if freerusch_core::RUSCH_TABLES_PRELIMINARY {
            assert!(html.contains("Resultados preliminares"));
            assert!(html.contains("Não use para projeto executivo"));
        } else {
            assert!(!html.contains("Resultados preliminares"));
        }
    }

    #[test]
    fn html_escapes_markup_in_title() {
        let (mut input, result) = sample();
        input.title = "<script>alert(1)</script>".into();
        let html = render_html(&input, &result);
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn writes_atomically_to_disk() {
        let dir =
            std::env::temp_dir().join(format!("freerusch-report-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("memoria.html");
        let (input, result) = sample();
        write_report(&path, &render_html(&input, &result)).expect("escrita atômica");
        let content = std::fs::read_to_string(&path).expect("leitura");
        assert!(content.contains("</html>"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn report_embeds_diagram_svg() {
        let (input, result) = sample();
        let html = render_html(&input, &result);
        assert!(html.contains("<svg"));
        assert!(html.contains("polyline"));
        assert!(!html.contains("NaN"));
    }

    #[test]
    fn diagram_svg_survives_zero_moment_range() {
        // Diagrama inteiramente nulo: a escala vertical não pode dividir por zero.
        let flat = MomentResult {
            max_positive: 0.0,
            max_negative: 0.0,
            alpha: 0.0,
            beta: None,
        };
        let d = MomentDiagram::for_result(&flat, 8.0, 9);
        let svg = diagram_svg(&d);
        assert!(svg.contains("polyline"));
        assert!(!svg.contains("NaN"));
        assert!(!svg.to_lowercase().contains("inf"));
    }

    #[test]
    fn empty_diagram_renders_nothing() {
        let empty = MomentDiagram { points: Vec::new() };
        assert!(diagram_svg(&empty).is_empty());
    }
}
