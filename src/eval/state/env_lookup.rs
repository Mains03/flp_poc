use super::frame::env::env_value::EnvValue;

pub trait EnvLookup {
    fn lookup(&self, var: &String) -> Option<EnvValue>;
}