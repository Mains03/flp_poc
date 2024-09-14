#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Arg {
    Ident(String),
    Pair(String, String)
}