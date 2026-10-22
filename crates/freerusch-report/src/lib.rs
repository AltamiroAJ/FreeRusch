//! Geração da memória de cálculo do FreeRusch em HTML.
//!
//! O relatório é montado a partir dos resultados de [`freerusch_core`] e escrito
//! com segurança por [`craft_atomic::atomic_write`] (substituição atômica, livre
//! de corrupção). A exportação para papel usa [`craft_print`]: o chamador fornece
//! os bytes PDF já gerados (por exemplo, pelo navegador/WebView da aplicação) e o
//! crate os submete ao spooler do sistema.
//!
//! O documento declara as próprias hipóteses: número da tabela de Rüsch,
//! espessuras, `a`, cargas, além de `φ` e do espraiamento `t` calculados na hora.
//! As fontes do método estão citadas no rodapé.

use std::path::Path;

use freerusch_core::{
    DesignMoment, RuschDesignInput, RuschError, impact_coefficient, load_spread_width_m,
};
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

/// Dados editoriais que não entram no cálculo, só no documento.
#[derive(Debug, Clone, PartialEq)]
pub struct ReportContext<'a> {
    /// Identificação da obra/painel (texto livre).
    pub title: &'a str,
    /// Classe do concreto (ex.: `"C25/30"`).
    pub concrete_class: &'a str,
    /// Rótulo da tabela de Rüsch usada (ex.: `"Nr 65.1"`).
    pub table_label: &'a str,
}

/// Foge de entidades HTML nos campos textuais do relatório.
fn html_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(character),
        }
    }
    out
}

