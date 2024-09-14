use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "
length :: [Nat] -> Nat
length xs = fold (\\x. \\y. x+1) 0 xs.

half xs = exists ys :: [Nat].
            exists zs :: [Nat].
                ys ++ zs =:= xs.
                    let z = length ys <> (length ys) + 1 in length zs =:= z. (ys, zs).

half [1,2,3].
";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv));
}
