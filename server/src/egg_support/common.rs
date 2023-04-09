use egg::{rewrite as rw, *};

// TODO 目前 仅仅简单地混合 lambda 与 math
// 不准备实现常数折叠

use ordered_float::NotNan;
pub type Constant = NotNan<f64>;

// 该函数定义语言: SimpleLanguage。
// 它包括 Num、加号"+"(Add、两个Id标志符参数)、
// "*" 乘号(Mul、两个Id标志符参数)以及Symbol标记.
define_language! {
    pub enum CommonLanguage {
        Num(i32),
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        Symbol(Symbol),

        // * lambda

        Bool(bool),

        "var" = Var(Id),        // 声明之后为一个 var

        "=" = Eq([Id; 2]),

        "app" = App([Id; 2]),   // apply 使用lam函数 声明之后为一个lambda, 如果是函数名，则需var声明
        "lam" = Lambda([Id; 2]),
        "let" = Let([Id; 3]),
        "fix" = Fix([Id; 2]),

        "if" = If([Id; 3]),

        // * math

        // "d" = Diff([Id; 2]),
        // "i" = Integral([Id; 2]),

        "-" = Sub([Id; 2]),
        "/" = Div([Id; 2]),
        "pow" = Pow([Id; 2]),
        "ln" = Ln(Id),
        "sqrt" = Sqrt(Id),

        // Constant(Constant),

        // // * Scheme
        // "display" = Display(Id),

    }
}

