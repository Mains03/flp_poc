use crate::{cbpv::Term, parser::syntax::stm::Stm};

use super::Translate;

impl Translate for Stm {
    fn translate(self) -> Term {
        match self {
            Stm::If { cond, then, r#else } => Term::Bind {
                var: "".to_string(),
                val: Box::new(cond.translate()),
                body: Box::new(Term::If {
                    cond: "".to_string(),
                    then: Box::new(then.translate()),
                    r#else: Box::new(r#else.translate())
                })
            },
            Stm::Let { var, val, body } => Term::Bind {
                var: var,
                val: Box::new(val.translate()),
                body: Box::new(body.translate())
            },
            Stm::Exists { var, r#type, body } => Term::Exists {
                var,
                r#type: r#type.clone(),
                body: Box::new(body.translate())
            },
            Stm::Equate { lhs, rhs, body } => Term::Bind {
                var: "0".to_string(),
                val: Box::new(lhs.translate()),
                body: Box::new(Term::Bind {
                    var: "1".to_string(),
                    val: Box::new(rhs.translate()),
                    body: Box::new(Term::Equate {
                        lhs: "0".to_string(),
                        rhs: "1".to_string(),
                        body: Box::new(body.translate())
                    })
                })
            },
            Stm::Choice(exprs) => Term::Choice(
                exprs.into_iter()
                    .map(|e| e.translate()).collect()
            ),
            Stm::Expr(e) => e.translate()
        }
    }
}