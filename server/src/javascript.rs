use log::*;
use tree_sitter::{Parser, Node, TreeCursor};

/// 树形递归打印
#[allow(unused)]
fn print_node(node: &Node, code: &str, indent_level: usize) {
    let indent = "|   ".repeat(indent_level);
    let start = node.start_position();
    let end = node.end_position();
    debug!(
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
}


/// 树指针的方式打印
fn print_tree(
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
        print_tree(&cursor, code, indent_level + 1);
        while cursor.goto_next_sibling() {
            print_tree(&cursor, code, indent_level + 1);
        }
        cursor.goto_parent();
    }
}

fn ast_to_sexpr(
    tree_cursor: &TreeCursor,
    code: &str,
) -> String {
	let node = tree_cursor.node();
    match node.kind() {
        // 逻辑常量
        "true" => "true".to_string(),
        "false" => "false".to_string(),

        "number" => node.utf8_text(code.as_bytes()).unwrap().to_string(),
        "identifier" => {
            let var = node.utf8_text(code.as_bytes()).unwrap().to_string();
            format!("(var {})", var)
        }
        // 一元表达式
        "unary_expression" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let op = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling();
            let value = ast_to_sexpr(&children, code);
            match op {
                "!" => format!("(~ {})", value),
                "-" => format!("(- 0 {})", value),
                _ => format!("<错误 unhandled unary op kind: ({:?} {:?})>", op, value),
            }
        }

        // 二元表达式
        "binary_expression" | "boolean_expression" | "comparison_expression" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let left = ast_to_sexpr(&children, code);
            children.goto_next_sibling();
            let op = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling();
            let right = ast_to_sexpr(&children, code);
            let op = match op {
                "&&" => "&",
                "||" => "|",
                "==" => "=",
                _ => op,
            };
            format!("({} {} {})", op, left, right)
        }

        // let
        // 目前用 seqlet
		// (seq (seqlet a 1) (var a))
        "assignment_expression" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let name = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling();
            // 跳过 `=`
			children.goto_next_sibling();
			let value = ast_to_sexpr(&children, code);
			assert_eq!(children.goto_next_sibling(), false);
			format!("(seqlet {} {})", name, value)

        }

        // function_definition
        // 目前用 seqlet
		// (seqlet a (lam _ _))
        "function_declaration" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `function` (javascript)
            children.goto_next_sibling();
            let name = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling();
            let parameters = ast_to_sexpr(&children, code);
            children.goto_next_sibling();
            let body = ast_to_sexpr(&children, code);
			assert_eq!(children.goto_next_sibling(), false);
            format!(
                "(seqlet {} (lam {} {}))",
                name,                                   // 函数名
                parameters,                             // 参数
                body                                    // 函数体
            )
        }
        "formal_parameters" => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);

			// TODO 0 个参数的情况

            // 只有一个参数 
            children.next(); // 跳过 `(` (javascript)
            let name = children.next().unwrap().utf8_text(code.as_bytes()).unwrap();

			// TODO 支持多个参数 laml

            name.to_string()
        }
        "call_expression" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let name = ast_to_sexpr(&children, code);
            children.goto_next_sibling();
            let args = ast_to_sexpr(&children, code);
			
            format!("(app {} {})", name, args)
        }
        "arguments" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `(` (javascript)
            children.goto_next_sibling();
            //TODO: 先假设只有一个参数 | 支持多个参数 通过柯里化实现
            let arg = ast_to_sexpr(&children, code);
            arg
        }
        "return_statement" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `return` (javascript)
            children.goto_next_sibling();
            let value = ast_to_sexpr(&children, code);
            format!("{}", value)
        }

        // 表达式语句
        "expression_statement" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            format!("{}", ast_to_sexpr(&children, code))
        }

        // 流程控制语句
        "if_statement" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `if` (python)
            children.goto_next_sibling();
            let cond = ast_to_sexpr(&children, code);
            children.goto_next_sibling();
            let then = ast_to_sexpr(&children, code); // "block"
            if children.goto_next_sibling() == false {
                return format!("(if {} {} skip)", cond, then); // 返回空
            } else {
                let else_ = ast_to_sexpr(&children, code);
                return format!("(if {} {} {})", cond, then, else_);
            }
        }
        // "else_clause" => {
        //     let mut children = tree_cursor.clone();
        //     children.goto_first_child();
        //     // 跳过 `else` (python)
        //     children.goto_next_sibling();
        //     // 跳过 `:` (python)
        //     children.goto_next_sibling();
        //     return ast_to_sexpr(&children, code);
        // }

        // 块
		// seq 实现
		// (seq ... (seq ...))
        "program" => {
            let mut children = tree_cursor.clone();
            if children.goto_first_child() {
				let mut sexpr = format!("{}", ast_to_sexpr(&children, code));
				while children.goto_next_sibling() {
					sexpr = format!("(seq {} {})", sexpr, ast_to_sexpr(&children, code));
				}
				sexpr
			} else {
				format!("")
			}
        }
		"statement_block" => {
			let mut children = tree_cursor.clone();
			children.goto_first_child();
			// 跳过 `{` (javascript)
			children.goto_next_sibling();
			let mut sexpr = format!("{}", ast_to_sexpr(&children, code));
			while children.goto_next_sibling() && children.node().kind() != "}" {
				sexpr = format!("(seq {} {})", sexpr, ast_to_sexpr(&children, code));
			}
			sexpr
		}
		"parenthesized_expression" => {
			let mut children = tree_cursor.clone();
			children.goto_first_child();
			// 跳过 `(` (javascript)
			children.goto_next_sibling();
			return ast_to_sexpr(&children, code);
		}
		// 面向对象特性
		// 变成函数调用
		// "member_expression":193  [10:0 - 10:11]  
		// |   "identifier":1  [10:0 - 10:7] console 
		// |   ".":47  [10:7 - 10:8] . 
		// |   "property_identifier":244  [10:8 - 10:11] log 
		"member_expression" => {
			let mut children = tree_cursor.clone();
			children.goto_first_child();
			let identifier = children.node().utf8_text(code.as_bytes()).unwrap();
			children.goto_next_sibling();
			// 跳过 `.` (javascript)
			children.goto_next_sibling();
			let property_identifier = children.node().utf8_text(code.as_bytes()).unwrap();
			format!("(var {}.{})", identifier, property_identifier)

		}

        // 杂项 & 语言特性
        "comment" | "empty_statement" => {
            format!("skip") // 最好是返回空
        }

        &_ => {
            format!("<发生错误 unhandled node kind: {:?}>", node.kind())
        }
    }
}

