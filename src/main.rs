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

sum :: [Nat] -> Nat
sum xs = case xs of
    [] -> 0.
    (x:xs) -> x + (sum xs).

exists xs :: [Nat]. sum xs =:= 5. length xs =:= 2. xs.
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