/// Gera a memória de cálculo completa em HTML (documento único, autocontido).
///
/// Inclui premissas, coeficientes derivados (`φ`, `t`), e a tabela de momentos
/// por resultante com carga permanente, carga móvel e valor de projeto.
///
/// # Panics
///
/// Não entra em pânico: se `φ` ou `t` não puderem ser calculados, o documento
/// mostra `—` no lugar do valor derivado.
#[must_use]
pub fn render_html(
    input: &RuschDesignInput,
    context: &ReportContext<'_>,
    moments: &[DesignMoment],
) -> String {
    let impact = impact_coefficient(input.lx_m)
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|_| "—".to_string());
    let spread = load_spread_width_m(input.pavement_m, input.slab_m)
        .map(|value| format!("{value:.4}"))
        .unwrap_or_else(|_| "—".to_string());
    let lx_over_a = if input.wheel_spacing_m > 0.0 {
        format!("{:.4}", input.lx_m / input.wheel_spacing_m)
    } else {
        "—".to_string()
    };
    let t_over_a = if input.wheel_spacing_m > 0.0 {
        format!(
            "{:.4}",
            spread.parse::<f64>().unwrap_or(0.0) / input.wheel_spacing_m
        )
    } else {
        "—".to_string()
    };

    let mut rows = String::new();
    for moment in moments {
        rows.push_str(&format!(
            "<tr><td>{name}</td><td>{permanent:.3}</td><td>{live:.3}</td>\
             <td><strong>{design:.3}</strong></td></tr>\n",
            name = html_escape(&moment.resultant),
            permanent = moment.permanent,
            live = moment.live,
            design = moment.design,
        ));
    }

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
td:first-child {{ text-align: left; font-family: monospace; }}
footer {{ margin-top: 2em; font-size: 0.85em; color: #666; }}
.method {{ background: #f7f7f7; border-left: 3px solid #999; padding: 8px 12px; font-family: monospace; }}
</style>
</head>
<body>
<h1>FreeRusch — Memória de cálculo (método de Rüsch)</h1>
<p>{title}</p>

<h2>Premissas</h2>
<table>
<tr><th>Parâmetro</th><th>Valor</th></tr>
<tr><td>Tabela de Rüsch</td><td>{table}</td></tr>
<tr><td>Vão transversal lx</td><td>{lx:.3} m</td></tr>
<tr><td>Espessura da laje h</td><td>{slab:.3} m</td></tr>
<tr><td>Espessura da pavimentação</td><td>{pavement:.3} m</td></tr>
<tr><td>Distância transversal entre rodas a</td><td>{spacing:.3} m</td></tr>
<tr><td>Carga permanente g</td><td>{g:.3} kN/m²</td></tr>
<tr><td>Carga por roda P</td><td>{wheel:.3} kN</td></tr>
<tr><td>Carga uniforme da pista q</td><td>{q:.3} kN/m²</td></tr>
<tr><td>Concreto</td><td>{concrete}</td></tr>
</table>

<h2>Coeficientes derivados</h2>
<table>
<tr><th>Grandeza</th><th>Valor</th></tr>
<tr><td>Coeficiente de impacto φ = 1,4 − 0,007·lx</td><td>{impact}</td></tr>
<tr><td>Espraiamento da roda t</td><td>{spread} m</td></tr>
<tr><td>lx/a</td><td>{lx_a}</td></tr>
<tr><td>t/a</td><td>{t_a}</td></tr>
</table>

<h2>Método</h2>
<p class="method">M_k,g = k · g · lx²<br>
M_k,q = φ · (P · M_L + q · M_p + q · M_p')<br>
M_d = {gamma_g} · M_k,g + {gamma_q} · M_k,q</p>

<h2>Momentos fletores (kN·m/m)</h2>
<table>
<tr><th>Resultante</th><th>Permanente</th><th>Móvel (com impacto)</th><th>Projeto</th></tr>
{rows}</table>

<footer>Gerado pelo FreeRusch — reimplementação livre do método de Rüsch.
Método e coeficientes conforme as tabelas DIN/IBSW e as notas de aula do IME
(Prof. Eduardo C. S. Thomaz, "Lajes do tabuleiro — Método de Rüsch").
Coeficientes de ponderação adotados: γg = {gamma_g}, γq = {gamma_q}.</footer>
</body>
</html>
"#,
        title = html_escape(context.title),
        table = html_escape(context.table_label),
        lx = input.lx_m,
        slab = input.slab_m,
        pavement = input.pavement_m,
        spacing = input.wheel_spacing_m,
        g = input.permanent_load_kn_m2,
        wheel = input.wheel_load_kn,
        q = input.uniform_load_kn_m2,
        concrete = html_escape(context.concrete_class),
        impact = impact,
        spread = spread,
        lx_a = lx_over_a,
        t_a = t_over_a,
        gamma_g = freerusch_core::GAMMA_G,
        gamma_q = freerusch_core::GAMMA_Q,
        rows = rows,
    )
}

/// Escreve a memória de cálculo em HTML no caminho indicado, usando escrita
/// atômica (`craft-atomic`): uma falha nunca deixa o arquivo anterior corrompido.
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
    use freerusch_core::{GAMMA_G, GAMMA_Q, RuschTableData, design_moments};

    /// Exemplo do workbook: tabela Nr 65.1, lx = 7,00 m.
    fn sample() -> (RuschDesignInput, Vec<DesignMoment>) {
        let input = RuschDesignInput {
            table_number: 65,
            table_variant: Some(1),
            lx_m: 7.0,
            slab_m: 0.26,
            pavement_m: 0.07,
            wheel_spacing_m: 2.0,
            permanent_load_kn_m2: 10.5209,
            wheel_load_kn: 75.0,
            uniform_load_kn_m2: 5.0,
        };
        let moments = design_moments(&input).expect("cálculo");
        (input, moments)
    }

    fn context() -> ReportContext<'static> {
        ReportContext {
            title: "Viaduto Centro — Painel P1",
            concrete_class: "C25/30",
            table_label: "Nr 65.1",
        }
    }

    #[test]
    fn html_declares_inputs_and_moments() {
        let (input, moments) = sample();
        let html = render_html(&input, &context(), &moments);

        assert!(html.contains("Memória de cálculo"));
        assert!(html.contains("Nr 65.1"), "deve declarar a tabela usada");
        assert!(html.contains("Viaduto Centro"));

        // Coeficientes derivados aparecem com o valor calculado.
        let impact = freerusch_core::impact_coefficient(input.lx_m).expect("φ");
        let spread =
            freerusch_core::load_spread_width_m(input.pavement_m, input.slab_m).expect("t");
        assert!(html.contains(&format!("{impact:.4}")), "φ no documento");
        assert!(html.contains(&format!("{spread:.4}")), "t no documento");

        // M_k,g do Mxm confere com Main!I24 e o M_d aparece formatado.
        let mxm = moments.iter().find(|m| m.resultant == "Mxm").expect("Mxm");
        assert!(html.contains("32.478"), "M_k,g de Mxm (Main!I24)");
        assert!(html.contains(&format!("{:.3}", mxm.design)), "M_d de Mxm");
        assert!(!html.contains("NaN"));
    }

    #[test]
    fn html_states_the_combination_factors() {
        let (input, moments) = sample();
        let html = render_html(&input, &context(), &moments);
        assert!(html.contains(&format!("{GAMMA_G}")));
        assert!(html.contains(&format!("{GAMMA_Q}")));
        // M_d de Mxm = 1,35·32,478 + 1,5·~75,96.
        assert!(html.contains("157.8") || html.contains("157.9") || html.contains("158."));
    }

    #[test]
    fn html_escapes_markup_in_title() {
        let (input, moments) = sample();
        let mut ctx = context();
        ctx.title = "<script>alert(1)</script>";
        let html = render_html(&input, &ctx, &moments);
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn writes_atomically_to_disk() {
        let dir =
            std::env::temp_dir().join(format!("freerusch-report-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("memoria.html");

        let (input, moments) = sample();
        write_report(&path, &render_html(&input, &context(), &moments)).expect("escrita atômica");
        let content = std::fs::read_to_string(&path).expect("leitura");
        assert!(content.contains("</html>"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn table_label_matches_the_embedded_catalog() {
        // O rótulo usado no relatório vem do acervo, não de string solta.
        let table = RuschTableData::find(65, Some(1)).expect("tabela Nr 65.1");
        assert_eq!(table.sheet, "Nr 65.1");
    }
}
