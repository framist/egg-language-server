use egg::{rewrite as rw, *};
// TODO 更多的控制流 比如 While

// 不准备实现浮点数的常数折叠
#[cfg(feature = "float")]
pub type Constant = ordered_float::NotNan<f64>;
#[cfg(feature = "float")]
define_language! {
    pub enum CommonLanguage {
        Num(i32),
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
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        "-" = Sub([Id; 2]),
        "/" = Div([Id; 2]),
        "pow" = Pow([Id; 2]),
        "ln" = Ln(Id),
        "sqrt" = Sqrt(Id),

        Constant(Constant),     // 浮点数常量

        // * Prop
        "&" = And([Id; 2]),
        "~" = Not(Id),
        "|" = Or([Id; 2]),

        // * Relation
        ">" = Gt([Id; 2]),
        "<" = Lt([Id; 2]),
        ">=" = Ge([Id; 2]),
        "<=" = Le([Id; 2]),
        "!=" = Ne([Id; 2]),
    }
}

// 该函数定义语言: SimpleLanguage。
// 它包括 Num、加号"+"(Add、两个Id标志符参数)、
// "*" 乘号(Mul、两个Id标志符参数)以及Symbol标记.
#[cfg(not(feature = "float"))]
define_language! {
    pub enum CommonLanguage {
        Num(i32),
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
        // "-" = Neg([Id; 1]),
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        "-" = Sub([Id; 2]),
        "/" = Div([Id; 2]),
        "pow" = Pow([Id; 2]),
        "ln" = Ln(Id),
        "sqrt" = Sqrt(Id),

        // Constant(Constant),     // 浮点数常量

        // * Prop
        "&" = And([Id; 2]),
        "~" = Not(Id),
        "|" = Or([Id; 2]),

        // * Relation
        ">" = Gt([Id; 2]),
        "<" = Lt([Id; 2]),
        ">=" = Ge([Id; 2]),
        "<=" = Le([Id; 2]),
        "!=" = Ne([Id; 2]),

        // TODO * List 注意，为了防止歧义，目前仅用于解决多参数问题；数据结构构建都应看作未定义的函数
        "cons" = Cons([Id; 2]),
        // "car" = Car(Id), 
        // "cdr" = Cdr(Id),
        "nil" = Nil,
        // 多参 的函数 (laml (cons x (cons y nil)) (+ x y))
        "laml" = LambdaL([Id; 2]),
        "appl" = AppL([Id; 2]),

        // TODO * Imp 指令式程序
        // "skip" = Skip,
        // "seq" = Seq([Id; 2]), 序列指令
    }
}

// 自定义的方法
impl CommonLanguage {
    fn num(&self) -> Option<i32> {
        match self {
            CommonLanguage::Num(n) => Some(*n),
            _ => None,
        }
    }
    fn bool(&self) -> Option<bool> {
        match self {
            CommonLanguage::Bool(b) => Some(*b),
            _ => None,
        }
    }
    #[cfg(feature = "float")]
    fn float_constant(&self) -> Option<Constant> {
        match self {
            CommonLanguage::Constant(c) => Some(*c),
            _ => None,
        }
    }
}

#[derive(Default)]
struct LambdaAnalysis;
type EGraph = egg::EGraph<CommonLanguage, LambdaAnalysis>;

use fxhash::FxHashSet as HashSet;

#[derive(Debug)]
struct Data {
    free: HashSet<Id>,
    constant: Option<(CommonLanguage, PatternAst<CommonLanguage>)>,
}

