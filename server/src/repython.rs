//! egg-IR => python-AST => python
//! RecExpr<> 是一个 逆波兰表达式的结构

use crate::egg_support::*;

/// 因为
/// 	rpn_to_string(&best.to_string().parse().unwrap())
/// 这里 `to_string().parse()` 是必要的，RecExpr<> 可能遭到污染
/// 所以输入直接是 String 形式的 egg-IR
pub fn py_reparser(sexpr: &String) -> Result<String, String> {
    match sexpr.parse::<EggIR>() {
        Ok(rpn) => rpn_to_human(&rpn, rpn_helper_py),
        Err(e) => return Err(format!("egg-IR parse error: {}", e)),
    }
}

fn rpn_helper_py(token: &CommonLanguage, stack: &mut Vec<String>) -> Result<String, String> {
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
        Bool(val) => {
            match val {
                false => "False".to_string(),
                true => "True".to_string()
            }
        }
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
        op @ (Add(_) | Sub(_) | Mul(_) | Div(_) | Pow(_) | And(_) | Or(_) | Gt(_) | Ge(_)
        | Lt(_) | Le(_) | Ne(_)) => {
            let right = stack.pop().ok_or(&err)?;
            let left = stack.pop().ok_or(&err)?;
            format!("({} {} {})", left, op.to_string(), right)
        }
        Var(_) => {
            let var = stack.pop().ok_or(&err)?;
            format!("{}", var)
        }
        Lambda(_) => {
            let body = stack.pop().ok_or(&err)?;
            let var = stack.pop().ok_or(&err)?;
            format!("(lambda {}: {})", var, body)
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
            format!("{} = {}\n{}", var, body, then)
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
            format!("{} {}: {}", "Y", then, body)
        }
        // List
        Cons(_) => {
            let right = stack.pop().ok_or(&err)?;
            let left = stack.pop().ok_or(&err)?;
            if right == "nil" {
                format!("{}", left)
            } else {
                format!("{}, {}", left, right)
            }
        }
        Nil => "nil".to_string(),
        LambdaL(_) => {
            let body = stack.pop().ok_or(&err)?;
            let varl = stack.pop().ok_or(&err)?;
            format!("(lambda {}: {})", varl, body)
        }
        AppL(_) => {
            let body = stack.pop().ok_or(&err)?;
            let f = stack.pop().ok_or(&err)?;

            format!("{}({})", f, body)
        }
        Seq(_) => {
            let then = stack.pop().ok_or(&err)?;
            let body = stack.pop().ok_or(&err)?;
            format!("{}\n{}", body, then)
        }
        Skip => "pass".to_string(),
        SeqLet(_) => {
            let body = stack.pop().ok_or(&err)?;
            let var = stack.pop().ok_or(&err)?;
            format!("{} = {}", var, body)
        }
        While(_) => {
            let body = stack.pop().ok_or(&err)?;
            let cond = stack.pop().ok_or(&err)?;
            format!("while {}:\n{}", cond, add_widths(body))
        }
        For(_) => return Err(format!("un imp token = {}", "for")),
        Other(s, argids) => {
            let mut ans = stack.pop().ok_or(&err)?;
            for _ in 0..argids.len() - 1 {
                let arg = stack.pop().ok_or(&err)?;
                ans = arg + ", " + &ans;
            }
            format!("{}({})", s, ans)
        } // op @ _ => return Err(format!("un imp token = {:?}", op)),
    })
}

#[test]
#[rustfmt::skip]
fn rpn_to_string_test() {
    let test_helper = |a: &str, b: &str| {
        assert_eq!(
            rpn_to_human(&a.parse().unwrap(), rpn_helper_py).unwrap(),
            b.to_string().trim()
        );
    };
// 数学运算
test_helper("(+ 1 2)", "(1 + 2)");
test_helper("(+ 1 (- a (* a (+ 2 -1))))", "(1 + (a - (a * (2 + -1))))");
// 控制流
test_helper(
"(if (= 1 2) 3 4)",
r"
if 1 == 2:
    3
else:
    4"
);
    // lambda
test_helper(
"(lam x (+ x 4))",
r"
(lambda x: (x + 4))"
    );
    // 多参函数
test_helper(
"(seq (seqlet f (laml (cons x (cons y nil)) (laml (var x) (+ 42 (appl (laml (var y) (var y)) (cons 24 nil)))))) (appl (appl (var f) (cons 2 (cons 3 nil))) (cons 6 nil)))",
r"
f = (lambda x, y: (lambda x: (42 + (lambda y: y)(24))))
f(2, 3)(6)
"
);
}

#[test]
fn lisp_temp_test() {
    let s = "(let add1 (lam x (let x (+ (var x) 1) (var x))) (let y 1 (app (var add1) (var y))))";
    println!("[*]pretty:\n{}", s.parse::<EggIR>().unwrap().pretty(20));
    println!(
        "[*]rpn_to_string:\n{}",
        rpn_to_human(&s.parse().unwrap(), rpn_helper_py).unwrap()
    );
}

#[test]
fn lisp_temp_test2() {
    let s = "(seq (seqlet f (laml (cons x (cons y nil)) (laml (var x) (+ 42 (appl (laml (var y) (var y)) (cons 24 nil)))))) (appl (appl (var f) (cons 2 (cons 3 nil))) (cons 6 nil)))";
    println!("[*]pretty:\n{}", s.parse::<EggIR>().unwrap().pretty(20));
    println!(
        "[*]rpn_to_string:\n{}",
        rpn_to_human(&s.parse().unwrap(), rpn_helper_py).unwrap()
    );
}
