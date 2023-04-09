mod simple;
mod math;
mod lambda;
mod prop;
pub mod common;

pub type EggIR = egg::RecExpr<common::CommonLanguage>;

pub fn egg_violence(s: &str) -> Result<String, String> {    
    common::simplify_test(s)
}

pub fn simplify(s: &str) -> Result<Option<EggIR>, String> {
    common::simplify(s)
}


