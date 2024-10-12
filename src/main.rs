use std::env;

use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "
cat :: [Nat] -> [Nat] -> [Nat]
cat xs ys = case xs of
    [] -> ys.
    (x:xs) -> x : (cat xs ys).

last :: [Nat] -> Nat
last xs = exists ys :: [Nat]. exists y :: Nat.
    cat ys [y] =:= xs. y.

last [1,2,3,4,5,3].
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
