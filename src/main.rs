use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "
head :: [Nat] -> Nat
head xs = case xs of
    [] -> 0.
    (x:xs) -> x.

itemOf :: Nat -> [Nat] -> Nat
itemOf n xs = case n of
    Zero -> head xs.
    (Succ n) -> itemOf n xs.

itemOf 1 [1,2,3].
";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv));
}
