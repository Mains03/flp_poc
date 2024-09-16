use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "
length :: [Nat] -> Nat
length xs = fold (\\x. \\y. x+1) 0 xs.

itemOf :: Nat -> [Nat] -> Nat
itemOf n xs =
    exists ys :: [Nat].
        exists z :: Nat.
            exists zs :: [Nat].
                length ys =:= n.
                    ys ++ [z] ++ zs =:= xs.
                        z.

itemOf 4 [1,1,2,3,5,8,13,21].
";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv));
}
