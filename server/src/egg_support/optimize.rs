/* NOTICE 
println 等向标准输出流写入会影响 语言服务器和客户端的通信
请使用 debug!
*/

pub fn egg_violence(s: &str) -> Result<String, String> {
    // let mut ans = String::from("");
    // if let Ok(s1) = super::simple::simplify(s){
    //     ans = ans + &s1;
    // };
    // if let Ok(s1) = super::math::simplify(s){
    //     ans = ans + &s1;
    // };
    // if let Ok(s1) = super::lambda::simplify(s){
    //     ans = ans + &s1;
    // };
    // if let Ok(s1) = super::prop::simplify(s){
    //     ans = ans + &s1;
    // };
    
    super::lisp::simplify(s)
}
