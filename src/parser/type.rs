#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Arrow(Box<Type>, Box<Type>),
    Ident(String),
    List(Box<Type>),
    Pair(Box<Type>, Box<Type>)
}