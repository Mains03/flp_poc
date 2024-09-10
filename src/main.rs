use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "
sum :: [Nat] -> Nat
sum xs = fold (\\x. \\y. x+y) 0 xs.

length :: [Nat] -> Nat
length xs = fold (\\x. \\y. x+1) 0 xs.

exists xs :: [Nat]. length xs =:= 7. sum xs =:= 5. xs.";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv));
}
