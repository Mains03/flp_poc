use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "
length :: [Nat] -> Nat
length xs = fold (\\x. \\y. x+1) 0 xs.

f (xs, i) n = n =:= i. (xs, i+1).

fst (xs, n) = xs.

exists xs :: [Nat]. length xs =:= 100. fst (fold f (xs, 0) xs).
";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv));
}
