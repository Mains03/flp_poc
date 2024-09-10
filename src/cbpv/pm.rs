use super::Term;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PM {
    PMNat(PMNat),
    PMList(PMList)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMNat {
    pub var: String,
    pub zero: Box<Term>,
    pub succ: PMNatSucc
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMNatSucc {
    pub var: String,
    pub body: Box<Term>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMList {
    pub var: String,
    pub nil: Box<Term>,
    pub cons: PMListCons
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMListCons {
    pub x: String,
    pub xs: String,
    pub body: Box<Term>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMSucc {
    pub var: String,
    pub body: Box<Term>
}