// 这段代码的作用是创建一个 Vec，里面包含了4条重写规则，
// 用于对SimpleLanguage语言中的表达式进行重写。
fn make_rules() -> Vec<Rewrite<CommonLanguage, ()>> {
    vec![
        // 交换加法运算数顺序
        rw!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        // 交换乘法运算数顺序
        rw!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        // 加 0 = 本身
        rw!("add-0"; "(+ ?a 0)" => "?a"),
        // 乘 0 = 0
        rw!("mul-0"; "(* ?a 0)" => "0"),
        // 乘 1 = 本身
        rw!("mul-1"; "(* ?a 1)" => "?a"),
        // * lambda

        // open term rules
        rw!("if-true";  "(if  true ?then ?else)" => "?then"),
        rw!("if-false"; "(if false ?then ?else)" => "?else"),
        rw!("if-elim"; "(if (= (var ?x) ?e) ?then ?else)" => "?else"
            if ConditionEqual::parse("(let ?x ?e ?then)", "(let ?x ?e ?else)")),
        rw!("add-comm";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rw!("add-assoc"; "(+ (+ ?a ?b) ?c)" => "(+ ?a (+ ?b ?c))"),
        rw!("eq-comm";   "(= ?a ?b)"        => "(= ?b ?a)"),
        // subst 规则
        rw!("fix";      "(fix ?v ?e)"             => "(let ?v (fix ?v ?e) ?e)"),
        rw!("beta";     "(app (lam ?v ?body) ?e)" => "(let ?v ?e ?body)"),
        rw!("let-app";  "(let ?v ?e (app ?a ?b))" => "(app (let ?v ?e ?a) (let ?v ?e ?b))"),
        rw!("let-add";  "(let ?v ?e (+   ?a ?b))" => "(+   (let ?v ?e ?a) (let ?v ?e ?b))"),
        rw!("let-eq";   "(let ?v ?e (=   ?a ?b))" => "(=   (let ?v ?e ?a) (let ?v ?e ?b))"),
        // rw!("let-const";
        //     "(let ?v ?e ?c)" => "?c" if is_const(var("?c"))),
        rw!("let-if";
            "(let ?v ?e (if ?cond ?then ?else))" =>
            "(if (let ?v ?e ?cond) (let ?v ?e ?then) (let ?v ?e ?else))"
        ),
        rw!("let-var-same"; "(let ?v1 ?e (var ?v1))" => "?e"),
        // rw!("let-var-diff"; "(let ?v1 ?e (var ?v2))" => "(var ?v2)"
        //     if is_not_same_var(var("?v1"), var("?v2"))),
        rw!("let-lam-same"; "(let ?v1 ?e (lam ?v1 ?body))" => "(lam ?v1 ?body)"),
        // rw!("let-lam-diff";
        //     "(let ?v1 ?e (lam ?v2 ?body))" =>
        //     { CaptureAvoid {
        //         fresh: var("?fresh"), v2: var("?v2"), e: var("?e"),
        //         if_not_free: "(lam ?v2 (let ?v1 ?e ?body))".parse().unwrap(),
        //         if_free: "(lam ?fresh (let ?v1 ?e (let ?v2 (var ?fresh) ?body)))".parse().unwrap(),
        //     }}
        //     if is_not_same_var(var("?v1"), var("?v2"))),

        // * math
        rw!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rw!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
        rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        rw!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
        rw!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
        // rw!("div-canon"; "(/ ?a ?b)" => "(* ?a (pow ?b -1))" if is_not_zero("?b")),
        rw!("zero-add"; "(+ ?a 0)" => "?a"),
        rw!("zero-mul"; "(* ?a 0)" => "0"),
        rw!("one-mul";  "(* ?a 1)" => "?a"),
        rw!("add-zero"; "?a" => "(+ ?a 0)"),
        rw!("mul-one";  "?a" => "(* ?a 1)"),
        rw!("cancel-sub"; "(- ?a ?a)" => "0"),
        // rw!("cancel-div"; "(/ ?a ?a)" => "1" if is_not_zero("?a")),
        rw!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
        rw!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
        rw!("pow-mul"; "(* (pow ?a ?b) (pow ?a ?c))" => "(pow ?a (+ ?b ?c))"),
        // rw!("pow0"; "(pow ?x 0)" => "1"
        //     if is_not_zero("?x")),
        rw!("pow1"; "(pow ?x 1)" => "?x"),
        rw!("pow2"; "(pow ?x 2)" => "(* ?x ?x)"),
        // rw!("pow-recip"; "(pow ?x -1)" => "(/ 1 ?x)"
        //     if is_not_zero("?x")),
        // rw!("recip-mul-div"; "(* ?x (/ 1 ?x))" => "1" if is_not_zero("?x")),

        // rw!("d-variable"; "(d ?x ?x)" => "1" if is_sym("?x")),
        // rw!("d-constant"; "(d ?x ?c)" => "0" if is_sym("?x") if is_const_or_distinct_var("?c", "?x")),

        // rw!("d-add"; "(d ?x (+ ?a ?b))" => "(+ (d ?x ?a) (d ?x ?b))"),
        // rw!("d-mul"; "(d ?x (* ?a ?b))" => "(+ (* ?a (d ?x ?b)) (* ?b (d ?x ?a)))"),

        // rw!("d-sin"; "(d ?x (sin ?x))" => "(cos ?x)"),
        // rw!("d-cos"; "(d ?x (cos ?x))" => "(* -1 (sin ?x))"),

        // rw!("d-ln"; "(d ?x (ln ?x))" => "(/ 1 ?x)" if is_not_zero("?x")),

        // rw!("d-power";
        //     "(d ?x (pow ?f ?g))" =>
        //     "(* (pow ?f ?g)
        //         (+ (* (d ?x ?f)
        //               (/ ?g ?f))
        //            (* (d ?x ?g)
        //               (ln ?f))))"
        // ),

        // rw!("i-one"; "(i 1 ?x)" => "?x"),
        // rw!("i-cos"; "(i (cos ?x) ?x)" => "(sin ?x)"),
        // rw!("i-sin"; "(i (sin ?x) ?x)" => "(* -1 (cos ?x))"),
        // rw!("i-sum"; "(i (+ ?f ?g) ?x)" => "(+ (i ?f ?x) (i ?g ?x))"),
        // rw!("i-dif"; "(i (- ?f ?g) ?x)" => "(- (i ?f ?x) (i ?g ?x))"),
        // rw!("i-parts"; "(i (* ?a ?b) ?x)" =>
        //     "(- (* ?a (i ?b ?x)) (i (* (d ?x ?a) (i ?b ?x)) ?x))"),
    ]
}

/// 解析一个表达式，使用 egg 对其进行简化，然后将其打印出来
pub fn simplify_test(s: &str) -> Result<String, String> {
    // 解析表达式，类型注释(<Language>)告诉它使用哪种语言
    // let expr: RecExpr<Language> = s.parse().unwrap();
    let expr = match s.parse() {
        Ok(expr) => expr,
        Err(error) => return Err(format!("Failed to parse expression: {}", error)),
    };

    // 使用 Runner 简化表达式，该运行器创建带有
    // 给定的表达式的 e-graph ，并在其上运行给定的规则
    let runner = Runner::default().with_expr(&expr).run(&make_rules());

    // Runner 知道用 with_expr 给出的表达式在哪个 e-class 中
    let root = runner.roots[0];

    // 使用提取器 extractor 选择 根 eclass 的最佳元素
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (_best_cost, best) = extractor.find_best(root);
    Ok(best.to_string())
}

/// 解析一个表达式，使用 egg 对其进行简化，然后将其打印出来
/// - 如果解析失败，则返回错误
/// - 如果没有产生简化，则返回 None
/// - 如果产生了简化，则返回 Some(简化后的 RecExpr 逆波兰表达式)
pub fn simplify(s: &str) -> Result<Option<RecExpr<CommonLanguage>>, String> {
    let expr = match s.parse() {
        Ok(expr) => expr,
        Err(error) => return Err(format!("Failed to parse expression: {}", error)),
    };

    let runner = Runner::default().with_expr(&expr).run(&make_rules());

    // Runner 知道用 with_expr 给出的表达式在哪个 e-class 中
    let root = runner.roots[0];

    // 使用提取器 extractor 选择 根 eclass 的最佳元素
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (best_cost, best) = extractor.find_best(root);

    // cost  的变化
    if best_cost < AstSize.cost_rec(&expr) {
        Ok(Some(best))
    } else {
        Ok(None)
    }
}


fn rpn_to_string(rpn: &RecExpr<CommonLanguage>) -> Result<String, String> {
    let mut stack = Vec::new();
    let err = "RPN has invalid format".to_string();
    // println!("rpn = {:?}", rpn.as_ref());
    for token in rpn.as_ref() {
        let exp = rpn_helper_math(token, &mut stack)?;
        stack.push(exp);
    }

    if stack.len() != 1 {
        return Err(err);
    }

    stack.pop().ok_or(err)
}

fn rpn_helper_math(token: &CommonLanguage, stack: &mut Vec<String>) -> Result<String, String> {
    let err = format!("RPN has invalid format: token = {:?}", token);
    let width = "    ";
    use CommonLanguage::*;
    Ok(match token {
        Num(val) => val.to_string(),
        Bool(val) => val.to_string(),
        Symbol(s) => s.to_string(),
        op @ (Add(_) | Sub(_) | Mul(_) | Div(_) | Pow(_)) => {
            let right = stack.pop().ok_or(&err)?;
            let left = stack.pop().ok_or(&err)?;
            format!("({} {} {})", left, op.to_string(), right)
        }
        op @ (Ln(_) | Sqrt(_)) => {
            let exp = stack.pop().ok_or(&err)?;
            format!("({} {})", op.to_string(), exp)
        }
        Var(_) => {
            let var = stack.pop().ok_or(&err)?;
            format!("`{}`", var)
        }
        Lambda(_) => {
            let body = stack.pop().ok_or(&err)?;
            let var = stack.pop().ok_or(&err)?;
            // 为 body 增加缩进
            let body = body
                .lines()
                .map(|line| format!("{}{}", width, line))
                .collect::<Vec<_>>()
                .join("\n");
            format!("(λ {}:\n{})", var, body)
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
            format!("({} {} {} {})", "if", cond, then_exp, else_exp)
        }
        Eq(_) => {
            let right = stack.pop().ok_or(&err)?;
            let left = stack.pop().ok_or(&err)?;
            format!("({} {} {})", left, "==", right)
        }
        Fix(_) => {
            // TODO
            let body = stack.pop().ok_or(&err)?;
            let then = stack.pop().ok_or(&err)?;
            format!("({} {} {})", "fix", then, body)
        }
    })
}

#[test]
fn rpn_to_string_test() {
    println!("{}", rpn_to_string(&"(+ 1 2)".parse().unwrap()).unwrap());
    println!(
        "{}",
        rpn_to_string(&"(+ 1 (- a (* a (+ 2 -1))))".parse().unwrap()).unwrap()
    );
    println!(
        "{}",
        rpn_to_string(&"(lam x (+ x 4))".parse().unwrap()).unwrap()
    );
    println!(
        "{}",
        rpn_to_string(
            &"(let add1 (lam x (let x (+ (var x) 1) (var x))) (let y 1 (app (var add1) (var y))))"
                .parse()
                .unwrap()
        )
        .unwrap()
    );
}

#[test]
fn lisp_test() {
    println!("{:?}", simplify_test("(+ 0 (* 1 foo))"));
    println!("{:?}", simplify_test("(+ 1 1)"));
    println!("{:?}", simplify_test("(+ 1 (- a (* (- 2 1) a)))"));
    println!(
        "{:?}",
        simplify_test(
            "(lam x (+ 4
                (app (lam y (var y))
                    4)))"
        )
    );
}

#[test]
fn lisp_temp_test() {
    let s = "(let add1 (lam x (let x (+ (var x) 1) (var x))) (let y 1 (app (var add1) (var y))))";
    println!("[*]pretty:\n{}", s.parse::<RecExpr<CommonLanguage>>().unwrap().pretty(20));
    println!("[*]rpn_to_string:\n{}", rpn_to_string(&s.parse().unwrap()).unwrap());
    println!(
        "{:?}",
        simplify_test(s)
    );
}
