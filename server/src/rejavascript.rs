//! egg-IR => python-AST => python
//! RecExpr<> 是一个 逆波兰表达式的结构

use crate::egg_support::*;

/// 因为
/// 	rpn_to_string(&best.to_string().parse().unwrap())
/// 这里 `to_string().parse()` 是必要的 ，RecExpr<> 可能遭到污染
/// 所以输入直接是 String 形式的 egg-IR
pub fn js_reparser(sexpr: &String) -> Result<String, String> {
    match sexpr.parse::<EggIR>() {
        Ok(rpn) => rpn_to_human(&rpn, rpn_helper_js),
        Err(e) => return Err(format!("egg-IR parse error: {}", e)),
    }
}

// TODO 先做好 rpn_helper_simple 的测试
fn rpn_helper_js(token: &CommonLanguage, stack: &mut Vec<String>) -> Result<String, String> {
    rpn_helper_simple(token, stack)
}

#[test]
fn lisp_temp_test() {
    let s = "(let add1 (lam x (let x (+ (var x) 1) (var x))) (let y 1 (app (var add1) (var y))))";
    println!("[*]pretty:\n{}", s.parse::<EggIR>().unwrap().pretty(20));
    println!(
        "[*]rpn_to_string:\n{}",
        rpn_to_human(&s.parse().unwrap(), rpn_helper_js).unwrap()
    );
}
