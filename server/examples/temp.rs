use egg::*;
use std::borrow::Cow;
use std::sync::Arc;

define_language! {
    enum SimpleLanguage {
        Num(i32),
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
    }
}

type EGraph = egg::EGraph<SimpleLanguage, ()>;

fn main() {
    let mut rules: Vec<Rewrite<SimpleLanguage, ()>> = vec![
        rewrite!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
        rewrite!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
        rewrite!("mul-0"; "(* ?a 0)" => "0"),
        rewrite!("silly"; "(* ?a 1)" => { MySillyApplier("foo") }),
        rewrite!("something_conditional";
             "(/ ?a ?b)" => "(* ?a (/ 1 ?b))"
             if is_not_zero("?b")),
    ];

    // rewrite! supports bidirectional rules too
    // it returns a Vec of length 2, so you need to concat
    rules.extend(
        vec![
            rewrite!("add-0"; "(+ ?a 0)" <=> "?a"),
            rewrite!("mul-1"; "(* ?a 1)" <=> "?a"),
        ]
        .concat(),
    );
}

#[test]
fn concat() {
    assert_eq!(["hello", "world"].concat(), "helloworld");
    assert_eq!([[1, 2], [3, 4]].concat(), [1, 2, 3, 4]);
}

#[derive(Debug)]
struct MySillyApplier(&'static str);
impl Applier<SimpleLanguage, ()> for MySillyApplier {
    fn apply_one(
        &self,
        _: &mut EGraph,
        _: Id,
        _: &Subst,
        _: Option<&PatternAst<SimpleLanguage>>,
        _: Symbol,
    ) -> Vec<Id> {
        panic!()
    }
}

// This returns a function that implements Condition
fn is_not_zero(var: &'static str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    let var = var.parse().unwrap();
    let zero = SimpleLanguage::Num(0);
    move |egraph, _, subst| !egraph[subst[var]].nodes.contains(&zero)
}
