use std::collections::HashMap;

use state::State;

use crate::cbpv::Term;

mod state;

pub fn eval(cbpv: HashMap<String, Term>) -> Term {
    let mut states = vec![State::new(cbpv)];

    loop {
        println!("{:#?}", states);
        let flag = states.iter()
                    .fold(true, |acc, x| acc && x.is_value());

        if flag {
            break;
        } else {
            states = states.into_iter()
                        .flat_map(|s| s.step())
                        .collect();
        }
    }

    if states.len() == 0 {
        Term::Fail
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