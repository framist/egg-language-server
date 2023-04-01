//! TODO
use tree_sitter::{Parser, Node};

fn print_node(node: Node, code: &str, indent_level: usize) {
    let indent = "    ".repeat(indent_level);
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
        print_node(child, code, indent_level + 1);
    }

}

// fn my_ast_to_sexpr(node: Node, code: &str) -> String {
//     // 递归地转换为自定义的 s-expr
//     match node.kind() {
        
//     }
    
// }

fn main() {
    let code = r#"
        function double(x) {
            return x * 2;
        }
    "#;

    let mut parser = Parser::new();
    parser.set_language(tree_sitter_javascript::language()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let root_node = tree.root_node();

    println!("Root node: {:?}", root_node);
    println!("Roor sexp: {:?}", root_node.to_sexp()); 

    print_node(root_node, code, 0);

}

#[test]
fn my_test() {
    println!("{:?}", )
}
