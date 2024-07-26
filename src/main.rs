use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;

fn main() {
    let src = "exists n :: Nat. n + n =:= 2. n.";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", cbpv.get("main").unwrap());
}
