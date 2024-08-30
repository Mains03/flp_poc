use cbpv::translate::translate;

mod parser;
mod type_check;
mod cbpv;
mod eval;

fn main() {
    let src = "exists n :: Nat. n =:= 1. exists m :: Nat. n =:= m. m.";

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

    #[test]
    fn test2() {
        let src = "let n = 1 in (let n = 2 in n) + n.";

        let ast = parser::parse(src).unwrap();
        let cbpv = translate(ast);
        let term = eval::eval(cbpv);

        assert_eq!(
            term,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Succ(Box::new(Term::Succ(Box::new(Term::Zero))))))))
        );
    }
}