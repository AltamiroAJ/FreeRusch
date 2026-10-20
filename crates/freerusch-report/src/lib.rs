//! Geração da memória de cálculo do FreeRusch em HTML.
//!
//! O relatório é montado a partir dos resultados de [`freerusch_core`] e escrito
//! com segurança por [`craft_atomic::atomic_write`] (substituição atômica, livre
//! de corrupção). A exportação para papel usa [`craft_print`]: o chamador fornece
//! os bytes PDF já gerados (por exemplo, pelo navegador/WebView da aplicação) e o
//! crate os submete ao spooler do sistema.

use std::path::Path;

use freerusch_core::{MomentResult, RuschError, TrainType};
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

/// Gera a memória de cálculo completa em HTML (documento único, autocontido).
///
/// Inclui cabeçalho, premissas de entrada, coeficientes Rüsch interpolados e
/// momentos de projeto por metro de largura.
pub fn render_html(input: &SlabInput, result: &MomentResult) -> String {
    let beta = result
        .beta
        .map(|b| format!("{b:.4}"))
        .unwrap_or_else(|| "—".to_string());
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
footer {{ margin-top: 2em; font-size: 0.85em; color: #666; }}
</style>
</head>
<body>
<h1>FreeRusch — Memória de cálculo (método de Rüsch)</h1>
<p>{title}</p>
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
        assert!(html.contains("18.000")); // momento positivo esperado (kN·m/m)
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
}
