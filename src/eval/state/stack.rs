use super::StateTerm;

#[derive(Debug)]
pub struct Stack {
    stack: Vec<StackTerm>
}

#[derive(Debug)]
pub enum StackTerm {
    Cont(String, StateTerm),
    Term(StateTerm),
    Release(String),
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

    pub fn is_empty(&self) -> bool {
        self.stack.len() == 0
    }
}