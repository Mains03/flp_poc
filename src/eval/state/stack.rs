use crate::cbpv::Term;

#[derive(Clone, Debug)]
pub struct Stack {
    stack: Vec<StackTerm>
}

#[derive(Clone, Debug)]
pub enum StackTerm {
    Cont(String, Term),
    Term(Term)
}

impl Stack {
    pub fn new() -> Self {
        Stack { stack: vec![] }
    }

    pub fn push(&mut self, term: StackTerm) {
        todo!()
    }

    pub fn pop(&mut self) -> Option<StackTerm> {
        todo!()
    }
}