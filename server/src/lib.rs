pub mod egg_support;
pub mod python;

// TODO egg parser 的接口
// pub trait EggParser {
// 	fn parse(&self, egg: &str) -> Option<String>;
// }
pub trait CommonLanguageTrans {
    fn ast_to_sexpr(tree: &tree_sitter::Tree, tree_cursor: &tree_sitter::TreeCursor, code: &str) -> String;
}
