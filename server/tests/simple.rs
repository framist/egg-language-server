use egg_language_server::*;

// TODO

#[test]
fn test_py() {
    log_init();

    // python 额外注意空格与 tab 是不一样的！
    let code: &str = r#"
def add1(x):
    return x + 1
y = 1
add1(y)
"#;
    println!("{:#?}", py_parser(code));
}

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
