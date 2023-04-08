/* NOTICE 
println 等向标准输出流写入会影响 语言服务器和客户端的通信
请使用 debug!
*/
use egg::RecExpr;
use super::lisp::LispLanguage;

pub fn egg_violence(s: &str) -> Result<String, String> {    
    super::lisp::simplify_test(s)
}

pub fn simplify(s: &str) -> Result<Option<RecExpr<LispLanguage>>, String> {
    super::lisp::simplify(s)
}

