pub mod egg_support;
pub mod javascript;
pub mod lisp;
pub mod python;
pub mod rejavascript;
pub mod relisp;
pub mod repython;

pub use crate::javascript::js_parser;
pub use crate::lisp::lisp_parser;
pub use crate::python::py_parser;
pub use crate::rejavascript::js_reparser;
pub use crate::relisp::lisp_reparser;
pub use crate::repython::py_reparser;

use crate::egg_support::*;
use log::*;

pub fn debug_reparser(s: &String) -> Result<String, String> {
    match s.parse::<EggIR>() {
        Ok(rpn) => rpn_to_human(&rpn, rpn_helper_simple),
        Err(e) => return Err(format!("egg-IR parse error: {}", e)),
    }
}

/// 树指针的方式打印
pub fn print_tree_sitter(cursor: &tree_sitter::TreeCursor, code: &str, indent_level: usize) {
    let indent = "|   ".repeat(indent_level);
    let node = cursor.node();
    let start = node.start_position();
    let end = node.end_position();
    debug!(
        "{}{:?}:{}  [{}:{} - {}:{}] {} {}",
        indent,
        node.kind(),
        node.kind_id(),
        start.row,
        start.column,
        end.row,
        end.column,
        if node.child_count() == 0 {
            node.utf8_text(code.as_bytes()).unwrap()
        } else {
            ""
        },
        if node.is_extra() { "extra" } else { "" }
    );
    let mut cursor = cursor.clone();
    if cursor.goto_first_child() {
        print_tree_sitter(&cursor, code, indent_level + 1);
        while cursor.goto_next_sibling() {
            print_tree_sitter(&cursor, code, indent_level + 1);
        }
        cursor.goto_parent();
    }
}

// 树形递归打印
// pub fn print_tree_sitter_node(node: &Node, code: &str, indent_level: usize) {
//     let indent = "|   ".repeat(indent_level);
//     let start = node.start_position();
//     let end = node.end_position();
//     debug!(
//         "{}{:?}:{}  [{}:{} - {}:{}] {}",
//         indent,
//         node.kind(),
//         node.kind_id(),
//         start.row,
//         start.column,
//         end.row,
//         end.column,
//         if node.child_count() == 0 {
//             node.utf8_text(code.as_bytes()).unwrap()
//         } else {
//             ""
//         }
//     );
//     let mut cursor = node.walk();
//     for child in node.children(&mut cursor) {
//         print_tree_sitter_node(&child, code, indent_level + 1);
//     }
// }
