use log::*;
use crate::*;
use crate::egg_support::simplify;

pub fn lisp_parser(s: &str) -> Vec<EggDiagnostic> {
    // no parser step
	// no ast_to_sexpr step
    let sexpr = s;
    info!("sexpr: \n{}", &sexpr);
    debug!(
        "pretty sexp: \n{}",
        rpn_to_human(&s.parse().unwrap(), rpn_helper_simple).unwrap()
    );

    let mut diagnostics: Vec<EggDiagnostic> = Vec::new();

    let start_position = Position::new(0, 0);
    let lines = s.lines();
    let end_position = match (lines.clone().count(), lines.last()) {
        (count, Some(last_line)) => Position::new(count as u32 - 1, last_line.len() as u32),
        _ => Position::new(0, 0),
    };
    let span = Range::new(start_position, end_position);

    match simplify(&sexpr) {
        Ok(sexp) => match sexp {
            Some(s) => {
                diagnostics.push(EggDiagnostic {
                    span,
                    reason: "can be simplified".to_string(),
                    sexpr: Some(s.to_string()),
                    label: DiagnosticSeverity::INFORMATION,
                });
                return diagnostics;
            }
            None => {
                return vec![];
            }
        },
        Err(e) => {
            diagnostics.push(EggDiagnostic {
                span,
                reason: format!("egg error: {}", e),
                sexpr: None,
                label: DiagnosticSeverity::ERROR,
            });
            return diagnostics;
        }
    }
}
