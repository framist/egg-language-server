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
        "-" = Sub([Id; 2]),
        "/" = Div([Id; 2]),
        "pow" = Pow([Id; 2]),
        "ln" = Ln(Id),
        "sqrt" = Sqrt(Id),

        // Constant(Constant),

    }
}

impl CommonLanguage {
    fn num(&self) -> Option<i32> {
        match self {
            CommonLanguage::Num(n) => Some(*n),
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

fn eval(
    egraph: &EGraph,
    enode: &CommonLanguage,
) -> Option<(CommonLanguage, PatternAst<CommonLanguage>)> {
    let x = |i: &Id| egraph[*i].data.constant.as_ref().map(|c| &c.0);
    match enode {
        CommonLanguage::Num(n) => Some((enode.clone(), format!("{}", n).parse().unwrap())),
        CommonLanguage::Bool(b) => Some((enode.clone(), format!("{}", b).parse().unwrap())),
        CommonLanguage::Add([a, b]) => Some((
            CommonLanguage::Num(x(a)?.num()? + x(b)?.num()?),
            format!("(+ {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        CommonLanguage::Eq([a, b]) => Some((
            CommonLanguage::Bool(x(a)? == x(b)?),
            format!("(= {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
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

// 用于对SimpleLanguage语言中的表达式进行重写。
fn make_rules() -> Vec<Rewrite<CommonLanguage, LambdaAnalysis>> {
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
