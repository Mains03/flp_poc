use std::collections::HashMap;

use env::env::Env;
use stack::Stack;

pub mod env;
pub mod stack;

#[derive(Debug)]
pub struct Frame {
    env: Env,
    stack: Stack
}

impl Frame {
    pub fn new() -> Self {
        Frame { env: Env::new(), stack: Stack::new() }
    }

    pub fn env(&mut self) -> &mut Env {
        &mut self.env
    }

    pub fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }

    pub fn stack_ref(&self) -> &Stack {
        &self.stack
    }
}

impl Clone for Frame {
    fn clone(&self) -> Self {
        let mut locations = HashMap::new();

        Self {
            env: self.env.clone_with_locations(&mut locations),
            stack: self.stack.clone_with_locations(&mut locations)
        }
    }
}
