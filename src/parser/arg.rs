
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Arg {
    Ident(String),
    Pair(Box<Arg>, Box<Arg>)
}