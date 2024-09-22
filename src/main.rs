use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "
mult :: Nat -> Nat -> Nat
mult n m = case n of
	Zero -> 0.
	(Succ n) -> m + (mult n m).

square :: Nat -> Nat
square n = mult n n.

head :: [Nat] -> Nat
head xs = case xs of
	[] -> Zero.
	(x:xs) -> x.

squareRoot :: [Nat] -> [Nat]
squareRoot xs = case xs of
	[] -> [].
	(x:xs) -> (exists y :: Nat. square y =:= x. y : (squareRoot xs)).

squareRoot [36, 9, 144].
";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv));
}
