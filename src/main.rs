mod parser;
mod type_check;
mod cbpv;

fn main() {
    let src = "";

    let ast = parser::parse(src).unwrap();
    println!("{:#?}", cbpv::eval(ast));
}
