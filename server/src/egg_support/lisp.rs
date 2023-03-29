use egg::{rewrite as rw, *};

// TODO 目前 仅仅简单地混合 lambda 与 math
// 不准备实现常数折叠

use ordered_float::NotNan;
pub type Constant = NotNan<f64>;

// 该函数定义语言: SimpleLanguage。
// 它包括 Num、加号"+"(Add、两个Id标志符参数)、
// "*" 乘号(Mul、两个Id标志符参数)以及Symbol标记.
define_language! {
    enum LispLanguage {
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

        Constant(Constant),

        // * Scheme
        "display" = Display(Id),

    }
}

// 这段代码的作用是创建一个 Vec，里面包含了4条重写规则，
// 用于对SimpleLanguage语言中的表达式进行重写。
fn make_rules() -> Vec<Rewrite<LispLanguage, ()>> {
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
#[allow(unused)]
pub fn simplify(s: &str) -> Result<String, String> {
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

#[test]
fn lisp_test() {
    println!("{:?}", simplify("(+ 0 (* 1 foo))") );
    println!("{:?}", simplify("(+ 1 1)") );
    println!("{:?}", simplify("(+ 1 (- a (* (- 2 1) a)))") );
    println!("{:?}", simplify("(lam x (+ 4
                                    (app (lam y (var y))
                                        4)))") );
}

#[test]
fn lisp_temp_test() {
    println!("{:?}", simplify(" (let add (lam x (+ (var x) 1))(app (var add) 1)))") );
}

