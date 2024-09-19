use super::stm::Stm;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Case {
    Nat(NatCase),
    List(ListCase)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NatCase {
    pub zero: Option<NatZeroCase>,
    pub succ: Option<NatSuccCase>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NatZeroCase {
    pub stm: Box<Stm>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NatSuccCase {
    pub var: String,
    pub stm: Box<Stm>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListCase {
    pub empty: Option<ListEmptyCase>,
    pub cons: Option<ListConsCase>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListEmptyCase {
    pub stm: Box<Stm>
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListConsCase {
    pub x: String,
    pub xs: String,
    pub stm: Box<Stm>
}

impl Case {
    pub fn combine(self, case: Case) -> Case {
        match self {
            Case::Nat(mut nat_case) => match case {
                Case::Nat(other_nat_case) => {
                    if other_nat_case.zero.is_some() {
                        if nat_case.zero.is_some() {
                            panic!()
                        } else {
                            nat_case.zero = other_nat_case.zero;
                        }
                    }

                    if other_nat_case.succ.is_some() {
                        if nat_case.succ.is_some() {
                            panic!()
                        } else {
                            nat_case.succ = other_nat_case.succ;
                        }
                    }

                    Case::Nat(nat_case)
                },
                _ => panic!()
            },
            Case::List(mut list_case) => match case {
                Case::List(other_list_case) => {
                    if other_list_case.empty.is_some() {
                        if list_case.empty.is_some() {
                            panic!()
                        } else {
                            list_case.empty = other_list_case.empty;
                        }
                    }

                    if other_list_case.cons.is_some() {
                        if list_case.cons.is_some() {
                            panic!()
                        } else {
                            list_case.cons = other_list_case.cons;
                        }
                    }

                    Case::List(list_case)
                },
                _ => panic!()
            }
        }
    }
}
