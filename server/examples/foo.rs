//! This is the parser and interpreter for the 'Foo' language. See `tutorial.md` in the repository's root to learn
//! about it.
use chumsky::prelude::*;

#[derive(Debug)]
enum Expr<'a> {
    Num(f64),
    Var(&'a str),

    Neg(Box<Expr<'a>>),
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    Sub(Box<Expr<'a>>, Box<Expr<'a>>),
    Mul(Box<Expr<'a>>, Box<Expr<'a>>),
    Div(Box<Expr<'a>>, Box<Expr<'a>>),

    Call(&'a str, Vec<Expr<'a>>),
    Let {
        name: &'a str,
        rhs: Box<Expr<'a>>,
        then: Box<Expr<'a>>,
    },
    Fn {
        name: &'a str,
        args: Vec<&'a str>,
        body: Box<Expr<'a>>,
        then: Box<Expr<'a>>,
    },
}

impl Expr<'_> {
    fn ast_to_sexpr(&self) -> Result<String, String> {
        match self {
            Expr::Num(n) => Ok(n.to_string()),
            Expr::Var(name) => Ok(format!("(var {})", name.to_string())),
            Expr::Neg(expr) => Ok(format!("(- {})", expr.ast_to_sexpr()?)),
            Expr::Add(lhs, rhs) => Ok(format!("(+ {} {})", lhs.ast_to_sexpr()?, rhs.ast_to_sexpr()?)),
            Expr::Sub(lhs, rhs) => Ok(format!("(- {} {})", lhs.ast_to_sexpr()?, rhs.ast_to_sexpr()?)),
            Expr::Mul(lhs, rhs) => Ok(format!("(* {} {})", lhs.ast_to_sexpr()?, rhs.ast_to_sexpr()?)),
            Expr::Div(lhs, rhs) => Ok(format!("(/ {} {})", lhs.ast_to_sexpr()?, rhs.ast_to_sexpr()?)),
            Expr::Call(name, args) => {
                let mut sexpr = format!("(app (var {})", name);
                for arg in args {
                    sexpr.push_str(&format!(" {}", arg.ast_to_sexpr()?));
                }
                sexpr.push(')');
                Ok(sexpr)
            },
            Expr::Let { name, rhs, then } => Ok(format!("(let {} {} {})", name, rhs.ast_to_sexpr()?, then.ast_to_sexpr()?)),
            Expr::Fn { name, args, body, then } => {
                let mut sexpr = format!("(let {} (lam ", name);
                for arg in args {
                    sexpr.push_str(&format!("{} ", arg));  // TODO 函数多参数的处理 -> 柯里化？
                }
                sexpr.push_str(&format!("{}", body.ast_to_sexpr()?));
                sexpr.push(')');
                sexpr.push_str(&format!("{}", then.ast_to_sexpr()?));
                sexpr.push(')');
                Ok(sexpr)
            },
        }
    }
}

fn parser<'a>() -> impl Parser<'a, &'a str, Expr<'a>> {
    let ident = text::ident().padded();

    let expr = recursive(|expr| {
        let int = text::int(10)
            .map(|s: &str| Expr::Num(s.parse().unwrap()))
            .padded();

        let call = ident
            .then(
                expr.clone()
                    .separated_by(just(','))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just('('), just(')')),
            )
            .map(|(f, args)| Expr::Call(f, args));

        let atom = int
            .or(expr.delimited_by(just('('), just(')')))
            .or(call)
            .or(ident.map(Expr::Var));

        let op = |c| just(c).padded();

        let unary = op('-')
            .repeated()
            .foldr(atom, |_op, rhs| Expr::Neg(Box::new(rhs)));

        let product = unary.clone().foldl(
            choice((
                op('*').to(Expr::Mul as fn(_, _) -> _),
                op('/').to(Expr::Div as fn(_, _) -> _),
            ))
            .then(unary)
            .repeated(),
            |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
        );

        let sum = product.clone().foldl(
            choice((
                op('+').to(Expr::Add as fn(_, _) -> _),
                op('-').to(Expr::Sub as fn(_, _) -> _),
            ))
            .then(product)
            .repeated(),
            |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
        );

        sum
    });

    let decl = recursive(|decl| {
        let r#let = text::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl.clone())
            .map(|((name, rhs), then)| Expr::Let {
                name,
                rhs: Box::new(rhs),
                then: Box::new(then),
            });

        let r#fn = text::keyword("fn")
            .ignore_then(ident)
            .then(ident.repeated().collect::<Vec<_>>())
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl)
            .map(|(((name, args), body), then)| Expr::Fn {
                name,
                args,
                body: Box::new(body),
                then: Box::new(then),
            });

        r#let.or(r#fn).or(expr).padded()
    });

    decl
}

