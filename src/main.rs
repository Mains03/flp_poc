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

fst (xs, i) = xs.

exists xs :: [Nat]. let ys = fst (fold f (xs, 0) xs) in length ys =:= 10. ys.
";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv));
}
