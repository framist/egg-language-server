use egg::*;

// 该函数定义语言：SimpleLanguage。
// 它包括 Num、加号"+"(Add、两个 Id 标志符参数)、
// "*" 乘号 (Mul、两个 Id 标志符参数) 以及 Symbol 标记。
define_language! {
    enum SimpleLanguage {
        Num(i32),
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        Symbol(Symbol),
    }
}

// 这段代码的作用是创建一个 Vec，里面包含了 4 条重写规则，
// 用于对 SimpleLanguage 语言中的表达式进行重写。
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
fn simplify() {
    let s = "(+ 1 1)";
    // 解析表达式，类型注释 (<Language>) 告诉它使用哪种语言
    // 实际上 RecExpr<SimpleLanguage> 是一个 逆波兰表达式的结构
    let expr: RecExpr<SimpleLanguage> = s.parse().unwrap();

    // 使用 Runner 简化表达式，该运行器创建带有
    // 给定的表达式的 e-graph，并在其上运行给定的规则
    let runner = Runner::default().with_expr(&expr).run(&make_rules());

    // Runner 知道用 with_expr 给出的表达式在哪个 e-class 中
    let root = runner.roots[0];

    // 使用提取器 extractor 选择 根 eclass 的最佳元素
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (best_cost, best) = extractor.find_best(root);
    println!("RecExpr<SimpleLanguage>:\n {:?}", expr);
    println!("rpn to string: \n{}", rpn_to_string(&expr).unwrap());
    println!("rpn to string best: \n{:?}", rpn_to_string(&best.to_string().parse().unwrap())); // to_string().parse() 是必要的
    println!("best_cost: {}", best_cost);
    println!("best pretty: \n{}", best.pretty(10));

    // cost  的变化
    let cost_orgi = AstSize.cost_rec(&expr);
    let cost_best = AstSize.cost_rec(&best);
    if cost_orgi > cost_best {
        println!("cost_orgi: {} > cost_best: {}", cost_orgi, cost_best);
    }
}

fn rpn_to_string(rpn: &RecExpr<SimpleLanguage>) -> Result<String, &str> {
    let mut stack = Vec::new();
    for token in rpn.as_ref() {
        match token {
            SimpleLanguage::Num(val) => stack.push(val.to_string()),
            SimpleLanguage::Add(_) => {
                let right = stack.pop().ok_or("RPN has invalid format")?;
                let left = stack.pop().ok_or("RPN has invalid format")?;

                let exp = format!("({} {} {})", left, "+", right);
                stack.push(exp);
            }
            SimpleLanguage::Mul(_) => {
                let right = stack.pop().ok_or("RPN has invalid format")?;
                let left = stack.pop().ok_or("RPN has invalid format")?;

                let exp = format!("({} {} {})", left, "*", right);
                stack.push(exp);
            }
            SimpleLanguage::Symbol(s) => {
                stack.push(s.to_string());
            }
        }
    }

    if stack.len() != 1 {
        return Err("RPN has invalid format");
    }

    Ok(stack.pop().unwrap())
}


fn main() {
    simplify();
}
