use egg::{rewrite as rw, *};
use log::*;
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
        "app" = App([Id; 2]),   // apply 使用 lam 函数 声明之后为一个 lambda, 如果是函数名，则需 var 声明
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

// 该函数定义语言：SimpleLanguage。
// 它包括 Num、加号"+"(Add、两个 Id 标志符参数)、
// "*" 乘号 (Mul、两个 Id 标志符参数) 以及 Symbol 标记。
#[cfg(not(feature = "float"))]
define_language! {
    pub enum CommonLanguage {
        Num(i32),

        // * lambda
        Bool(bool),
        "var" = Var(Id),        // 实参注明
        "=" = Eq([Id; 2]),
        "app" = App([Id; 2]),   // apply 施加函数
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

        // * List
        // 注意，为了防止歧义，目前仅用于解决多参数问题；数据结构构建都应看作未定义的函数
        "cons" = Cons([Id; 2]),
        // "car" = Car(Id),
        // "cdr" = Cdr(Id),
        "nil" = Nil,
        // 多参 的函数 例如 (laml (cons x (cons y nil)) (+ x y))
        "laml" = LambdaL([Id; 2]),
        "appl" = AppL([Id; 2]),

        // * Imp 指令式程序
        "skip" = Skip,
        "seq" = Seq([Id; 2]),           // 序列指令
        "seqlet" = SeqLet([Id; 2]),     // (sqlet x 1)

        "while" = While([Id; 2]),       // (while (x < 10) (x = x + 1))
        "for" = For([Id; 4]),           // (for (x = 0) (x < 10) (x = x + 1) (x = x + 1))

        // can also do a variable number of children in a boxed slice
        // this will only match if the lengths are the same
        // 但是涉及 Multi-matching patterns (ex: `?a...`)  egg 还没有实现 参见 Egg CHANGELOG
        // TODO "list" = List(Box<[Id]>),
 
        Symbol(Symbol),// 语言项的解析是按顺序进行的，所以这个应放在后面
        // 这是最终的回退，它将解析任何运算符 (作为字符串) 和任意数量的孩子。
        // 请注意，如果有 0 个子级，则前一个分支将成功
        Other(Symbol, Vec<Id>),
    }
}

