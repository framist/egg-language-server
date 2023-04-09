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



pub trait CommonLanguageTrans {
    fn ast_to_sexpr(tree: &tree_sitter::Tree, tree_cursor: &tree_sitter::TreeCursor, code: &str) -> String;
    fn rpn_helper(token: &common::CommonLanguage, stack: &mut Vec<String>) -> Result<String, String>;
}

pub struct SimpleLanguage {

}

