use egg::{rewrite as rw, *};

define_language! {
    enum SimpleMath {
        "+" = Add([Id; 2]),
        "*" = Mul([Id; 2]),
        Num(i32),
        Symbol(Symbol),
    }
}

// in this case, our analysis itself doesn't require any data, so we can just
// use a unit struct and derive Default
#[derive(Default)]
struct ConstantFolding;
impl Analysis<SimpleMath> for ConstantFolding {
    type Data = Option<i32>;

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        let cmp = (*to).cmp(&from);
        match cmp {
            std::cmp::Ordering::Less => {
                *to = from;
                DidMerge(true, false)
            }
            std::cmp::Ordering::Equal => DidMerge(false, false),
            std::cmp::Ordering::Greater => DidMerge(false, true),
        }
    }

    fn make(egraph: &EGraph<SimpleMath, Self>, enode: &SimpleMath) -> Self::Data {
        let x = |i: &Id| egraph[*i].data;
        match enode {
            SimpleMath::Num(n) => Some(*n),
            SimpleMath::Add([a, b]) => Some(x(a)? + x(b)?),
            SimpleMath::Mul([a, b]) => Some(x(a)? * x(b)?),
            _ => None,
        }
    }

    fn modify(egraph: &mut EGraph<SimpleMath, Self>, id: Id) {
        if let Some(i) = egraph[id].data {
            let added = egraph.add(SimpleMath::Num(i));
            egraph.union(id, added);
        }
    }
}

fn main() {
    let rules = &[
        rw!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rw!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        rw!("add-0"; "(+ ?a 0)" => "?a"),
        rw!("mul-0"; "(* ?a 0)" => "0"),
        rw!("mul-1"; "(* ?a 1)" => "?a"),
    ];

    let expr = "(+ 0 (* (+ 4 -3) foo))".parse().unwrap();
    let mut runner = Runner::<SimpleMath, ConstantFolding, ()>::default()
        .with_expr(&expr)
        .run(rules);
    let just_foo = runner.egraph.add_expr(&"foo".parse().unwrap());
    assert_eq!(
        runner.egraph.find(runner.roots[0]),
        runner.egraph.find(just_foo)
    );
}