struct CommonLanguageCostFn;
impl CostFunction<CommonLanguage> for CommonLanguageCostFn {
    type Cost = f64;
    fn cost<C>(&mut self, enode: &CommonLanguage, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        let op_cost = match enode {
            CommonLanguage::Cons(..) => 0.01,
            CommonLanguage::Nil => 0.01,
            CommonLanguage::Seq(..) => 0.01,
            _ => 1.0,
        };
        enode.fold(op_cost, |sum, i| sum + costs(i))
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

// 要仔细检查算术溢出的情况 https://course.rs/compiler/pitfalls/arithmetic-overflow.html
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
        // CommonLanguage::Div([a, b]) if x(b)?.num()? != 0 => Some((  // 除数不能为 0
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
        // TODO 变量情况下没法常数折叠 (let a 1 (!= (var a) 2))
        CommonLanguage::Gt([a, b]) => Some((
            CommonLanguage::Bool(x(a)?.num()? > x(b)?.num()?),
            format!("(> {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        // TODO 下面的其实可有可无
        CommonLanguage::Lt([a, b]) => Some((
            CommonLanguage::Bool(x(a)?.num()? < x(b)?.num()?),
            format!("(< {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        CommonLanguage::Ge([a, b]) => Some((
            CommonLanguage::Bool(x(a)?.num()? >= x(b)?.num()?),
            format!("(>= {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        CommonLanguage::Le([a, b]) => Some((
            CommonLanguage::Bool(x(a)?.num()? <= x(b)?.num()?),
            format!("(<= {} {})", x(a)?, x(b)?).parse().unwrap(),
        )),
        CommonLanguage::Ne([a, b]) => Some((
            CommonLanguage::Bool(x(a)? != x(b)?),
            format!("(!= {} {})", x(a)?, x(b)?).parse().unwrap(),
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
/// => 重写是不可逆的。可逆重写可以用 <=> 实现
#[rustfmt::skip]
fn make_rules() -> Vec<Rewrite<CommonLanguage, LambdaAnalysis>> {
    vec![
        // * lambda
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
        rw!("let-lam-same"; "(let ?v1 ?e (lam ?v1 ?body))" => "(lam ?v1 ?body)"), // = identity_assignment sikp
        rw!("let-lam-diff";
            "(let ?v1 ?e (lam ?v2 ?body))" =>
            { CaptureAvoid {
                fresh: var("?fresh"), v2: var("?v2"), e: var("?e"),
                if_not_free: "(lam ?v2 (let ?v1 ?e ?body))".parse().unwrap(),
                if_free: "(lam ?fresh (let ?v1 ?e (let ?v2 (var ?fresh) ?body)))".parse().unwrap(),
            }}
            if is_not_same_var(var("?v1"), var("?v2"))),

        // 柯里化 currying
        rw!("laml-currying-end"; "(laml (cons ?v nil) ?body)" => "(lam ?v ?body)"),
        rw!("laml-currying-fix"; "(laml (cons ?v ?list) ?body)" => "(lam ?v (laml ?list ?body))"),
        rw!("appl-currying-end"; "(appl ?f (cons ?v nil))" => "(app ?f ?v)"),
        rw!("appl-currying-fix"; "(appl ?f (cons ?v ?list))" => "(appl (app ?f ?v) ?list)"),

        // * math
        rw!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rw!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
        rw!("add-assoc"; "(+ (+ ?a ?b) ?c)" => "(+ ?a (+ ?b ?c))"),  // ? 不加上有部分测试过不了，还不知道为什么
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
        // 对称关系
        rw!("eq-comm"; "(= ?a ?b)" => "(= ?b ?a)"), 
        rw!("gt-comm"; "(& (> ?a ?b) (> ?b ?a))" => "false"),
        // 自反关系
        rw!("eq-true"; "(= ?a ?a)" => "true"),
        rw!("gt-reflexive"; "(> ?a ?a)" => "false"),
        // 转换 <, >=, <=, !=, =
        rw!("gt-flip"; "(< ?b ?a)" => "(> ?a ?b)"), // < => >
        rw!("ge-expand"; "(>= ?a ?b)" => "(~ (> ?b ?a))"), // >= => not >+flip
        rw!("le-flip"; "(<= ?a ?b)" => "(>= ?b ?a)"), // <= => >=
        rw!("ne-expand"; "(!= ?a ?b)" => "(~ (= ?a ?b))"),
        // = => <= and >= 首尾相连
        rw!("eq-expand"; "(= ?a ?b)" => "(& (<= ?a ?b) (>= ?a ?b))"), 

        // * List

        // * Imp
        // 序列 seq
        rw!("seq-end"; "(seq ?a nil)" => "?a"),
        rw!("seq-let"; "(seq (seqlet ?v ?e) ?body)" => "(let ?v ?e ?body)"),
        rw!("seq-assoc"; "(seq (seq ?a ?b) ?c)" => "(seq ?a (seq ?b ?c))"),
        rw!("seq-skip-left"; "(seq skip ?a)" => "?a"),
        rw!("seq-skip-right"; "(seq ?a skip)" => "?a"),
        // if
        rw!("if-true";  "(if  true ?then ?else)" => "?then"),
        rw!("if-false"; "(if false ?then ?else)" => "?else"),
        rw!("if-elim"; "(if (= (var ?x) ?e) ?then ?else)" => "?else"
            if ConditionEqual::parse("(let ?x ?e ?then)", "(let ?x ?e ?else)")),
        // while
        rw!("while-true"; "(while true ?body)" => "(while true skip)"),
        rw!("while-false"; "(while false ?body)" => "skip"),
        rw!("while-expand"; "(while ?cond ?body)" => "(seq ?body (while ?cond ?body))"),
        // for
        rw!("for-expand"; "(for ?init ?cond ?update ?body)" => "(seq ?init (while ?cond (seq ?body ?update)))"),
        // TODO ...

    ]
}

use std::time::{Duration, Instant};
/// 解析一个表达式，使用 egg 对其进行简化
/// - 如果解析失败，则返回错误
/// - 如果没有产生简化，则返回 None
/// - 如果产生了简化，则返回 Some(简化后的 RecExpr 逆波兰表达式)
pub fn simplify(s: &str) -> Result<Option<RecExpr<CommonLanguage>>, String> {
    if s.is_empty() {
        return Ok(None);
    }
    if s.contains("unhandled-node-kind") {
        return Ok(None); // TODO 暂时不解决这个问题
    }

    let expr = match s.parse() {
        Ok(expr) => expr,
        Err(error) => return Err(format!("Egg failed to parse: {}", error)),
    };

    // 一个计时器
    let start = Instant::now();
    let runner = Runner::default()
        .with_time_limit(Duration::new(5, 500_000_000)) // 这个超时时间应该要能设置为自定义的，也可以参考测试结果的最长时间 ps. release 模式下基本上不用限制时间
        .with_expr(&expr)
        .run(&make_rules());
    debug!("runner spend: {:?}", start.elapsed());

    // Runner 知道用 with_expr 给出的表达式在哪个 e-class 中
    let root = runner.roots[0];

    // 使用提取器 extractor 选择 根 eclass 的最佳元素
    let extractor = Extractor::new(&runner.egraph, CommonLanguageCostFn);
    let (best_cost, best) = extractor.find_best(root);
    debug!("best sexpr:\n{}", best.to_string());
    // cost  的变化
    debug!(
        "cost: {} -> {}",
        CommonLanguageCostFn.cost_rec(&expr),
        best_cost
    );
    if best_cost <= CommonLanguageCostFn.cost_rec(&expr) - 1.0 {
        Ok(Some(best))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
pub fn simplify_test(s: &str) -> Result<String, String> {
    match simplify(s)? {
        Some(expr) => Ok(format!("{}", expr)),
        None => Ok(s.to_string()),
    }
}

#[test]
fn simplify_explain_test() {
    let expr = "(+ 1 1)".parse().unwrap();
    // let expr = match start.parse() {
    //     Ok(expr) => expr,
    //     Err(error) => return Err(format!("Failed to parse expression: {}", error)),
    // };

    // 使用 Runner 简化表达式，该运行器创建带有
    // 给定的表达式的 e-graph，并在其上运行给定的规则
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
    // ~60s in debug mode
    // ~12s in release mode

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

// * 以下是测试代码 *

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
// test_fn! { simplify_test10, make_rules(), " (let a 1 (!= (var a) 2))" => "true" }

// TODO
#[test]
fn debug_test1() {
    println!(
        "{:?}",
        simplify_test("(let x 2 (| (= (var x) 0) (= (var x) 1))) ")
    );
}

// TODO
#[test]
fn debug_test2() {
    println!(
        "{:?}",
        simplify_test(
            "(let x 2 (* 9 (var x)))"
        )
    );
}

#[test]
fn time_test1() {
    println!("{:?}", simplify_test("(+ 1 1)"));
}


// * math test *

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

test_fn! {math_simplify_factor, make_rules(), "(* (+ x 3) (+ x 1))" => "(+ (+ (* x x) (* 4 x)) 3)"}

// * lambda test *

test_fn! {
    lambda_under, make_rules(),
    "(lam x (+ 4
               (app (lam y (var y))
                    4)))"
    =>
    // "(lam x (+ 4 (let y 4 (var y))))",
    // "(lam x (+ 4 4))",
    "(lam x 8))",
}

test_fn! {
    lambda_if_elim, make_rules(),
    "(if (= (var a) (var b))
         (+ (var a) (var a))
         (+ (var a) (var b)))"
    =>
    "(+ (var a) (var b))"
}

test_fn! {
    lambda_let_simple, make_rules(),
    "(let x 0
     (let y 1
     (+ (var x) (var y))))"
    =>
    // "(let ?a 0
    //  (+ (var ?a) 1))",
    // "(+ 0 1)",
    "1",
}

test_fn! {
    #[should_panic(expected = "Could not prove goal 0")]
    lambda_capture, make_rules(),
    "(let x 1 (lam x (var x)))" => "(lam x 1)"
}

test_fn! {
    #[should_panic(expected = "Could not prove goal 0")]
    lambda_capture_free, make_rules(),
    "(let y (+ (var x) (var x)) (lam x (var y)))" => "(lam x (+ (var x) (var x)))"
}

test_fn! {
    #[should_panic(expected = "Could not prove goal 0")]
    lambda_closure_not_seven, make_rules(),
    "(let five 5
     (let add-five (lam x (+ (var x) (var five)))
     (let five 6
     (app (var add-five) 1))))"
    =>
    "7"
}

test_fn! {
    lambda_compose, make_rules(),
    "(let compose (lam f (lam g (lam x (app (var f)
                                       (app (var g) (var x))))))
     (let add1 (lam y (+ (var y) 1))
     (app (app (var compose) (var add1)) (var add1))))"
    =>
    "(lam ?x (+ 1
                (app (lam ?y (+ 1 (var ?y)))
                     (var ?x))))",
    "(lam ?x (+ (var ?x) 2))"
}

