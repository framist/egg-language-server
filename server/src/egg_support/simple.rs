use egg::*;

// 该函数定义语言: SimpleLanguage。
// 它包括 Num、加号"+"(Add、两个Id标志符参数)、
// "*" 乘号(Mul、两个Id标志符参数)以及Symbol标记.
define_language! {
    enum SimpleLanguage {
        Num(i32),
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        Symbol(Symbol),
    }
}

// 这段代码的作用是创建一个 Vec，里面包含了4条重写规则，
// 用于对SimpleLanguage语言中的表达式进行重写。
fn make_rules() -> Vec<Rewrite<SimpleLanguage, ()>> {
    vec![
        // 交换加法运算数顺序
        rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        // 交换乘法运算数顺序
        rewrite!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        // 加 0 = 本身
        rewrite!("add-0"; "(+ ?a 0)" => "?a"),
        // 乘 0 = 0
        rewrite!("mul-0"; "(* ?a 0)" => "0"),
        // 乘 1 = 本身
        rewrite!("mul-1"; "(* ?a 1)" => "?a"),
    ]
}

#[allow(unused)]
/// 解析一个表达式，使用 egg 对其进行简化，然后将其打印出来
pub fn simplify(s: &str) -> Result<String, String> {
    // 解析表达式，类型注释(<Language>)告诉它使用哪种语言
    // let expr: RecExpr<Language> = s.parse().unwrap();
    let expr: RecExpr<SimpleLanguage> = match s.parse() {
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
fn simple_tests() {
    assert_eq!(simplify("(* 0 42)").unwrap(), "0");
    assert_eq!(simplify("(+ 0 (* 1 foo))").unwrap(), "foo");
}

#[test]
fn my_tests() {
    println!("hello!");
    println!("{}", simplify("(* 0 42)").unwrap());
}