// TODO 要仔细检查算术溢出的情况 https://course.rs/compiler/pitfalls/arithmetic-overflow.html
fn eval(
    egraph: &EGraph,
    enode: &CommonLanguage,
) -> Option<(CommonLanguage, PatternAst<CommonLanguage>)> {
    let x = |i: &Id| egraph[*i].data.constant.as_ref().map(|c| &c.0);
    match enode {
        // * Lambda
        CommonLanguage::Num(n) => Some((enode.clone(), format!("{}", n).parse().unwrap())),
        CommonLanguage::Bool(b) => Some((enode.clone(), format!("{}", b).parse().unwrap())),
        CommonLanguage::Add([a, b]) => Some((
            CommonLanguage::Num(x(a)?.num()?.checked_add(x(b)?.num()?)?),
            format!("(+ {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        CommonLanguage::Eq([a, b]) => Some((
            CommonLanguage::Bool(x(a)? == x(b)?),
            format!("(= {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),

        // * math
        CommonLanguage::Mul([a, b]) => Some((
            CommonLanguage::Num(x(a)?.num()?.checked_mul(x(b)?.num()?)?),
            format!("(* {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        CommonLanguage::Sub([a, b]) => Some((
            CommonLanguage::Num(x(a)?.num()?.checked_sub(x(b)?.num()?)?),
            format!("(- {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        // CommonLanguage::Div([a, b]) if x(b)?.num()? != 0 => Some((  // 除数不能为0
        //     CommonLanguage::Num(x(a)?.num()? / x(b)?.num()?),   // 整数除法（不是的情况怎么办？）还是不要在整数集中实现除法了
        //     format!("(/ {} {})", x(a)?, x(b)?).parse().unwrap(),
        // )),
        // 不默认实现浮点数的常数折叠
        #[cfg(feature = "float")]
        CommonLanguage::Constant(c) => Some((enode.clone(), format!("{}", c).parse().unwrap())),
        #[cfg(feature = "float")]
        CommonLanguage::Add([a, b]) => Some((
            CommonLanguage::Constant(x(a)?.float_constant()? + x(b)?.float_constant()?),
            format!("(+ {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        #[cfg(feature = "float")]
        CommonLanguage::Sub([a, b]) => Some((
            CommonLanguage::Constant(x(a)?.float_constant()? - x(b)?.float_constant()?),
            format!("(- {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        #[cfg(feature = "float")]
        CommonLanguage::Mul([a, b]) => Some((
            CommonLanguage::Constant(x(a)?.float_constant()? * x(b)?.float_constant()?),
            format!("(* {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        #[cfg(feature = "float")]
        CommonLanguage::Div([a, b])
            if x(b)?.float_constant()? != (ordered_float::NotNan::new(0.0).unwrap()) =>
        {
            Some((
                CommonLanguage::Constant(x(a)?.float_constant()? / x(b)?.float_constant()?),
                format!("(/ {} {})", x(a)?, x(b)?).parse().unwrap(),
            ))
        }

        // * Prop
        CommonLanguage::And([a, b]) => Some((
            CommonLanguage::Bool(x(a)?.bool()? && x(b)?.bool()?),
            format!("(& {} {})", x(a)?.bool()?, x(b)?.bool()?)
                .parse()
                .unwrap(),
        )),
        CommonLanguage::Not(a) => Some((
            CommonLanguage::Bool(!x(a)?.bool()?),
            format!("(~ {})", x(a)?.bool()?).parse().unwrap(),
        )),
        CommonLanguage::Or([a, b]) => Some((
            CommonLanguage::Bool(x(a)?.bool()? || x(b)?.bool()?),
            format!("(| {} {})", x(a)?.bool()?, x(b)?.bool()?)
                .parse()
                .unwrap(),
        )),

        // * Relation
        CommonLanguage::Gt([a, b]) => Some((
            CommonLanguage::Bool(x(a)?.num()? > x(b)?.num()?),
            format!("(> {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        // TODO 下面的其实可有可无
        // CommonLanguage::Lt([a, b]) => Some((
        //     CommonLanguage::Bool(x(a)?.num()? < x(b)?.num()?),
        //     format!("(< {} {})", x(a)?, x(b)?).parse().unwrap(),
        // )),
        // CommonLanguage::Ge([a, b]) => Some((
        //     CommonLanguage::Bool(x(a)?.num()? >= x(b)?.num()?),
        //     format!("(>= {} {})", x(a)?, x(b)?).parse().unwrap(),
        // )),
        // CommonLanguage::Le([a, b]) => Some((
        //     CommonLanguage::Bool(x(a)?.num()? <= x(b)?.num()?),
        //     format!("(<= {} {})", x(a)?, x(b)?).parse().unwrap(),
        // )),
        // CommonLanguage::Ne([a, b]) => Some((
        //     CommonLanguage::Bool(x(a)?.num()? != x(b)?.num()?),
        //     format!("(!= {} {})", x(a)?, x(b)?).parse().unwrap(),
        // )),
        _ => None,
    }
}

impl Analysis<CommonLanguage> for LambdaAnalysis {
    type Data = Data;
    fn merge(&mut self, to: &mut Data, from: Data) -> DidMerge {
        let before_len = to.free.len();
        // to.free.extend(from.free);
        to.free.retain(|i| from.free.contains(i));
        // compare lengths to see if I changed to or from
        DidMerge(
            before_len != to.free.len(),
            to.free.len() != from.free.len(),
        ) | merge_option(&mut to.constant, from.constant, |a, b| {
            assert_eq!(a.0, b.0, "Merged non-equal constants");
            DidMerge(false, false)
        })
    }

    fn make(egraph: &EGraph, enode: &CommonLanguage) -> Data {
        let f = |i: &Id| egraph[*i].data.free.iter().cloned();
        let mut free = HashSet::default();
        match enode {
            CommonLanguage::Var(v) => {
                free.insert(*v);
            }
            CommonLanguage::Let([v, a, b]) => {
                free.extend(f(b));
                free.remove(v);
                free.extend(f(a));
            }
            CommonLanguage::Lambda([v, a]) | CommonLanguage::Fix([v, a]) => {
                free.extend(f(a));
                free.remove(v);
            }
            _ => enode.for_each(|c| free.extend(&egraph[c].data.free)),
        }
        let constant = eval(egraph, enode);
        Data { constant, free }
    }

    fn modify(egraph: &mut EGraph, id: Id) {
        if let Some(c) = egraph[id].data.constant.clone() {
            if egraph.are_explanations_enabled() {
                egraph.union_instantiations(
                    &c.0.to_string().parse().unwrap(),
                    &c.1,
                    &Default::default(),
                    "analysis".to_string(),
                );
            } else {
                let const_id = egraph.add(c.0);
                egraph.union(id, const_id);
            }
        }
    }
}

fn var(s: &str) -> Var {
    s.parse().unwrap()
}

fn is_not_same_var(v1: Var, v2: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    move |egraph, _, subst| egraph.find(subst[v1]) != egraph.find(subst[v2])
}

fn is_const(v: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    move |egraph, _, subst| egraph[subst[v]].data.constant.is_some()
}

// TODO 有待测试正确性
fn is_not_zero(v: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    move |egraph, _, subst| {
        if let Some(n) = &egraph[subst[v]].data.constant {
            if let CommonLanguage::Num(i) = &n.0 {
                *i != 0
            } else {
                false
            }
        } else {
            false
        }
    }
}

struct CaptureAvoid {
    fresh: Var,
    v2: Var,
    e: Var,
    if_not_free: Pattern<CommonLanguage>,
    if_free: Pattern<CommonLanguage>,
}

impl Applier<CommonLanguage, LambdaAnalysis> for CaptureAvoid {
    fn apply_one(
        &self,
        egraph: &mut EGraph,
        eclass: Id,
        subst: &Subst,
        searcher_ast: Option<&PatternAst<CommonLanguage>>,
        rule_name: Symbol,
    ) -> Vec<Id> {
        let e = subst[self.e];
        let v2 = subst[self.v2];
        let v2_free_in_e = egraph[e].data.free.contains(&v2);
        if v2_free_in_e {
            let mut subst = subst.clone();
            let sym = CommonLanguage::Symbol(format!("_{}", eclass).into());
            subst.insert(self.fresh, egraph.add(sym));
            self.if_free
                .apply_one(egraph, eclass, &subst, searcher_ast, rule_name)
        } else {
            self.if_not_free
                .apply_one(egraph, eclass, subst, searcher_ast, rule_name)
        }
    }
}

/// 对表达式进行重写。
/// 重写实际上不是可逆的，即 ==>
#[rustfmt::skip]
fn make_rules() -> Vec<Rewrite<CommonLanguage, LambdaAnalysis>> {
    vec![
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
        rw!("let-const";
            "(let ?v ?e ?c)" => "?c" if is_const(var("?c"))),
        rw!("let-if";
            "(let ?v ?e (if ?cond ?then ?else))" =>
            "(if (let ?v ?e ?cond) (let ?v ?e ?then) (let ?v ?e ?else))"
        ),
        rw!("let-var-same"; "(let ?v1 ?e (var ?v1))" => "?e"),
        rw!("let-var-diff"; "(let ?v1 ?e (var ?v2))" => "(var ?v2)"
            if is_not_same_var(var("?v1"), var("?v2"))),
        rw!("let-lam-same"; "(let ?v1 ?e (lam ?v1 ?body))" => "(lam ?v1 ?body)"),
        rw!("let-lam-diff";
            "(let ?v1 ?e (lam ?v2 ?body))" =>
            { CaptureAvoid {
                fresh: var("?fresh"), v2: var("?v2"), e: var("?e"),
                if_not_free: "(lam ?v2 (let ?v1 ?e ?body))".parse().unwrap(),
                if_free: "(lam ?fresh (let ?v1 ?e (let ?v2 (var ?fresh) ?body)))".parse().unwrap(),
            }}
            if is_not_same_var(var("?v1"), var("?v2"))),

        // * math
        rw!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rw!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
        rw!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        rw!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
        rw!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
        rw!("div-canon"; "(/ ?a ?b)" => "(* ?a (pow ?b -1))" if is_not_zero(var("?b"))),
        rw!("zero-add"; "(+ ?a 0)" => "?a"),
        rw!("zero-mul"; "(* ?a 0)" => "0"),
        rw!("one-mul";  "(* ?a 1)" => "?a"),
        rw!("add-zero"; "?a" => "(+ ?a 0)"),
        rw!("mul-one";  "?a" => "(* ?a 1)"),
        rw!("cancel-sub"; "(- ?a ?a)" => "0"),
        rw!("cancel-div"; "(/ ?a ?a)" => "1" if is_not_zero(var("?a"))),
        rw!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
        rw!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
        rw!("pow-mul"; "(* (pow ?a ?b) (pow ?a ?c))" => "(pow ?a (+ ?b ?c))"),
        rw!("pow0"; "(pow ?x 0)" => "1"
            if is_not_zero(var("?x"))),
        rw!("pow1"; "(pow ?x 1)" => "?x"),
        rw!("pow2"; "(pow ?x 2)" => "(* ?x ?x)"),
        rw!("pow-recip"; "(pow ?x -1)" => "(/ 1 ?x)"
            if is_not_zero(var("?x"))),
        rw!("recip-mul-div"; "(* ?x (/ 1 ?x))" => "1" if is_not_zero(var("?x"))),
        // rw!("neg-to-sub"; "(- ?a)" => "(0 - ?a)"),      // 取反运算符 (新增)

        // * Logic 这里有些公理应该是多余的
        rw!("double-neg-flip"; "(~ (~ ?a))" => "?a"),
        rw!("assoc-or"; "(| ?a (| ?b ?c))" => "(| (| ?a ?b) ?c)"),
        rw!("dist-and-or"; "(& ?a (| ?b ?c))" => "(| (& ?a ?b) (& ?a ?c))"),
        rw!("dist-or-and"; "(| ?a (& ?b ?c))" => "(& (| ?a ?b) (| ?a ?c))"),
        rw!("comm-or"; "(| ?a ?b)" => "(| ?b ?a)"),
        rw!("comm-and"; "(& ?a ?b)" => "(& ?b ?a)"),
        rw!("lem"; "(| ?a (~ ?a))" => "true"),
        rw!("or-true"; "(| ?a true)" => "true"),
        rw!("and-true"; "(& ?a true)" => "?a"),
        // 官方示例少了关于 false 的规则
        // 可能是因为在常数折叠中自动 make 相关规则
        // 已经 加入 prop 的常数折叠
        // rw!("not-true"; "(~ true)" => "false"),
        // rw!("not-false"; "(~ false)" => "true"),
        rw!("or-false"; "(| ?a false)" => "?a"),
        rw!("and-false"; "(& ?a false)" => "false"),
        // * 接下来是额外的自定义的规则 TODO 需仔细研究 加以精简
        // * Relation
        // TODO 加入 Relation 的常数折叠
        // 对称关系
        // rw!("eq-comm"; "(= ?a ?b)" => "(= ?b ?a)"), 前面已经有了
        rw!("gt-comm"; "(& (> ?a ?b) (> ?b ?a))" => "false"),
        // 自反关系
        rw!("eq-true"; "(= ?a ?a)" => "true"),
        rw!("gt-reflexive"; "(> ?a ?a)" => "false"),
        // 传递关系
        // rw!("gt-transitive"; "(& (> ?a ?b) (> ?b ?c))" => "(> ?a ?c)"), 错误的规则！
        // rw!("eq-transitive"; "(& (= ?a ?b) (= ?b ?c))" => "(= ?a ?c)"), 错误的规则！
        // 转换 <, >=, <=, !=, =
        rw!("gt-flip"; "(< ?b ?a)" => "(> ?a ?b)"), // < => >
        rw!("ge-expand"; "(>= ?a ?b)" => "(~ (> ?b ?a))"), // >= => not >+flip
        rw!("le-flip"; "(<= ?a ?b)" => "(>= ?b ?a)"), // <= => >=
        rw!("ne-expand"; "(!= ?a ?b)" => "(~ (= ?a ?b))"),
        // = => <= and >= 首尾相连，实测会让 simplify_test7 快很多
        rw!("eq-expand"; "(= ?a ?b)" => "(& (<= ?a ?b) (>= ?a ?b))"), 

        // * List
        // 柯里化 currying
        rw!("laml-currying-递归终止点"; "(laml (cons ?v nil) ?body)" => "(lam ?v ?body)"),
        rw!("laml-currying-递归"; "(laml (cons ?v ?list) ?body)" => "(lam ?v (laml ?list ?body))"),
        rw!("appl-currying-递归终止点"; "(appl ?f (cons ?v nil))" => "(app ?f ?v)"),
        rw!("appl-currying-递归"; "(appl ?f (cons ?v ?list))" => "(appl (app ?f ?v) ?list)"),

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

#[test]
fn simplify_explain_test() {
    let expr = "(& (| (<= 233 666) (= 2 3)) true)".parse().unwrap();
    // let expr = match start.parse() {
    //     Ok(expr) => expr,
    //     Err(error) => return Err(format!("Failed to parse expression: {}", error)),
    // };

    // 使用 Runner 简化表达式，该运行器创建带有
    // 给定的表达式的 e-graph ，并在其上运行给定的规则
    let mut runner = Runner::default()
        .with_explanations_enabled()
        .with_expr(&expr)
        .run(&make_rules());

    // Runner 知道用 with_expr 给出的表达式在哪个 e-class 中
    let root = runner.roots[0];

    // 使用提取器 extractor 选择 根 eclass 的最佳元素
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (_best_cost, end) = extractor.find_best(root);

    println!("best ================= \n{}", end.to_string());

    // TODO 获取解释性输出的时间很长，需要优化

    println!(
        "get_flat_string ================= \n{}",
        runner.explain_equivalence(&expr, &end).get_flat_string()
    );

    println!(
        "get_string ================= \n{}",
        runner.explain_equivalence(&expr, &end).get_string()
    );

    println!(
        "get_string_with_let ================= \n{}",
        runner
            .explain_equivalence(&expr, &end)
            .get_string_with_let()
    )
}

// * 以下是测试代码

test_fn! { simplify_test1, make_rules(), "(+ 0 (* 1 foo))" => "foo" }
test_fn! { simplify_test2, make_rules(), "(+ 1 1)" => "2" }
test_fn! { simplify_test3, make_rules(), "(+ 1 (- a (* (- 2 1) a)))" => "1" }
test_fn! { simplify_test4, make_rules(), "(lam x (+ 4 (app (lam y (var y)) 4)))" => "(lam x 8)" }
test_fn! { simplify_test5, make_rules(), "(| (& false true) (& true false))" => "false" }
#[cfg(feature = "float")]
test_fn! { simplify_test5, "(+ 0.1 0.2)",make_rules(), "0.3" }
#[cfg(not(feature = "float"))]
test_fn! { simplify_test6, make_rules(), "(+ 0.1 0.2)" => "(+ 0.1 0.2)" }
test_fn! { simplify_test7, make_rules(), "(& (| (> 3 2) (= 1 2)) true)" => "true" }
test_fn! { simplify_test8, make_rules(), "(& (| true (= 3 2)) true)" => "true" }

test_fn! { simplify_test9, make_rules(), "(& (| (<= 233 666) (= 2 3)) true)" => "true" }

// TODO
#[test]
fn debug_test1() {
    println!(
        "{:?}",
        simplify_test("(let x 2 (| (= (var x) 0) (= (var x) 1))) ")
    );
}
#[test]
fn debug_test2() {
    println!(
        "{:?}",
        simplify_test(
            "(if (let x 2 (| (= (var x) 0) (= (var x) 1))) 
    2 
    0)"
        )
    );
}

// math

test_fn! {math_div_same,      make_rules(), "(/ x x)" => "(/ x x)"}
test_fn! {math_div_one,       make_rules(), "(/ x 1)" => "x"}
test_fn! {math_div_zero,      make_rules(), "(/ x 0)" => "(/ x 0)"}
test_fn! {math_div_zero_zero, make_rules(), "(/ 0 0)" => "(/ 0 0)"}
test_fn! {math_div_zero_one,  make_rules(), "(/ 0 1)" => "0"}

egg::test_fn! {math_simplify_add, make_rules(), "(+ x (+ x (+ x x)))" => "(* 4 x)" }
egg::test_fn! {math_powers, make_rules(), "(* (pow 2 x) (pow 2 y))" => "(pow 2 (+ x y))"}

egg::test_fn! {
    math_simplify_const, make_rules(),
    "(+ 1 (- a (* (- 2 1) a)))" => "1"
}

egg::test_fn! {
    math_simplify_root, make_rules(),
    runner = Runner::default().with_node_limit(75_000),
    r#"
    (/ 1
       (- (/ (+ 1 (sqrt five))
             2)
          (/ (- 1 (sqrt five))
             2)))"#
    =>
    "(/ 1 (sqrt five))"
}

egg::test_fn! {math_simplify_factor, make_rules(), "(* (+ x 3) (+ x 1))" => "(+ (+ (* x x) (* 4 x)) 3)"}

egg::test_fn! {laml_curry1, make_rules(), "(laml (cons y nil) (+ 1 (var y)))" => "(lam y (+ 1 (var y)))" }
egg::test_fn! {laml_curry2, make_rules(), "(laml (cons x (cons y nil)) (+ (var x) (var y)))" 
                                       => "(lam x (lam y (+ (var x) (var y))))" }

#[test]
fn temp() {
    println!("{:?}", f64::MAX * f64::MAX);
}
