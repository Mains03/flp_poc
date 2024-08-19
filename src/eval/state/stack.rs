use crate::cbpv::Term;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stack {
    stack: Vec<StackTerm>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StackTerm {
    Cont(String, Term),
    Term(Term),
    PopEnv,
}

impl Stack {
    pub fn new() -> Self {
        Stack { stack: vec![] }
    }

    pub fn push(&mut self, term: StackTerm) {
        self.stack.push(term);
    }

    pub fn pop(&mut self) -> Option<StackTerm> {
        if self.stack.len() == 0 {
            None
        } else {
            Some(self.stack.remove(self.stack.len()-1))
        }
    }
}