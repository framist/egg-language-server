//! 用 tree-sitter 解析 python 代码
//! note: 解析出的 IR 的树结构与 AST 不同。

use tree_sitter::{Node, Parser};

/// 树形递归打印
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
}

/// 树指针的方式打印
fn print_tree(tree: &tree_sitter::Tree, cursor: &tree_sitter::TreeCursor, code: &str, indent_level: usize) {
    let indent = "|   ".repeat(indent_level);
    let node = cursor.node();
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
    let mut cursor = cursor.clone();
    if cursor.goto_first_child() {
        print_tree(tree, &cursor, code, indent_level + 1);
        while cursor.goto_next_sibling() {
            print_tree(tree, &cursor, code, indent_level + 1);
        }
        cursor.goto_parent();
    }
}

fn my_ast_to_sexpr(node: &Node, code: &str) -> String {
    // 递归地转换为自定义的 s-expr
    match node.kind() {
        "integer" => node.utf8_text(code.as_bytes()).unwrap().to_string(),
        "identifier" => {
            let var = node.utf8_text(code.as_bytes()).unwrap().to_string();
            format!("(var {})", var)
        }

        // 基本表达式
        "binary_operator" => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);
            let left = children.next().unwrap();
            let op = children.next().unwrap().utf8_text(code.as_bytes()).unwrap();
            let right = children.next().unwrap();
            format!(
                "({} {} {})",
                op,
                my_ast_to_sexpr(&left, code),
                my_ast_to_sexpr(&right, code)
            )
        }
        // TODO  let 应该形如 (let _ _ (...))
        // 最后一个参数 `then` 是上一级`expression_statement`的下一个同级节点
        // 例如
        // module [0, 0] - [3, 0]
        //     expression_statement [0, 0] - [0, 5]
        //         assignment [0, 0] - [0, 5]
        //         left: identifier [0, 0] - [0, 1]
        //         right: integer [0, 4] - [0, 5]
        //     expression_statement [1, 0] - [1, 5]
        //         assignment [1, 0] - [1, 5]
        //         left: identifier [1, 0] - [1, 1]
        //         right: integer [1, 4] - [1, 5]
        //     expression_statement [2, 0] - [2, 5]
        //         binary_operator [2, 0] - [2, 5]
        //         left: identifier [2, 0] - [2, 1]
        //         right: identifier [2, 4] - [2, 5]
        "assignment" => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);
            let name = children.next().unwrap().utf8_text(code.as_bytes()).unwrap();
            children.next(); // 跳过 `=` (python)
            let value = children.next().unwrap();

            // then 递归在不是 assignment 或 function_definition 中结束
            let mut then_cursor = node.walk();
            if ! then_cursor.goto_parent() {
                return format!(" then 提取出错: goto_parent");
            }
            if ! then_cursor.goto_next_sibling() {
                return format!(" then 提取出错: goto_next_sibling");
            }
            

            format!("(let {} {} {})",
                name,
                my_ast_to_sexpr(&value, code),
                my_ast_to_sexpr(&then_cursor.node(), code)
            )
        },
        // TODO  function_definition 应该形如 (let _ (lam _ _) (...))
        // 最后一个参数 `then` 是上一级`expression_statement`的下一个同级节点
        "function_definition" => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);
            children.next(); // 跳过 `function` | `def` (python)
            let name = children.next().unwrap().utf8_text(code.as_bytes()).unwrap();
            let parameters = children.next().unwrap();
            children.next(); // 跳过 `{` | `:` (python)
            let body = children.next().unwrap();

            // then 递归在不是 assignment 或 function_definition 中结束
            let mut then_cursor = node.walk();
            if ! then_cursor.goto_parent() {
                return format!(" then 提取出错: goto_parent");
            }
            if ! then_cursor.goto_next_sibling() {
                return format!(" then 提取出错: goto_next_sibling");
            }
                        
            format!("(let {} (lam {} {}) {})",
                name,                                       // 函数名
                my_ast_to_sexpr(&parameters, code),         // 参数
                my_ast_to_sexpr(&body, code),               // 函数体
                my_ast_to_sexpr(&then_cursor.node(), code)  // then
            )
        },
        "parameters" => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);

            //TODO: 先假设只有一个参数 | 支持多个参数 通过柯里化实现
            children.next(); // 跳过 `(` (python)
            let name = children.next().unwrap().utf8_text(code.as_bytes()).unwrap();
            name.to_string()
        }
        "call" => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);
            let name = children.next().unwrap();
            let args = children.next().unwrap();
            format!(
                "(app {} {})",
                my_ast_to_sexpr(&name, code),
                my_ast_to_sexpr(&args, code)
            )
        }
        "return_statement" => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);
            children.next(); // 跳过 `return` (python)
            let value = children.next().unwrap();
            format!("{}", my_ast_to_sexpr(&value, code))
        }

        // 表达式语句
        "expression_statement" => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);
            let value = children.next().unwrap();
            format!("{}", my_ast_to_sexpr(&value, code))
        }

        // 块
        // TODO 顺序结构问题 ？返回例如 (let _ _ (let ... )) 或者顺便解决粒度问题？
        // 对于 块语句，应该由多个嵌套的 let ，最终由一个有返回值的表达式结束
        //
        "module" | "block" => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);
            


            // 以下事务在 function_definition 或 assignment 中处理
            // 当 children 为 function_definition 或 assignment 时
            //      将其转换为 let 形式
            // 否则
            //      除非其是最后一个元素，则舍弃它
            // TODO 考虑是否舍弃
            // 例如
            // "a = 1;a" => "(let a 1 a)"
            // "a = 1; b = a; b" => "(let a 1 (let b a b))"

            // 所以只递归第一个元素就行
            let value = children.next().unwrap();
            format!("{}", my_ast_to_sexpr(&value, code))
            
            
        }

        &_ => {
            format!("发生错误 unhandled node kind: {:?}", node.kind())
            // // 返回空
            // " ".to_string()
        } // &_ => {
          //     let mut cursor = node.walk();
          //     let children = node.children(&mut cursor);
          //     let mut sexpr = format!("({} ", node.kind());
          //     // 返回每一个子节点的 s-expr
          //     sexpr += &children
          //         .map(|child| my_ast_to_sexpr(child, code))
          //         .collect::<Vec<String>>()
          //         .join(" ");
          //     sexpr += ")";
          //     sexpr
          // }
    }
}


