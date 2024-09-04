use std::{cell::RefCell, rc::Rc};

use crate::cbpv::Term;

pub fn equate(mut lhs: Term, mut rhs: Term) -> bool {
    let flag;
    
    loop {
        match lhs {
            Term::Zero =>{
                flag = match rhs {
                    Term::Zero => true,
                    Term::TypedVar(shape) => if shape.borrow().is_some() {
                        match shape.borrow().clone().unwrap() {
                            Term::Zero => true,
                            _ => false
                        }
                    } else {
                        shape.replace(Some(Term::Zero));

                        true
                    },
                    _ => false
                };

                break;
            },
            Term::Succ(lhs_term) => match rhs {
                Term::Succ(rhs_term) => {
                    lhs = *lhs_term;
                    rhs = *rhs_term;
                },
                Term::TypedVar(shape) => if shape.borrow().is_some() {
                    match shape.borrow().clone().unwrap() {
                        Term::Succ(rhs_term) => {
                            lhs = *lhs_term;
                            rhs = *rhs_term;
                        },
                        _ => {
                            flag = false;
                            break;
                        }
                    }
                } else {
                    rhs = Term::TypedVar(Rc::new(RefCell::new(None)));
                    shape.replace(Some(Term::Succ(Box::new(rhs.clone()))));

                    lhs = *lhs_term;
                },
                _ => {
                    flag = false;
                    break;
                }
            },
            Term::TypedVar(lhs_shape) => match rhs {
                Term::Zero => if lhs_shape.borrow().is_some() {
                    flag = match lhs_shape.borrow().clone().unwrap() {
                        Term::Zero => true,
                        _ => false
                    };

                    break;
                } else {
                    lhs_shape.replace(Some(Term::Zero));

                    flag = true;
                    break;
                },
                Term::Succ(rhs_term) => if lhs_shape.borrow().is_some() {
                    match lhs_shape.borrow().clone().unwrap() {
                        Term::Succ(lhs_term) => {
                            lhs = *lhs_term;
                            rhs = *rhs_term;
                        },
                        _ => {
                            flag = false;
                            break;
                        }
                    }
                } else {
                    lhs = Term::TypedVar(Rc::new(RefCell::new(None)));
                    lhs_shape.replace(Some(Term::Succ(Box::new(lhs.clone()))));

                    rhs = *rhs_term;
                },
                Term::TypedVar(rhs_shape) => if is_cyle(&lhs_shape, &rhs_shape) {
                    flag = false;
                    break;
                } else if lhs_shape.borrow().is_some() {
                    match lhs_shape.borrow().clone().unwrap() {
                        Term::Zero => if rhs_shape.borrow().is_some() {
                            flag = match rhs_shape.borrow().clone().unwrap() {
                                Term::Zero => true,
                                _ => false
                            };

                            break;
                        } else {
                            rhs_shape.replace(Some(Term::Zero));

                            flag = true;
                            break;
                        },
                        Term::Succ(lhs_term) => if rhs_shape.borrow().is_some() {
                            match rhs_shape.borrow().clone().unwrap() {
                                Term::Succ(rhs_term) => {
                                    lhs = *lhs_term;
                                    rhs = *rhs_term;
                                },
                                _ => {
                                    flag = false;
                                    break;
                                }
                            }
                        } else {
                            rhs = Term::TypedVar(Rc::new(RefCell::new(None)));
                            rhs_shape.replace(Some(Term::Succ(Box::new(rhs.clone()))));

                            lhs = *lhs_term;
                        },
                        _ => unreachable!()
                    }
                } else {
                    if rhs_shape.borrow().is_some() {
                        match rhs_shape.borrow().clone().unwrap() {
                            Term::Zero => {
                                lhs_shape.replace(Some(Term::Zero));

                                flag = true;
                                break;
                            },
                            Term::Succ(rhs_term) => {
                                lhs = Term::TypedVar(Rc::new(RefCell::new(None)));
                                lhs_shape.replace(Some(Term::Succ(Box::new(lhs.clone()))));

                                rhs = *rhs_term;
                            },
                            _ => unreachable!()
                        }
                    } else {
                        flag = true;
                        break;
                    }
                },
                _ => unreachable!()
            },
            _ => unreachable!()
        }
    }

    flag
}

fn is_cyle(lhs: &Rc<RefCell<Option<Term>>>, rhs: &Rc<RefCell<Option<Term>>>) -> bool {
    vec![lhs, rhs].into_iter()
        .fold(false, |acc, x| {
            if acc {
                true
            } else {
                let mut tmp = Rc::clone(x);

                loop {
                    if tmp.as_ptr() == rhs.as_ptr() {
                        return true;
                    } else {
                        match tmp.clone().borrow().clone() {
                            Some(term) => match term {
                                Term::Succ(term) => match *term {
                                    Term::TypedVar(shape) => tmp = shape,
                                    _ => return false
                                },
                                _ => return false
                            },
                            None => return false
                        }
                    }
                }
            }
        })
}