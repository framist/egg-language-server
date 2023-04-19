use egg::{rewrite as rw, *};

define_language! {
enum Math {
        Num(i32),
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        Symbol(Symbol),
    }
}

type EGraph = egg::EGraph<Math, MinSize>;

// Our metadata in this case will be size of the smallest
// represented expression in the eclass.
#[derive(Default)]
struct MinSize;
impl Analysis<Math> for MinSize {
    type Data = usize;
    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        merge_min(to, from)
    }
    fn make(egraph: &EGraph, enode: &Math) -> Self::Data {
        let get_size = |i: Id| egraph[i].data;
        AstSize.cost(enode, get_size)
    }
}

fn main() {
    let rules = &[
        rw!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rw!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        rw!("add-0"; "(+ ?a 0)" => "?a"),
        rw!("mul-0"; "(* ?a 0)" => "0"),
        rw!("mul-1"; "(* ?a 1)" => "?a"),
        // the rewrite macro parses the rhs as a single token tree, so
        // we wrap it in braces (parens work too).
        rw!("funky"; "(+ ?a (* ?b ?c))" => { Funky {
            a: "?a".parse().unwrap(),
            b: "?b".parse().unwrap(),
            c: "?c".parse().unwrap(),
            ast: "(+ (+ ?a 0) (* (+ ?b 0) (+ ?c 0)))".parse().unwrap(),
        }}),
    ];

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Funky {
        a: Var,
        b: Var,
        c: Var,
        ast: PatternAst<Math>,
    }

    impl Applier<Math, MinSize> for Funky {
        fn apply_one(
            &self,
            egraph: &mut EGraph,
            matched_id: Id,
            subst: &Subst,
            _searcher_pattern: Option<&PatternAst<Math>>,
            _rule_name: Symbol,
        ) -> Vec<Id> {
            let a: Id = subst[self.a];
            // In a custom Applier, you can inspect the analysis data,
            // which is powerful combination!
            let size_of_a = egraph[a].data;
            if size_of_a > 50 {
                println!("Too big! Not doing anything");
                vec![]
            } else {
                // we're going to manually add:
                // (+ (+ ?a 0) (* (+ ?b 0) (+ ?c 0)))
                // to be unified with the original:
                // (+    ?a    (*    ?b       ?c   ))
                let b: Id = subst[self.b];
                let c: Id = subst[self.c];
                let zero = egraph.add(Math::Num(0));
                let a0 = egraph.add(Math::Add([a, zero]));
                let b0 = egraph.add(Math::Add([b, zero]));
                let c0 = egraph.add(Math::Add([c, zero]));
                let b0c0 = egraph.add(Math::Mul([b0, c0]));
                let a0b0c0 = egraph.add(Math::Add([a0, b0c0]));
                // Don't forget to union the new node with the matched node!
                if egraph.union(matched_id, a0b0c0) {
                    vec![a0b0c0]
                } else {
                    vec![]
                }
            }
        }
    }

    let start = "(+ x (* y z))".parse().unwrap();

    let runner = Runner::default().with_expr(&start).run(rules);

    // Runner 知道用 with_expr 给出的表达式在哪个 e-class 中
    let root = runner.roots[0];

    // 使用提取器 extractor 选择 根 eclass 的最佳元素
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (best_cost, best) = extractor.find_best(root);
    println!("best_cost: {}", best_cost);
    println!("best pretty: \n{}", best.pretty(10));
}
