use std::collections::HashMap;

use state::State;

use crate::cbpv::Term;

pub use state::LocationsClone;

mod state;

pub fn eval(cbpv: HashMap<String, Term>) -> Term {
    let mut states = vec![State::new(cbpv)];

    loop {
        states = states.into_iter()
            .flat_map(|s| s.step())
            .filter(|s| !s.is_fail())
            .collect();

        let flag = states.iter()
            .fold(false, |acc, x| acc || x.is_value());

        if flag {
            states = states.into_iter()
                .filter(|s| s.is_value())
                .collect();

            break;
        } else if states.len() == 0 {
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