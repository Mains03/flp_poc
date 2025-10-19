use std::{fmt::Display, rc::Rc};

use crate::machine::{value_type::ValueType, vclosure::VClosure};

#[derive(PartialEq, Clone, Debug)]
pub enum MValue {
    Var(usize),
    Zero,
    Succ(Rc<MValue>),
    Pair(Rc<MValue>, Rc<MValue>),
    Inl(Rc<MValue>),
    Inr(Rc<MValue>),
    Nil,
    Cons(Rc<MValue>, Rc<MValue>),
    Thunk(Rc<MComputation>)
}

impl MValue {
    pub fn up(&self, offset : usize) -> MValue {
        match self {
            MValue::Var(i) if *i < offset => MValue::Var(*i),
            MValue::Var(i) => MValue::Var(*i + 1),
            MValue::Zero => MValue::Zero,
            MValue::Succ(rc) => MValue::Succ(rc.up(offset).into()),
            MValue::Pair(rc, rc1) => MValue::Pair(rc.up(offset).into(), rc1.up(offset).into()),
            MValue::Inl(v) => MValue::Inl(v.up(offset).into()),
            MValue::Inr(v) => MValue::Inr(v.up(offset).into()),
            MValue::Nil => MValue::Nil,
            MValue::Cons(v, w) => MValue::Cons(v.up(offset).into(), w.up(offset).into()),
            MValue::Thunk(rc) => MValue::Thunk(rc.up(offset).into()),
        }
    }
}
    
fn print_nat(n : &MValue) -> Option<String> {
    fn print_nat_aux(n : &MValue, i : usize) -> Option<usize> {
        match n {
            MValue::Zero => Some(i),
            MValue::Succ(v) => print_nat_aux(&v, i+1),
            _ => None
        }
    }
    Some(print_nat_aux(n, 0)?.to_string())
}

fn print_list(xs : &MValue) -> Option<String> {
    fn print_list_aux(n : &MValue, outs : &mut Vec<String>) -> bool {
        match n {
            MValue :: Nil => true,
            MValue::Cons(v, w) => {
                let value = v.to_string();
                outs.push(value);
                print_list_aux(&w, outs)
            },
            _ => false
        }
    }
    let mut outs = vec![];
    let result = print_list_aux(xs, &mut outs);
    if result {
        let output = outs.join(", ");
        Some("[".to_owned() + &output + "]")
    } else { None }
}

impl Display for MValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MValue::Var(i) => write!(f, "idx {}", i),
            MValue::Zero => write!(f, "{}", print_nat(&MValue::Zero).expect("foo")),
            MValue::Succ(v) => {
                match print_nat(self) {
                    Some(n) => write!(f, "{}", n),
                    None => write!(f, "Succ({})", v),
                }
            },
            MValue::Nil => {
                match print_list(self) {
                    Some(xs) => write!(f, "{}", xs),
                    None => write!(f, "Nil")
                }
            }
            MValue::Cons(v, w) => {
                match print_list(self) {
                    Some(xs) => write!(f, "{}", xs),
                    None => write!(f, "Cons({}, {})", v, w)
                }
            },
            MValue::Thunk(t) => write!(f, "Thunk({})", t),
            MValue::Pair(v, w) => write!(f, "({}, {})", v, w),
            MValue::Inl(v) => write!(f, "inl({})", v),
            MValue::Inr(w) => write!(f, "inr({})", w)
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum MComputation {
    // Value eliminators
    Ifz {
        num : Rc<MValue>,
        zk : Rc<MComputation>,
        sk : Rc<MComputation>
    },
    Match {
        list : Rc<MValue>,
        nilk : Rc<MComputation>,
        consk : Rc<MComputation>
    },
    Case { 
        sum : Rc<MValue>,
        inlk : Rc<MComputation>,
        inrk : Rc<MComputation>
    },
    // CBPV primitives
    Return(Rc<MValue>),
    Bind {
        comp: Rc<MComputation>,
        cont: Rc<MComputation>,
    },
    Force(Rc<MValue>),
    Lambda { body: Rc<MComputation> },
    App {
        op: Rc<MComputation>,
        arg: Rc<MValue>
    },
    // FLP
    Choice(Vec<Rc<MComputation>>),
    Exists {
        ptype : ValueType,
        body: Rc<MComputation>
    },
    Equate {
        lhs: Rc<MValue>,
        rhs: Rc<MValue>,
        body: Rc<MComputation>
    },
    // Recursion
    Rec {
        body : Rc<MComputation>
    },
}

impl MComputation {

    pub fn thunk(self : &Rc<MComputation>) -> Rc<MValue> {
        MValue::Thunk(self.clone()).into()
    }

    pub fn up(&self, offset : usize) -> MComputation {
        match self {
            MComputation::Ifz { num, zk, sk } => 
                MComputation::Ifz { num: num.up(offset).into(), zk: zk.up(offset).into(), sk: sk.up(offset).into() },
            MComputation::Match { list, nilk, consk } => 
                MComputation::Match { list: list.up(offset).into(), nilk: nilk.up(offset).into(), consk: consk.up(offset).into() },
            MComputation::Case { sum, inlk, inrk } => 
                MComputation::Case { sum: sum.up(offset).into(), inlk: inlk.up(offset).into(), inrk: inrk.up(offset).into() },
            MComputation::Return(rc) => MComputation::Return(rc.up(offset).into()),
            MComputation::Bind { comp, cont } => MComputation::Bind { comp: comp.up(offset).into(), cont: cont.up(offset + 1).into() },
            MComputation::Force(rc) => MComputation::Force(rc.up(offset).into()),
            MComputation::Lambda { body } => MComputation::Lambda { body: body.up(offset + 1).into() },
            MComputation::App { op, arg } => MComputation::App { op: op.up(offset).into(), arg: arg.up(offset).into() },
            MComputation::Choice(vec) => MComputation::Choice(vec.iter().map(|c| c.up(offset).into()).collect()),
            MComputation::Exists { ptype, body } => MComputation::Exists { ptype: ptype.clone(), body: body.up(offset+1).into() },
            MComputation::Equate { lhs, rhs, body } => 
                MComputation::Equate { lhs: lhs.up(offset).into(), rhs: rhs.up(offset).into(), body: body.up(offset).into() },
            MComputation::Rec { body } => MComputation::Rec { body: body.up(offset+1).into() },
        }
    }
}

impl Display for MComputation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MComputation::Return(v) => write!(f, "return({})", v),
            MComputation::Bind { comp, cont } => write!(f, "{} to {}", comp, cont),
            MComputation::Force(v) => write!(f, "force({})", v),
            MComputation::Lambda { body } => write!(f, "λ({})", body),
            MComputation::App { op, arg } => write!(f, "{}({})", op, arg),
            MComputation::Choice(vec) => {
                vec.iter().map(|c| write!(f, "{} []", c)).last().expect("lol")
            },
            MComputation::Exists { ptype, body } => 
                write!(f, "exists {}. {}", ptype, body),
            MComputation::Equate { lhs, rhs, body } => 
                write!(f, "{} =:= {}. {}", lhs, rhs, body),
            MComputation::Ifz { num, zk, sk } => 
                write!(f, "ifz({}, {}, {})", num, zk, sk),
            MComputation::Rec { body } => write!(f, "rec({})", body),
            MComputation::Match { list, nilk, consk } => 
                write!(f, "match({}, {}, {})", list, nilk, consk),
            _ => todo!()
        }
    }
}