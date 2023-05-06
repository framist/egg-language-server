use egg_language_server::*;
// use std::time::*;
use std::env;

fn main() {
    // log_init();
    // 获取命令行参数 参数内容是代码的文件路径
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <code_file>", args[0]);
        return;
    }
    let code_file = &args[1];
    let code = std::fs::read_to_string(code_file).unwrap();
    py_main(&code);
}

fn py_main(code: &str) {
    // 一个计时器
    // let start = Instant::now();
    println!("{:?}", py_parser(code));
    // println!("Time: {:?}", start.elapsed());
}

#[allow(dead_code)]
fn log_init() {
    // 设定日志输出为标准输出
    std::env::set_var("RUST_LOG", "egg_language_server=debug,egg=off");
    // 在客户端已设置环境变量
    use std::io::Write;
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} - {}] {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}