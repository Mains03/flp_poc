use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "exists n :: Nat. n + n =:= 2. n.";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", cbpv.get("main").unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let src = "1+1.";

        let ast = parser::parse(src).unwrap();
        let cbpv = translate(ast);
        println!("{:#?}", eval::eval(cbpv));
    }
}