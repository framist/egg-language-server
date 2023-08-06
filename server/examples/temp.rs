use egg::*;

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
    use serde_json::Value;

    let json_str = r#"
        {
            "name": "John",
            "age": 30,
            "city": "New York"
        }
    "#;
    
    let value: Value = serde_json::from_str(json_str).unwrap();
    
    println!("Name: {}", value["name"]);
    println!("Age: {}", value["age"]);
    println!("City: {}", value["city"]);
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
