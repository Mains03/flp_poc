use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "
concat :: [Nat] -> [Nat]
concat xs ys = case xs of
    [] -> ys
    (x:xs) -> x : concat xs ys

concat [1,2,3] [4,5].
";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv));
}