test_fn! {
    lambda_if_simple, make_rules(),
    "(if (= 1 1) 7 9)" => "7"
}

// this is a bit slow, need 12s+ ; but in release mode, it's ~0.1s
test_fn! {
    lambda_compose_many, make_rules(),
    "(let compose (lam f (lam g (lam x (app (var f)
                                       (app (var g) (var x))))))
     (let add1 (lam y (+ (var y) 1))
     (app (app (var compose) (var add1))
          (app (app (var compose) (var add1))
               (app (app (var compose) (var add1))
                    (app (app (var compose) (var add1))
                         (app (app (var compose) (var add1))
                              (app (app (var compose) (var add1))
                                   (var add1)))))))))"
    =>
    "(lam ?x (+ (var ?x) 7))"
}

test_fn! {
    lambda_if, make_rules(),
    "(let zeroone (lam x
        (if (= (var x) 0)
            0
            1))
        (+ (app (var zeroone) 0)
        (app (var zeroone) 10)))"
    =>
    // "(+ (if false 0 1) (if true 0 1))",
    // "(+ 1 0)",
    "1",
}

test_fn! {
    #[cfg(not(debug_assertions))]
    #[cfg_attr(feature = "test-explanations", ignore)]
    lambda_fib, make_rules(),
    runner = Runner::default()
        .with_iter_limit(60)
        .with_node_limit(500_000),
    "(let fib (fix fib (lam n
        (if (= (var n) 0)
            0
        (if (= (var n) 1)
            1
        (+ (app (var fib)
                (+ (var n) -1))
            (app (var fib)
                (+ (var n) -2)))))))
        (app (var fib) 4))"
    => "3"
}

