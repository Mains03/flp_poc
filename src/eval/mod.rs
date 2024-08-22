use std::collections::HashMap;

use state::State;

use crate::cbpv::Term;

mod state;

pub fn eval(cbpv: HashMap<String, Term>) -> Term {
    let mut states = vec![State::new(cbpv)];

    loop {
        let old = states.clone();

        states = states.into_iter()
            .flat_map(|s| s.step())
            .collect();

        if old == states {
            break;
        }
    }

    if states.len() == 0 {
        Term::Fail
    } else if states.len() == 1 {
        states.remove(0).as_term()
    } else {
        Term::Choice(
            states.into_iter()
                .fold(vec![], |mut acc, x| {
                    acc.push(x.as_term());
                    acc
                })
        )
    }
}