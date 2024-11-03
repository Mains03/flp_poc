use std::collections::HashMap;

use state::State;

use crate::cbpv::Term;

pub use state::LocationsClone;
pub use state::Env;

pub mod machine;
mod state;

pub fn eval(cbpv: HashMap<String, Term>, solution_count: usize) -> Term {
    let mut states = vec![State::new(cbpv)];
    let mut values = vec![];

    loop {
        states = states.into_iter()
            .flat_map(|s| s.step())
            .filter(|s| !s.is_fail())
            .flat_map(|s| if s.is_value() {
                values.push(s.term().clone());
                vec![]
            } else {
                vec![s]
            })
            .collect();

        if states.len() == 0 {
            break;
        } else if values.len() >= solution_count {
            break;
        }
    }

    values.truncate(solution_count);

    if values.len() == 0 {
        Term::Fail
    } else if values.len() == 1 {
        values.remove(0).term().clone()
    } else {
        Term::Choice(values)
    }
}