/// 类递归地转换为自定义的 s-expr
/// 树指针的方式
fn ast_to_sexpr(tree: &tree_sitter::Tree, tree_cursor: &tree_sitter::TreeCursor, code: &str) -> String {
    let node = tree_cursor.node();
    match node.kind() {
        "integer" => node.utf8_text(code.as_bytes()).unwrap().to_string(),
        "identifier" => {
            let var = node.utf8_text(code.as_bytes()).unwrap().to_string();
            format!("(var {})", var)
        }

        // 基本表达式
        "binary_operator" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let left = ast_to_sexpr(tree, &children, code);
            children.goto_next_sibling();
            let op = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling();
            let right = ast_to_sexpr(tree, &children, code);
            format!("({} {} {})", op, left, right)
        }

        // let 
        // 应该形如 (let _ _ (...))
        // 最后一个参数 `then` 是上一级`expression_statement`的下一个同级节点
        // 例如
        // "module":105  [1:0 - 3:0] 
        // |   "expression_statement":119  [1:0 - 1:5] 
        // |   |   "assignment":178  [1:0 - 1:5] 
        // |   |   |   "identifier":1  [1:0 - 1:1] x
        // |   |   |   "=":46  [1:2 - 1:3] =
        // |   |   |   "integer":92  [1:4 - 1:5] 1
        // |   "expression_statement":119  [2:0 - 2:1] 
        // |   |   "identifier":1  [2:0 - 2:1] x
        "assignment" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let name = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling(); 
            // 跳过 `=` (python)
            children.goto_next_sibling();
            let value = ast_to_sexpr(tree, &children, code);


            // then 递归在不是 assignment 或 function_definition 中结束
            let mut then_cursor = tree_cursor.clone();
            if ! then_cursor.goto_parent() {
                return format!("assignment then 提取出错: goto_parent");
            }
            if ! then_cursor.goto_next_sibling() {
                return format!("assignment then 提取出错: goto_next_sibling");
            }
            

            format!("(let {} {} {})",
                name,
                value,
                ast_to_sexpr(tree, &mut then_cursor, code)
            )
        },

        // function_definition 
        // 应该形如 (let _ (lam _ _) (...))
        // 最后一个参数 `then` 是上一级`expression_statement`的下一个同级节点
        "function_definition" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `function` | `def` (python)
            children.goto_next_sibling();
            let name = children.node().utf8_text(code.as_bytes()).unwrap();
            children.goto_next_sibling();
            let parameters = ast_to_sexpr(tree, &children, code);
            children.goto_next_sibling();
            // 跳过 `{` | `:` (python)
            children.goto_next_sibling();
            let body = ast_to_sexpr(tree, &children, code);

            // then 递归在不是 assignment 或 function_definition 中结束
            let mut then_cursor = tree_cursor.clone();
            if ! then_cursor.goto_next_sibling() {
                return format!("function_definition then 提取出错: goto_next_sibling");
            }
                        
            format!("(let {} (lam {} {}) {})",
                name,                                       // 函数名
                parameters,                                 // 参数
                body,                                       // 函数体
                ast_to_sexpr(tree, &then_cursor, code)      // then
            )
        },
        "parameters" => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);

            //TODO: 先假设只有一个参数 | 支持多个参数 通过柯里化实现
            children.next(); // 跳过 `(` (python)
            let name = children.next().unwrap().utf8_text(code.as_bytes()).unwrap();
            name.to_string()
        }
        "call" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            let name = ast_to_sexpr(tree, &children, code);
            children.goto_next_sibling();
            let args = ast_to_sexpr(tree, &children, code);
            
            format!(
                "(app {} {})",
                name,
                args
            )
        }
        "argument_list" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `(` (python)
            children.goto_next_sibling();
            //TODO: 先假设只有一个参数 | 支持多个参数 通过柯里化实现
            let arg = ast_to_sexpr(tree, &children, code);
            arg
        }
        "return_statement" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            // 跳过 `return` (python)
            children.goto_next_sibling();
            let value = ast_to_sexpr(tree, &children, code);
            format!("{}", value)
        }

        // 表达式语句
        "expression_statement" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();
            format!("{}", ast_to_sexpr(tree, &children, code))
            
        }

        // 块
        // TODO 顺序结构问题 ？返回例如 (let _ _ (let ... )) 或者顺便解决粒度问题？
        // 对于 块语句，应该由多个嵌套的 let ，最终由一个有返回值的表达式结束
        //
        "module" | "block" => {
            let mut children = tree_cursor.clone();
            children.goto_first_child();


            // 以下事务在 function_definition 或 assignment 中处理
            // 当 children 为 function_definition 或 assignment 时
            //      将其转换为 let 形式
            // 否则
            //      除非其是最后一个元素，则舍弃它
            // TODO 考虑是否舍弃
            // 例如
            // "a = 1;a" => "(let a 1 a)"
            // "a = 1; b = a; b" => "(let a 1 (let b a b))"

            // 所以只递归第一个元素就行
            format!("{}", ast_to_sexpr(tree, &children, code))            
            
        }

        &_ => {
            format!("发生错误 unhandled node kind: {:?}", node.kind())
        } 
    }
}

/// ast 不定子节点修正
/// 针对 module | block
fn ast_muti_children(node: &Node, code: &str) {
    todo!()
}

/// ast 函数柯里化修正
/// 针对 function_definition
fn ast_currying() {
    todo!()
}


// const CODE: &str = r#"
// x = 1
// x
// "#;

// python 额外注意空格与 tab 是不一样的！
const CODE: &str = r#"
def add1(x):
    x = x + 1
    return x
y = 1
add1(y)
"#;

fn main() {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_python::language()).unwrap();
    let tree = parser.parse(CODE, None).unwrap();
    let root_node = tree.root_node();

    println!("Root node: \n{:?}", &root_node);
    println!("sexp: \n{:?}", &root_node.to_sexp());
    print_node(&root_node, CODE, 0);

    let tree_cursor = tree.walk();
    // println!("tree_cursor 方式打印:");
    // print_tree(&tree, &tree_cursor, CODE, 0);

    // println!("my sexp: \n{:?}", my_ast_to_sexpr(&root_node, CODE));
    println!("my sexp: \n{:?}", ast_to_sexpr(&tree, &tree_cursor, CODE));
}


// #[test]
// fn my_test() {
//     tree_sitter_python
// }