// * imp test *

test_fn! {laml_curry1, make_rules(), "(laml (cons y nil) (+ 1 (var y)))" => "(lam y (+ 1 (var y)))" }
test_fn! {laml_curry2, make_rules(), "(laml (cons x (cons y nil)) (+ (var x) (var y)))"
=> "(lam x (lam y (+ (var x) (var y))))" }

test_fn! {seqlet1, make_rules(), "(seq (seqlet a 1) (seq (var a) nil))"
=> "1" }

test_fn! {skip1, make_rules(), "(seq (seq skip (seq (var a) skip)) skip)"
=> "(var a)" }


test_fn! {while_true, make_rules(), "(while true (var a))"
=> "(while true skip)" }

test_fn! {while_false, make_rules(), "(while false (var a))"
=> "skip" }

// test_fn! {for_to_while, make_rules(), "(for (seqlet i 0) (<= (var i) 10) (+ (var i) 1) (var i))"
// => "" }

// * Prop test *
test_fn! { not_false, make_rules(), "(~ false)" => "true" }
test_fn! { not_true, make_rules(), "(~ true)" => "false" }
test_fn! { not_not, make_rules(), "(~ (~ b))" => "b" }
test_fn! { not_not_many, make_rules(), "(~ (~ (~ (~ (~ (~ (~ (~ b))))))))" => "b" }

// TODO
// test_fn! { and_not, make_rules(), "(& (~ a) (a)))" => "false" }


// * Relation test *
test_fn! { rel_eq_true, make_rules(), "(= a a)" => "true" }
test_fn! { lq_true, make_rules(), "(<= a a)" => "true" }
test_fn! { ge_true, make_rules(), "(>= a a)" => "true" }
test_fn! { gt_false, make_rules(), "(> a a)" => "false" }
test_fn! { lt_false, make_rules(), "(< a a)" => "false" }

test_fn! {
    #[should_panic(expected = "Could not prove goal 0")]
    rel_eq_false, make_rules(),
    "(= a b)" => "false"
}


#[test]
fn temp() {
    println!("{:?}", f64::MAX * f64::MAX);
}
