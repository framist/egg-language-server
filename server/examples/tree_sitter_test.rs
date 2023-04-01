//! TODO

use tree_sitter::{Node, Parser};

fn print_node(node: &Node, code: &str, indent_level: usize) {
    let indent = "|   ".repeat(indent_level);
    let start = node.start_position();
    let end = node.end_position();
    println!(
        "{}{:?}:{}  [{}:{} - {}:{}] {}",
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
        }
    );
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_node(&child, code, indent_level + 1);
    }
	// let mut cursor = node.walk();
	// loop {
	// 	print_node(&cursor.node(), code, indent_level + 1);
	// 	if ! cursor.goto_next_sibling() {
	// 		break;
	// 	}
	// }
	
}

const CODE: &str = r#"
(lam x (+ 4
	(app (lam y (var y))
		 4)))
"#;

// const CODE: &str = r#"
// def add(x):
//     return x + 1
// y = 1
// add(y)
// "#;

fn main() {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_scheme::language()).unwrap();
    let tree = parser.parse(CODE, None).unwrap();
    let root_node = tree.root_node();

    println!("Root node: {:?}", &root_node);
    println!("sexp: {:?}", &root_node.to_sexp());

    print_node(&root_node, CODE, 0);

}
