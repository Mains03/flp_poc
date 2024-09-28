use std::env;

use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "
length :: [Nat] -> Nat
length xs = case xs of
    [] -> 0.
    (x:xs) -> 1 + (length xs).

const :: a -> b -> a
const x y = x.

f :: [Nat] -> Nat -> Nat
f xs y = const (const (length xs) 1) y.

f [1,2,3] 2.
";

    let args: Vec<String> = env::args().collect();

    let solution_count = match args.get(1) {
        Some(n) => n.parse().unwrap(),
        None => 1
    };

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv, solution_count));
}
