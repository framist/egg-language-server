use crate::egg_support::*;

pub fn lisp_reparser(s: &String) -> Result<String, String> {
    match s.parse::<EggIR>() {
        Ok(rpn) => rpn_to_human(&rpn, rpn_helper_simple), // TODO
        Err(e) => return Err(format!("egg-IR parse error: {}", e)),
    }
}