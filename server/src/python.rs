// 用 tree-sitter 解析 python 代码
// note: 解析出的 IR 的树结构与 AST 不同。
/*
TODO fixpoint、有时优化不彻底的问题
def fib(x):
    if x < 0:
        return -1
    elif x == 0 or x == 1:
        return x
    else:
        return fib(x-1) + fib(x-2)
fib(5)

 */

use crate::*;
use log::*;
use tree_sitter::{Parser, TreeCursor};

// 类递归地转换为自定义的 s-expr
// 树指针的方式
// 随着 seq 的加入，可以用递归的方式了，不过还是用树指针实现（效率高）
fn ast_to_sexpr(tree_cursor: &TreeCursor, code: &str) -> String {
    let no_var_ast_to_sexpr = |tree_cursor: &TreeCursor, code: &str | {
        let node = tree_cursor.node();
        match node.kind() {
            "identifier" => {
                let var = node.utf8_text(code.as_bytes()).unwrap().to_string();
                format!("{}", var)
            }
            &_ => {
                format!("<发生错误 no_var_ast_to_sexpr: {:?}>", node.kind())
            }
        }
    };
    let node = tree_cursor.node();
    match node.kind() {
        // 逻辑常量
        "true" => "true".to_string(),
        "false" => "false".to_string(),

        "integer" => node.utf8_text(code.as_bytes()).unwrap().to_string(),
        "identifier" => {
            let var = node.utf8_text(code.as_bytes()).unwrap().to_string();
            format!("(var {})", var)
        }
        // 一元表达式
        "not_operator" | "unary_operator" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let op = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling();
            let value = ast_to_sexpr(&children, code);
            match op {
                "not" => format!("(~ {})", value),
                "-" => format!("-{}", value),
                _ => format!("<错误 unhandled op kind: ({:?} {:?})>", op, value),
            }
        }
        "parenthesized_expression" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `(` (python)
            children.goto_next_sibling();
            ast_to_sexpr(&children, code)
        }

        // 二元表达式
        "binary_operator" | "boolean_operator" | "comparison_operator" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let left = ast_to_sexpr(&children, code);
            children.goto_next_sibling();
            let op = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling();
            let right = ast_to_sexpr(&children, code);
            let op = match op {
                "and" => "&",
                "or" => "|",
                "==" => "=",
                _ => op,
            };
            format!("({} {} {})", op, left, right)
        }

        // let
        // 目前用 seqlet
        // (seq (seqlet a 1) (var a))
        "assignment" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let name = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling();
            // 跳过 `=` (python)
            children.goto_next_sibling();
            let value = ast_to_sexpr(&children, code);
            children.goto_next_sibling();
            format!("(seqlet {} {})", name, value)
        }

        // function_definition
        // 目前用 seqlet
        // (seqlet a (laml _ _))
        "function_definition" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `function` | `def` (python)
            children.goto_next_sibling();
            let name = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling();
            let parameters = ast_to_sexpr(&children, code);
            children.goto_next_sibling();
            // 跳过 `{` | `:` (python)
            children.goto_next_sibling();
            let body = ast_to_sexpr(&children, code);
            children.goto_next_sibling();
            format!(
                "(seqlet {} (laml {} {}))",
                name,       // 函数名
                parameters, // 参数
                body        // 函数体
            )
        }
        "lambda" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `lambda` (python)
            children.goto_next_sibling();
            let parameters = ast_to_sexpr(&children, code); 
            children.goto_next_sibling();
            // 跳过 `:` (python)
            children.goto_next_sibling();
            let body = ast_to_sexpr(&children, code);
            
            format!("(laml {} {})", parameters, body)
        }
        "call" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let name = ast_to_sexpr(&children, code);
            children.goto_next_sibling();
            let args = ast_to_sexpr(&children, code);

            format!("(appl {} {})", name, args)
        }
        // argument_list 是调用时，所以参数有 var
        "argument_list" => {
            let mut a = vec![];
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `(` (python)
            children.goto_next_sibling();
            while children.node().kind() != ")" {
                a.push(ast_to_sexpr(&children, code));
                children.goto_next_sibling();
                // 跳过 `,` (python)
                children.goto_next_sibling();
            }
            let mut args = "nil".to_string();
            for i in a.into_iter().rev() {
                args = format!("(cons {} {})", i, args);
            }
            args            
        }        
        "parameters" => {
            let mut a = vec![];
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `(` (python)
            children.goto_next_sibling();
            while children.node().kind() != ")" {
                a.push(no_var_ast_to_sexpr(&children, code));
                children.goto_next_sibling();
                // 跳过 `,` (python)
                children.goto_next_sibling();
            }
            let mut args = "nil".to_string();
            for i in a.into_iter().rev() {
                args = format!("(cons {} {})", i, args);
            }
            args
        }
        "lambda_parameters" => {
            let mut a = vec![];
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            loop {
                a.push(no_var_ast_to_sexpr(&children, code));
                if !children.goto_next_sibling() {
                    break;
                }
                // 跳过 `,` (python)
                children.goto_next_sibling();
            }
            let mut args = "nil".to_string();
            for i in a.into_iter().rev() {
                args = format!("(cons {} {})", i, args);
            }
            args
        }
        "return_statement" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `return` (python)
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
        //"if_statement":128  [0:0 - 1:9]
        // |   "if":23  [0:0 - 0:2] if
        // |   "identifier":1  [0:3 - 0:4] t
        // |   ":":24  [0:4 - 0:5] :
        // |   "block":154  [1:4 - 1:9]
        // |   |   "expression_statement":119  [1:4 - 1:9]
        // |   |   |   "assignment":178  [1:4 - 1:9]
        // |   |   |   |   "identifier":1  [1:4 - 1:5] x
        // |   |   |   |   "=":46  [1:6 - 1:7] =
        // |   |   |   |   "integer":92  [1:8 - 1:9] 0
        "if_statement" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `if` (python)
            children.goto_next_sibling();
            let cond = ast_to_sexpr(&children, code);
            children.goto_next_sibling();
            // 跳过 `:` (python)
            children.goto_next_sibling();
            let then = ast_to_sexpr(&children, code); // "block"
            if children.goto_next_sibling() == false {
                return format!("(if {} {} else_pass)", cond, then); // 返回空
            } else {
                let else_ = ast_to_sexpr(&children, code);
                return format!("(if {} {} {})", cond, then, else_);
            }
        }
        "else_clause" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `else` (python)
            children.goto_next_sibling();
            // 跳过 `:` (python)
            children.goto_next_sibling();
            return ast_to_sexpr(&children, code);
        }

        // 块
        "module" | "block" => {
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

        // 杂项 & 语言特性
        "pass_statement" => {
            format!("skip") // TODO 空值处理？
        }
        "comment" => {
            format!("")
        }

        &_ => {
            format!("<unhandled-node-kind-{}>", node.kind()) // TODO 直接表示为 cl
        }
    }
}