use crate::egg_support::simplify;

pub fn js_parser(s: &str) -> Result<String, String> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_javascript::language()).unwrap();
    let tree = parser.parse(s, None).unwrap();
    let root_node = tree.root_node();

    debug!("Root node: \n{:?}", &root_node);
    debug!("sexp: \n{:?}", &root_node.to_sexp());

    let tree_cursor = tree.walk();
    debug!("tree_cursor 方式打印:");
    print_tree(&tree_cursor, s, 0);

    let sexpr = ast_to_sexpr(&tree_cursor, s);
    info!("sexpr: \n{}", &sexpr);
    match simplify(&sexpr.as_str()) {
        Ok(sexp) => match sexp {
            Some(sexp) => Ok(sexp.to_string()),
            None => Ok("已经最优了".to_string()),
        },
        Err(e) => Err(format!("egg error: {}", e)),
    }
}


#[test]
fn ast_test() {
	use crate::egg_support::*;
    // python 额外注意空格与 tab 是不一样的！
    let code: &str = r#"
	function fibonacci(n) {
		if (! (n > 0)) {
			return -1
		}
		if (n == 1 || n == 2) {
			return 1
		}
		return fibonacci(n - 2) + fibonacci(n - 1);
	}
	fibonacci(30)
    "#.trim();
	println!("code: \n{}", code);
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_javascript::language()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let root_node = tree.root_node();

    println!("Root node: \n{:?}", &root_node);
    println!("sexp: \n{:?}", &root_node.to_sexp());

    let tree_cursor = tree.walk();
    println!("tree_cursor 方式打印:");
    print_tree(&tree_cursor, code, 0);

	let s = ast_to_sexpr(&tree_cursor, code);
    println!("my sexp: \n{:?}", s);

	println!("pretty sexp: \n{}", rpn_to_string(&s.parse().unwrap(), rpn_helper_simple).unwrap());
}

