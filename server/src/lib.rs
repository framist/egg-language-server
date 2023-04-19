pub mod egg_support;
pub mod python;
pub mod repython;
pub mod lisp;
pub mod relisp;
pub mod javascript;
pub mod rejavascript;


use log::*;

/// 树指针的方式打印
pub fn print_tree_sitter(
    cursor: &tree_sitter::TreeCursor,
    code: &str,
    indent_level: usize,
) {
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
