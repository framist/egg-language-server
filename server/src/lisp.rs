use log::*;
use crate::egg_support::simplify;

pub fn lisp_parser(s: &str) -> Result<String, String> {
    // no parser step
	// no ast_to_sexpr step
    let sexpr = s;
    info!("sexpr: \n{}", &sexpr);
    match simplify(&sexpr) {
        Ok(sexp) => match sexp {
            Some(sexp) => Ok(sexp.to_string()),
            None => Ok("已经最优了".to_string()),
        },
        Err(e) => Err(format!("egg error: {}", e)),
    }
}
