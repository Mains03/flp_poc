use super::term_ptr::TermPtr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PM {
    PMNat(PMNat),
    PMList(PMList)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMNat {
    pub var: String,
    pub zero: TermPtr,
    pub succ: PMNatSucc
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMNatSucc {
    pub var: String,
    pub body: TermPtr
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMList {
    pub var: String,
    pub nil: TermPtr,
    pub cons: PMListCons
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMListCons {
    pub x: String,
    pub xs: String,
    pub body: TermPtr
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PMSucc {
    pub var: String,
    pub body: TermPtr
}