fn eval<'a>(
    expr: &'a Expr<'a>,
    vars: &mut Vec<(&'a str, f64)>,
    funcs: &mut Vec<(&'a str, &'a [&'a str], &'a Expr<'a>)>,
) -> Result<f64, String> {
    match expr {
        Expr::Num(x) => Ok(*x),
        Expr::Neg(a) => Ok(-eval(a, vars, funcs)?),
        Expr::Add(a, b) => Ok(eval(a, vars, funcs)? + eval(b, vars, funcs)?),
        Expr::Sub(a, b) => Ok(eval(a, vars, funcs)? - eval(b, vars, funcs)?),
        Expr::Mul(a, b) => Ok(eval(a, vars, funcs)? * eval(b, vars, funcs)?),
        Expr::Div(a, b) => Ok(eval(a, vars, funcs)? / eval(b, vars, funcs)?),
        Expr::Var(name) => {
            if let Some((_, val)) = vars.iter().rev().find(|(var, _)| var == name) {
                Ok(*val)
            } else {
                Err(format!("Cannot find variable `{}` in scope", name))
            }
        }
        Expr::Let { name, rhs, then } => {
            let rhs = eval(rhs, vars, funcs)?;
            vars.push((*name, rhs));
            let output = eval(then, vars, funcs);
            vars.pop();
            output
        }
        Expr::Call(name, args) => {
            if let Some((_, arg_names, body)) =
                funcs.iter().rev().find(|(var, _, _)| var == name).copied()
            {
                if arg_names.len() == args.len() {
                    let mut args = args
                        .iter()
                        .map(|arg| eval(arg, vars, funcs))
                        .zip(arg_names.iter())
                        .map(|(val, name)| Ok((*name, val?)))
                        .collect::<Result<_, String>>()?;
                    vars.append(&mut args);
                    let output = eval(body, vars, funcs);
                    vars.truncate(vars.len() - args.len());
                    output
                } else {
                    Err(format!(
                        "Wrong number of arguments for function `{}`: expected {}, found {}",
                        name,
                        arg_names.len(),
                        args.len(),
                    ))
                }
            } else {
                Err(format!("Cannot find function `{}` in scope", name))
            }
        }
        Expr::Fn {
            name,
            args,
            body,
            then,
        } => {
            funcs.push((name, args, body));
            let output = eval(then, vars, funcs);
            funcs.pop();
            output
        }
    }
}

// const SRC: &str = r"
// let five = 5;
// let eight = 3 + five;
// fn add x y = x + y;
// add(five, eight)
// ";

const SRC: &str = r"
fn add x = x + 1;
add(1)
";

// const SRC: &str = r"
// let x = 0;
// let y = 5 + x;
// x * y
// ";


fn main() {
    // let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let src = match std::env::args().nth(1) {
        Some(file_name) => std::fs::read_to_string(file_name).unwrap(),
        None => SRC.to_owned(),
    };
    println!("{SRC}");

    match parser().parse(&src).into_result() {
        Ok(ast) => {
            println!("ast: {:#?}", ast);
            println!("sexpr: {}", ast.ast_to_sexpr().unwrap());
            match eval(&ast, &mut Vec::new(), &mut Vec::new()) {
                Ok(output) => println!("计算结果为: {}", output),
                Err(eval_err) => println!("Evaluation error: {}", eval_err),
            }
        }
        Err(parse_errs) => parse_errs
            .into_iter()
            .for_each(|e| println!("Parse error: {}", e)),
    };
}