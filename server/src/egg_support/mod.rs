mod common;

pub type EggIR = egg::RecExpr<common::CommonLanguage>;

pub fn simplify(s: &str) -> Result<Option<EggIR>, String> {
    common::simplify(s)
}

pub use common::CommonLanguage;

pub fn rpn_to_human(
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
        op @ (Add(_) | Sub(_) | Mul(_) | Div(_) | Pow(_) | And(_) | Or(_) | Gt(_) | Ge(_)
        | Lt(_) | Le(_) | Ne(_)) => {
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
            format!("{} {}: {}", "Y", then, body)
        }
        // List
        Cons(_) => {
            let right = stack.pop().ok_or(&err)?;
            let left = stack.pop().ok_or(&err)?;
            format!("{} :: {}", left, right)
        }
        Nil => "nil".to_string(),
        LambdaL(_) => {
            let body = stack.pop().ok_or(&err)?;
            let varl = stack.pop().ok_or(&err)?;
            format!("(λ {}:\n{})", varl, add_widths(body))
        }
        AppL(_) => {
            let body = stack.pop().ok_or(&err)?;
            let f = stack.pop().ok_or(&err)?;

            format!("{}({})", f, body)
        }
        Seq(_) => {
            let then = stack.pop().ok_or(&err)?;
            let body = stack.pop().ok_or(&err)?;
            format!("{};;\n{}", body, then)
        }
        Skip => "SKIP".to_string(),
        SeqLet(_) => {
            let body = stack.pop().ok_or(&err)?;
            let var = stack.pop().ok_or(&err)?;
            format!("let {} = {}", var, body)
        }
        While(_) => {
            let body = stack.pop().ok_or(&err)?;
            let cond = stack.pop().ok_or(&err)?;
            format!("while {}:\n{}", cond, add_widths(body))
        }
        For(_) => {
            let body = stack.pop().ok_or(&err)?;
            let update = stack.pop().ok_or(&err)?;
            let cond = stack.pop().ok_or(&err)?;
            let init = stack.pop().ok_or(&err)?;
            format!("for {}; {}; {}:\n{}", init, cond, update, add_widths(body))
        }
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

#[cfg(test)]
fn direct_parser(s: &str) -> Result<String, String> {
    common::simplify_test(s)
}

#[cfg(test)]
fn simple_reparser(s: &String) -> Result<String, String> {
    match s.parse::<EggIR>() {
        Ok(rpn) => rpn_to_human(&rpn, rpn_helper_simple),
        Err(e) => return Err(format!("egg-IR parse error: {}", e)),
    }
}

/// rpn_to_string 测试
#[test]
#[rustfmt::skip]
fn rpn_to_string_test() {
    let test_helper = |a: &str, b: &str| {
        assert_eq!(
            rpn_to_human(&a.parse().unwrap(), rpn_helper_simple).unwrap(),
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
(λ x:
    (x + 4))"
    );
    // 多参函数
test_helper(
"(seq (seqlet f (laml (cons x (cons y nil)) (laml (var x) (+ 42 (appl (laml (var y) (var y)) (cons 24 nil)))))) (appl (appl (var f) (cons 2 (cons 3 nil))) (cons 6 nil)))",
r"
let f = (λ x :: y :: nil:
    (λ `x`:
        (42 + (λ `y`:
            `y`)(24 :: nil))));;
`f`(2 :: 3 :: nil)(6 :: nil)"
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
        rpn_to_human(&s.parse().unwrap(), rpn_helper_simple).unwrap()
    );
    // 优化后
    println!("[*]simply:\n{}", direct_parser(s).unwrap());
}

#[test]
fn curry_temp_test() {
    let s = "(let add 
                (laml (cons x (cons y nil)) 
                    (+ (var x) (var y)))
                (appl (var add) (cons 1 (cons 2 nil)) ))";
    println!("[*]pretty:\n{}", s.parse::<EggIR>().unwrap().pretty(20));
    println!(
        "[*]rpn_to_string:\n{}",
        rpn_to_human(&s.parse().unwrap(), rpn_helper_simple).unwrap()
    );
    // 优化后
    println!("[*]simply:\n{}", direct_parser(s).unwrap());
} // (cons x (cons y nil)) 不能取代默认为原子Var的地方，不然有错误

#[test]
fn imperative_temp_test() {
    let s = "(seq skip (seq skip nil))";
    println!("[*]pretty:\n{}", s.parse::<EggIR>().unwrap().pretty(20));
    println!(
        "[*]rpn_to_string:\n{}",
        rpn_to_human(&s.parse().unwrap(), rpn_helper_simple).unwrap()
    );
    // 优化后
    println!("[*]simply:\n{}", direct_parser(s).unwrap());
}

#[test]
fn imperative_temp_test2() {
    let s = "(seq (seqlet a 1) (seq (var a) nil))";
    println!("[*]pretty:\n{}", s.parse::<EggIR>().unwrap().pretty(20));
    println!(
        "[*]rpn_to_string:\n{}",
        rpn_to_human(&s.parse().unwrap(), rpn_helper_simple).unwrap()
    );
    // 优化后
    println!("[*]simply:\n{}", direct_parser(s).unwrap());
}

// ? String 竟然可以正确处理而没有发生错误
#[test]
fn temp_test2() {
    let s = r#"(app (var print) "hello World (+ 1 1)")"#;
    println!("[*]pretty:\n{}", s.parse::<EggIR>().unwrap().pretty(20));
    println!(
        "[*]rpn_to_string:\n{}",
        simple_reparser(&s.to_string()).unwrap()
    );
    // 优化后
    println!("[*]simply:\n{}", direct_parser(s).unwrap());
}

#[test]
fn list_temp_test1() {
    let s = "(my 1 2 3 4)";
    println!("[*]pretty:\n{}", s.parse::<EggIR>().unwrap().pretty(20));
    println!(
        "[*]rpn_to_string:\n{}",
        rpn_to_human(&s.parse().unwrap(), rpn_helper_simple).unwrap()
    );
    // 优化后
    println!("[*]simply:\n{}", direct_parser(s).unwrap());
}

#[test]
fn temp_test1() {
    let s = "(seq (seqlet f (laml (cons x (cons y nil)) (laml (var x) (+ 42 (appl (laml (var y) (var y)) (cons 24 nil)))))) (appl (appl (var f) (cons 2 (cons 3 nil))) (cons 6 nil)))";
    println!("[*]pretty:\n{}", s.parse::<EggIR>().unwrap().pretty(20));
    println!(
        "[*]rpn_to_string:\n{}",
        rpn_to_human(&s.parse().unwrap(), rpn_helper_simple).unwrap()
    );
    // 优化后
    println!("[*]simply:\n{}", direct_parser(s).unwrap());
}
