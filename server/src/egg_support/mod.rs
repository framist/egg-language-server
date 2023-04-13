mod common;

pub type EggIR = egg::RecExpr<common::CommonLanguage>;

pub fn direct_parser(s: &str) -> Result<String, String> {
    common::simplify_test(s)
}

pub fn simple_reparser(s: &String) -> Result<String, String> {
    match s.parse::<EggIR>() {
        Ok(rpn) => rpn_to_string(&rpn, rpn_helper_simple),
        Err(e) => return Err(format!("egg-IR parse error: {}", e)),
    }
}

pub fn simplify(s: &str) -> Result<Option<EggIR>, String> {
    common::simplify(s)
}

// trait CommonLanguageTrans {
//     fn ast_to_sexpr(tree: &tree_sitter::Tree, tree_cursor: &tree_sitter::TreeCursor, code: &str) -> String;
//     fn rpn_helper(token: &common::CommonLanguage, stack: &mut Vec<String>) -> Result<String, String>;
// }

// pub struct SimpleLanguage {

// }

pub use common::CommonLanguage;

pub fn rpn_to_string(
    rpn: &EggIR,
    rpn_helper: fn(token: &CommonLanguage, stack: &mut Vec<String>) -> Result<String, String>,
) -> Result<String, String> {
    let mut stack = Vec::new();
    let err = "RPN has invalid format".to_string();
    // println!("rpn = {:?}", rpn.as_ref());
    for token in rpn.as_ref() {
        let exp = rpn_helper(token, &mut stack)?;
        stack.push(exp);
    }

    if stack.len() != 1 {
        return Err(err);
    }

    stack.pop().ok_or(err)
}

pub fn rpn_helper_simple(
    token: &CommonLanguage,
    stack: &mut Vec<String>,
) -> Result<String, String> {
    let err = format!("RPN has invalid format: token = {:?}", token);
    let width = "    "; // 后续考虑需从编辑器环境中获取 width 信息
    let add_widths = |s: String| {
        s.lines()
            .map(|line| format!("{}{}", width, line))
            .collect::<Vec<_>>()
            .join("\n")
    };
    use CommonLanguage::*;
    Ok(match token {
        #[cfg(feature = "float")]
        Constant(f64) => f64.to_string(),
        Num(val) => val.to_string(),
        Bool(val) => val.to_string(),
        Symbol(s) => s.to_string(),
        // 一元运算符
        op @ (Ln(_) | Sqrt(_)) => {
            let exp = stack.pop().ok_or(&err)?;
            format!("{}({})", op.to_string(), exp)
        }
        Not(_) => {
            let exp = stack.pop().ok_or(&err)?;
            format!("not {}", exp)
        }
        // Neg(_) => {
        //     let exp = stack.pop().ok_or(&err)?;
        //     format!("-{}", exp)
        // }
        // 二元运算符
        op @ (Add(_) | Sub(_) | Mul(_) | Div(_) | Pow(_) | And(_) 
        | Or(_) | Gt(_) | Ge(_) | Lt(_) | Le(_) | Ne(_)) => {
            let right = stack.pop().ok_or(&err)?;
            let left = stack.pop().ok_or(&err)?;
            format!("({} {} {})", left, op.to_string(), right)
        }
        Var(_) => {
            let var = stack.pop().ok_or(&err)?;
            format!("`{}`", var)
        }
        Lambda(_) => {
            let body = stack.pop().ok_or(&err)?;
            let var = stack.pop().ok_or(&err)?;
            format!("(λ {}:\n{})", var, add_widths(body))
        }
        App(_) => {
            let right = stack.pop().ok_or(&err)?;
            let f = stack.pop().ok_or(&err)?;

            format!("{}({})", f, right)
        }
        Let(_) => {
            let then = stack.pop().ok_or(&err)?;
            let body = stack.pop().ok_or(&err)?;
            let var = stack.pop().ok_or(&err)?;
            format!("let {} = {};\n{}", var, body, then)
        }
        If(_) => {
            let else_exp = stack.pop().ok_or(&err)?;
            let then_exp = stack.pop().ok_or(&err)?;
            let cond = stack.pop().ok_or(&err)?;
            format!(
                "if {}:\n{}\nelse:\n{}",
                cond,
                add_widths(then_exp),
                add_widths(else_exp)
            )
        }
        Eq(_) => {
            let right = stack.pop().ok_or(&err)?;
            let left = stack.pop().ok_or(&err)?;
            format!("{} {} {}", left, "==", right)
        }
        Fix(_) => {
            // TODO
            let body = stack.pop().ok_or(&err)?;
            let then = stack.pop().ok_or(&err)?;
            format!("{} {} {}", "fixpoint", then, body)
        }
    })
}

// 正儿八经的测试
#[test]
fn rpn_to_string_test() {
    // 数学运算
    assert_eq!(
        rpn_to_string(&"(+ 1 2)".parse().unwrap(), rpn_helper_simple).unwrap(),
        "(1 + 2)"
    );
    assert_eq!(
        rpn_to_string(
            &"(+ 1 (- a (* a (+ 2 -1))))".parse().unwrap(),
            rpn_helper_simple
        )
        .unwrap(),
        "(1 + (a - (a * (2 + -1))))"
    );
    // 控制流
    assert_eq!(
        rpn_to_string(&"(if (= 1 2) 3 4)".parse().unwrap(), rpn_helper_simple).unwrap(),
        r"
if 1 == 2:
    3
else:
    4"
        .to_string()
        .trim()
    );
    assert_eq!(
        rpn_to_string(&"(if (= 1 2) 3 4)".parse().unwrap(), rpn_helper_simple).unwrap(),
        r"
if 1 == 2:
    3
else:
    4"
        .to_string()
        .trim()
    );
    // lambda
    assert_eq!(
        rpn_to_string(&"(lam x (+ x 4))".parse().unwrap(), rpn_helper_simple).unwrap(),
        r"
(λ x:
    (x + 4))"
            .to_string()
            .trim()
    );
    // mix
    assert_eq!(
        rpn_to_string(
            &"(let fib (fix fib (lam n
                (if (= (var n) 0)
                    0
                (if (= (var n) 1)
                    1
                (+ (app (var fib)
                        (+ (var n) -1))
                    (app (var fib)
                        (+ (var n) -2)))))))
                (app (var fib) 4))"
                .parse()
                .unwrap(),
            rpn_helper_simple
        )
        .unwrap(),
        r"
let fib = fixpoint fib (λ n:
    if `n` == 0:
        0
    else:
        if `n` == 1:
            1
        else:
            (`fib`((`n` + -1)) + `fib`((`n` + -2))));
`fib`(4)"
            .to_string()
            .trim()
    );
}

#[test]
fn lisp_temp_test() {
    let s = "(let fib (fix fib (lam n
        (if (= (var n) 0)
            0
        (if (= (var n) 1)
            1
        (+ (app (var fib)
                (+ (var n) -1))
            (app (var fib)
                (+ (var n) -2)))))))
        (app (var fib) 4))";
    println!("[*]pretty:\n{}", s.parse::<EggIR>().unwrap().pretty(20));
    println!(
        "[*]rpn_to_string:\n{}",
        rpn_to_string(&s.parse().unwrap(), rpn_helper_simple).unwrap()
    );
    // 优化后
    println!("simply:\n{}", direct_parser(s).unwrap());
}
