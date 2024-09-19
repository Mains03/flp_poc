use super::{free_vars::FreeVars, term_ptr::TermPtr};

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

impl PM {
    pub fn free_vars(&self) -> FreeVars {
        match self {
            PM::PMNat(pm_nat) => {
                let mut free_vars = FreeVars::from_vars(vec![pm_nat.var.clone()]);
                free_vars.extend(pm_nat.zero.free_vars());
                free_vars.extend(pm_nat.succ.body.free_vars());
                free_vars.remove_var(&pm_nat.succ.var);
                free_vars
            },
            PM::PMList(pm_list) => {
                let mut free_vars = FreeVars::from_vars(vec![pm_list.var.clone()]);
                free_vars.extend(pm_list.nil.free_vars());
                free_vars.extend(pm_list.cons.body.free_vars());
                free_vars.remove_var(&pm_list.cons.x);
                free_vars.remove_var(&pm_list.cons.xs);
                free_vars
            }
        }
    }
}