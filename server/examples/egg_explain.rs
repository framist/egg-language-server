use egg::{rewrite as rw, *};

fn main() {
    let rules: &[Rewrite<SymbolLang, ()>] = &[
        rw!("div-one 除以 1"; "?x" => "(/ ?x 1)"),
        rw!("unsafe-invert-division 不安全的消除分母"; "(/ ?a ?b)" => "(/ 1 (/ ?b ?a))"),
        rw!("simplify-frac 简化分式"; "(/ ?a (/ ?b ?c))" => "(/ (* ?a ?c) (* (/ ?b ?c) ?c))"),
        rw!("cancel-denominator 约简分母"; "(* (/ ?a ?b) ?b)" => "?a"),
        rw!("times-zero 乘零"; "(* ?a 0)" => "0"),
    ];

    let start = "(/ (* (/ 2 3) (/ 3 2)) 1)".parse().unwrap();
    let end = "1".parse().unwrap();
    let mut runner = Runner::default()
        .with_explanations_enabled()
        .with_expr(&start)
        .run(rules);

    println!(
        "get_flat_string ================= \n{}",
        runner.explain_equivalence(&start, &end).get_flat_string()
    );

    println!(
        "get_string ================= \n{}",
        runner.explain_equivalence(&start, &end).get_string()
    );

    println!(
        "get_string_with_let ================= \n{}",
        runner.explain_equivalence(&start, &end).get_string_with_let()
    )
}

