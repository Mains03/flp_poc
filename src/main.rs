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

exists xs :: [Nat]. length xs =:= 3. xs.
";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv));
}
