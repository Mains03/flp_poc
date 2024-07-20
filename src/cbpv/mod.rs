use std::collections::{HashMap, HashSet};

use term::Term;
use translate::Translate;

use crate::parser::syntax::decl::Decl;

pub mod term;
pub mod translate;
mod exists;
mod equate;

pub fn eval<'a>(ast: Vec<Decl<'a>>) -> Term<'a> {
    ast.translate(&mut HashSet::new(), &mut HashMap::new()).eval()
}

#[cfg(test)]
mod tests {
    use crate::parser;

    use super::*;

    #[test]
    fn test1() {
        let src = "id :: a -> a
id x = x.

id 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Succ(Box::new(Term::Zero)))
            )
        );
    }

    #[test]
    fn test2() {
        let src = "const :: a -> b -> a
const x y = x.

const 1 2.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Succ(Box::new(Term::Zero)))
            )
        );
    }

    #[test]
    fn test3() {
        let src = "const :: a -> b -> a
const x y = x.

const 2 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Succ(Box::new(Term::Succ(Box::new(Term::Zero)))))
            )
        );
    }

    #[test]
    fn test4() {
        let src: &str = "id :: a -> a
id x = x.

f :: (a -> a) -> a -> a
f g x = g x.

f (f id) 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Succ(Box::new(Term::Zero)))
            )
        );
    }

    #[test]
    fn test5() {
        let src = "const :: a -> b -> a
const x y = x.

const (let x = 1 in x) 2.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Succ(Box::new(Term::Zero)))
            )
        )
    }

    #[test]
    fn test6() {
        let src = "const :: a -> b -> a
const x y = x.

let x = 1 in const x (let x = 2 in x).";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(
                Box::new(Term::Succ(Box::new(Term::Zero)))
            )
        );
    }

    #[test]
    fn test7() {
        let src = "const1 :: a -> b -> a
const1 x y = x.

const2 :: a -> b -> b
const2 x y = y.

let f = const1 <> const2 in f 0 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Zero)),
                Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))
            ])
        );
    }

    #[test]
    fn test8() {
        let src = "num :: Nat
num = 1.

num.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))
        );
    }

    #[test]
    fn test9() {
        let src = "num :: Nat
num = 0 <> 1.

const1 :: Nat -> Nat -> Nat
const1 x y = x.

const2 :: Nat -> Nat -> Nat
const2 x y = y.

let f = const1 <> const2 in f num num.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Zero)),
                Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
                Term::Return(Box::new(Term::Zero)),
                Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
                Term::Return(Box::new(Term::Zero)),
                Term::Return(Box::new(Term::Zero)),
                Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
                Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
            ])
        )
    }

    #[test]
    fn test10() {
        let src = "f :: Nat -> Nat -> Nat
f = const1 <> const2.

const1 :: Nat -> Nat -> Nat
const1 x y = x.

const2 :: Nat -> Nat -> Nat
const2 x y = y.

let num = 0 <> 1 in f num num.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Choice(vec![
                Term::Return(Box::new(Term::Zero)),
                Term::Return(Box::new(Term::Zero)),
                Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
                Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
            ])
        )
    }

    #[test]
    fn test11() {
        let src = "exists n :: Nat. n =:= 1. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
        );
    }

    #[test]
    fn test12() {
        let src = "id :: a -> a
id x = x.
        
exists n :: Nat. id n =:= 1. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
        );
    }

    #[test]
    fn test13() {
        let src = "exists n :: Nat. 0 =:= 1. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Fail
        );
    }

    #[test]
    fn test14() {
        let src = "1 + 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Succ(Box::new(Term::Zero)))))),
        );
    }

    #[test]
    fn test15() {
        let src = "addOne :: Nat -> Nat
addOne n = n + 1.

addOne 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Succ(Box::new(Term::Zero)))))),
        );
    }

    #[test]
    fn test16() {
        let src: &str = "exists n :: Nat. n =:= n+1. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val, 
            Term::Fail
        );
    }

    #[test]
    fn test17() {
        let src = "id :: Nat -> Nat
id n = exists m :: Nat. m =:= n. m.

id 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
        );
    }

    #[test]
    fn test18() {
        let src = "exists n :: Nat. n + 1 =:= 2. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
        );
    }

    #[test]
    fn test19() {
        let src = "exists n :: Nat. n + n =:= 2. n.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Zero)))),
        );
    }

    #[test]
    fn test20() {
        let src = "if true then 1 else 0.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))
        );
    }

    #[test]
    fn test21() {
        let src = "if !(1 != 2) then 0 else 1.";

        let ast = parser::parse(src).unwrap();
        let val = eval(ast);

        assert_eq!(
            val,
            Term::Return(Box::new(Term::Succ(Box::new(Term::Zero))))
        );
    }
}