use crate::egg_support::simplify;

#[cfg(test)]
fn py_parser_all(s: &str) -> Result<String, String> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_python::language()).unwrap();
    let tree = parser.parse(s, None).unwrap();
    let root_node = tree.root_node();

    debug!("Root node: \n{:?}", &root_node);
    debug!("sexp: \n{:?}", &root_node.to_sexp());

    let tree_cursor = tree.walk();
    debug!("tree_cursor 方式打印:");
    print_tree_sitter(&&tree_cursor, s, 0);

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

// 分块 语法分析
// 解决粒度问题
pub fn py_parser(s: &str) -> Vec<EggDiagnostic> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_python::language()).unwrap();
    let tree = parser.parse(s, None).unwrap();
    let root_node = tree.root_node();

    debug!("Root node: \n{:?}", &root_node);
    debug!("sexp: \n{:?}", &root_node.to_sexp());

    let tree_cursor = tree.walk();
    debug!("tree_cursor 方式打印:");
    print_tree_sitter(&&tree_cursor, s, 0);

    let d = parser_batch_helper(&tree_cursor, s);
    debug!("diagnostics: \n{:?}", &d);
    d
}

// TODO 并行化 函数返回 simplify 的 vec handle， 在调用处 join
// use rayon::prelude::*;
// use std::thread;

fn parser_batch_helper(tree_cursor: &TreeCursor, code: &str) -> Vec<EggDiagnostic> {
    let node = tree_cursor.node();
    let mut diagnostics: Vec<EggDiagnostic> = Vec::new();

    // 原始
    // let mut children = tree_cursor.clone();
    // if children.goto_first_child() {
    //     loop {
    //         diagnostics.append(&mut parser_batch_helper(&children, code));
    //         if children.goto_next_sibling() == false {
    //             break;
    //         }
    //     }
    // }

    // 迭代器
    let children_diagnostics: Vec<Vec<EggDiagnostic>> = node
        .children(&mut node.walk())
        .map(|child| parser_batch_helper(&child.walk(), &code))
        .collect();
    for mut child_diagnostics in children_diagnostics {
        diagnostics.append(&mut child_diagnostics);
    }

    match node.kind() {
        // 递归终止点
        "module" | "block" | "expression_statement" | "comparison_operator" | "binary_operator" 
            if diagnostics.is_empty() =>
        {
            let sexpr = ast_to_sexpr(&tree_cursor, code);
            debug!("sexpr: \n{}", &sexpr);
            let tspan = node.range();
            let span = Range {
                start: Position {
                    line: tspan.start_point.row as u32,
                    character: tspan.start_point.column as u32,
                },
                end: Position {
                    line: tspan.end_point.row as u32,
                    character: tspan.end_point.column as u32,
                },
            };
            match simplify(&sexpr.as_str()) {
                Ok(sexp) => match sexp {
                    Some(s) => {
                        diagnostics.push(EggDiagnostic {
                            span,
                            reason: "Can be simplified".to_string(),
                            sexpr: Some(s.to_string()),
                            label: DiagnosticSeverity::INFORMATION,
                        });
                        return diagnostics;
                    }
                    None => {
                        return vec![];
                    }
                },
                Err(e) => {
                    diagnostics.push(EggDiagnostic {
                        span,
                        reason: format!("egg error: {}", e),
                        sexpr: None,
                        label: DiagnosticSeverity::ERROR,
                    });
                    return diagnostics;
                }
            }
        }
        _ => diagnostics,
    }
}

// * test *

#[test]
fn test_py_parser() {
    // python 额外注意空格与 tab 是不一样的！
    let code: &str = r#"
def add1(x):
    x = x + 1
    return x
y = 1
add1(y)
"#;
    assert_eq!(py_parser_all(code).unwrap(), "2");
}

#[test]
fn ast_test() {
    // python 额外注意空格与 tab 是不一样的！
    const CODE: &str = r#"
def add1(x):
    x = x + 1
    return x
y = 1
add1(y)
    "#;
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_python::language()).unwrap();
    let tree = parser.parse(CODE, None).unwrap();
    let root_node = tree.root_node();

    println!("Root node: \n{:?}", &root_node);
    println!("sexp: \n{:?}", &root_node.to_sexp());

    let tree_cursor = tree.walk();
    println!("tree_cursor 方式打印:");
    print_tree_sitter(&tree_cursor, CODE, 0);

    println!("my sexp: \n{:?}", ast_to_sexpr(&tree_cursor, CODE));
}
