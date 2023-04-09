//! egg-IR => python-AST => python 
//! RecExpr<> 是一个 逆波兰表达式的结构

use egg_language_server::egg_support::EggIR;


// 解析逆波兰表达式
// fn rpn_to_string(rpn: &Vec<EggIR>) -> String {
//     let mut stack = Vec::new();

//     for token in rpn {
//         match token {
//             EggIR::Int(val) => stack.push(val.to_string()),
//             EggIR::Op(op) => {
//                 let right = stack.pop().expect("RPN has invalid format");
//                 let left = stack.pop().expect("RPN has invalid format");

//                 let exp = format!("{} {} {}", left, right, op);
//                 stack.push(exp);
//             }
//         }
//     }

//     if stack.len() != 1 {
//         panic!("RPN has invalid format");
//     }

//     stack.pop().unwrap()
// }

fn sexpr_to_python(sexpr: &EggIR) -> String {
	todo!("sexpr_to_python{sexpr}")
}


fn main() {
	let sexpr: EggIR = "(+ 0 (* 1 foo))".parse().unwrap();
	println!("{}", sexpr_to_python(&sexpr));
	
}