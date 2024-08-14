use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "
add :: Nat -> Nat -> Nat
add n m = n + m.
    
let n = 1 in (let m = 2 in add n m).";

    let ast = parser::parse(src).unwrap();
    let cbpv = translate(ast);
    println!("{:#?}", eval::eval(cbpv));
}

#[cfg(test)]
mod tests {
    use cbpv::Term;

    use super::*;

    #[test]
    fn test1() {
        let src = "1+1.";

        let ast = parser::parse(src).unwrap();
        let cbpv = translate(ast);
        let term = eval::eval(cbpv);

        assert_eq!(
            term,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Succ(Box::new(Term::Zero))))))
        );
    }

    
}