pub enum Type<'a> {
    Arrow(Box<Type<'a>>, Box<Type<'a>>),
    Ident(&'a str),
}