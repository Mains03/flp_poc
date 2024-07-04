mod parser;
mod type_check;
mod cbpv;

fn main() {
    let src = "
id :: Nat -> Nat
id x = x

let y = 5 in id y"; // TODO: fresh variables in bind

    let ast = parser::parse(src).unwrap();
    println!("{:#?}", cbpv::eval(ast));
}
