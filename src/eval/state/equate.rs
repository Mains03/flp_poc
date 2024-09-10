use std::{cell::RefCell, rc::Rc};

use crate::cbpv::{term_ptr::TermPtr, Term};

pub fn equate(mut lhs: TermPtr, mut rhs: TermPtr) -> bool {
    let flag;
    
    loop {
        match lhs.clone().term() {
            Term::Zero => {
                flag = match rhs.term() {
                    Term::Zero => true,
                    Term::TypedVar(shape) => if shape.borrow().is_some() {
                        match shape.borrow().as_ref().unwrap().term() {
                            Term::Zero => true,
                            _ => false
                        }
                    } else {
                        shape.replace(Some(TermPtr::from_term(Term::Zero)));

                        true
                    },
                    _ => false
                };

                break;
            },
            Term::Succ(lhs_term) => match rhs.clone().term() {
                Term::Succ(rhs_term) => {
                    lhs = lhs_term.clone();
                    rhs = rhs_term.clone();
                },
                Term::TypedVar(shape) => if shape.borrow().is_some() {
                    match shape.borrow().as_ref().unwrap().term() {
                        Term::Succ(rhs_term) => {
                            lhs = lhs_term.clone();
                            rhs = rhs_term.clone();
                        },
                        _ => {
                            flag = false;
                            break;
                        }
                    }
                } else {
                    rhs = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                    shape.replace(Some(TermPtr::from_term(Term::Succ(rhs.clone()))));

                    lhs = lhs_term.clone();
                },
                _ => {
                    flag = false;
                    break;
                }
            },
            Term::Nil => {
                flag = match rhs.term() {
                    Term::Nil => true,
                    Term::TypedVar(shape) => if shape.borrow().is_some() {
                        match shape.borrow().as_ref().unwrap().term() {
                            Term::Nil => true,
                            _ => false
                        }
                    } else {
                        shape.replace(Some(TermPtr::from_term(Term::Nil)));

                        true
                    },
                    _ => false
                };

                break;
            },
            Term::Cons(x, xs) => match rhs.clone().term() {
                Term::Cons(y, ys) => if equate(x.clone(), y.clone()) {
                    lhs = xs.clone();
                    rhs = ys.clone();
                } else {
                    flag = false;
                    break;
                },
                Term::TypedVar(shape) => if shape.borrow().is_some() {
                    match shape.borrow().as_ref().unwrap().term() {
                        Term::Cons(y, ys) => if equate(x.clone(), y.clone()) {
                            lhs = xs.clone();
                            rhs = ys.clone();
                        } else {
                            flag = false;
                            break;
                        },
                        _ => {
                            flag = false;
                            break;
                        }
                    }
                } else {
                    let y = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                    let ys = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));

                    shape.replace(Some(TermPtr::from_term(Term::Cons(y.clone(), ys.clone()))));

                    equate(x.clone(), y);

                    lhs = xs.clone();
                    rhs = ys;
                },
                _ => {
                    flag = false;
                    break;
                }
            },
            Term::TypedVar(lhs_shape) => match rhs.clone().term() {
                Term::Zero => if lhs_shape.borrow().is_some() {
                    flag = match lhs_shape.borrow().as_ref().unwrap().term() {
                        Term::Zero => true,
                        _ => false
                    };

                    break;
                } else {
                    lhs_shape.replace(Some(TermPtr::from_term(Term::Zero)));

                    flag = true;
                    break;
                },
                Term::Succ(rhs_term) => if lhs_shape.borrow().is_some() {
                    match lhs_shape.borrow().as_ref().unwrap().term() {
                        Term::Succ(lhs_term) => {
                            lhs = lhs_term.clone();
                            rhs = rhs_term.clone();
                        },
                        _ => {
                            flag = false;
                            break;
                        }
                    }
                } else {
                    lhs = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                    lhs_shape.replace(Some(TermPtr::from_term(Term::Succ(lhs.clone()))));

                    rhs = rhs_term.clone();
                },
                Term::Nil => if lhs_shape.borrow().is_some() {
                    flag = match lhs_shape.borrow().as_ref().unwrap().term() {
                        Term::Nil => true,
                        _ => false
                    };

                    break;
                } else {
                    lhs_shape.replace(Some(TermPtr::from_term(Term::Nil)));

                    flag = true;
                    break;
                },
                Term::Cons(y, ys) => if lhs_shape.borrow().is_some() {
                    match lhs_shape.borrow().as_ref().unwrap().term() {
                        Term::Cons(x, xs) => if equate(x.clone(), y.clone()) {
                            lhs = xs.clone();
                            rhs = ys.clone();
                        } else {
                            flag = false;
                            break;
                        },
                        _ => {
                            flag = false;
                            break;
                        }
                    }
                } else {
                    let x = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                    let xs = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));

                    lhs_shape.replace(Some(TermPtr::from_term(Term::Cons(x.clone(), xs.clone()))));

                    equate(x, y.clone());

                    lhs = xs;
                    rhs = ys.clone();
                },
                Term::TypedVar(rhs_shape) => if is_cyle(lhs_shape, rhs_shape) {
                    flag = false;
                    break;
                } else if lhs_shape.borrow().is_some() {
                    match lhs_shape.borrow().as_ref().unwrap().term() {
                        Term::Zero => if rhs_shape.borrow().is_some() {
                            flag = match rhs_shape.borrow().as_ref().unwrap().term() {
                                Term::Zero => true,
                                _ => false
                            };

                            break;
                        } else {
                            rhs_shape.replace(Some(TermPtr::from_term(Term::Zero)));

                            flag = true;
                            break;
                        },
                        Term::Succ(lhs_term) => if rhs_shape.borrow().is_some() {
                            match rhs_shape.borrow().as_ref().unwrap().term() {
                                Term::Succ(rhs_term) => {
                                    lhs = lhs_term.clone();
                                    rhs = rhs_term.clone();
                                },
                                _ => {
                                    flag = false;
                                    break;
                                }
                            }
                        } else {
                            rhs = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                            rhs_shape.replace(Some(TermPtr::from_term(Term::Succ(rhs.clone()))));

                            lhs = lhs_term.clone();
                        },
                        _ => unreachable!()
                    }
                } else {
                    if rhs_shape.borrow().is_some() {
                        match rhs_shape.borrow().as_ref().unwrap().term(){
                            Term::Zero => {
                                lhs_shape.replace(Some(TermPtr::from_term(Term::Zero)));

                                flag = true;
                                break;
                            },
                            Term::Succ(rhs_term) => {
                                lhs = TermPtr::from_term(Term::TypedVar(Rc::new(RefCell::new(None))));
                                lhs_shape.replace(Some(TermPtr::from_term(Term::Succ(lhs.clone()))));

                                rhs = rhs_term.clone();
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

fn is_cyle(lhs: &Rc<RefCell<Option<TermPtr>>>, rhs: &Rc<RefCell<Option<TermPtr>>>) -> bool {
    vec![(lhs, rhs), (rhs, lhs)].into_iter()
        .fold(false, |acc, (lhs, rhs)| {
            if acc {
                true
            } else {
                let mut tmp = Rc::clone(lhs);

                loop {
                    if Rc::ptr_eq(&tmp, rhs) {
                        return true;
                    } else {
                        match tmp.clone().borrow().as_ref() {
                            Some(term_ptr) => match term_ptr.term() {
                                Term::Succ(term_ptr) => match term_ptr.term() {
                                    Term::TypedVar(shape) => tmp = Rc::clone(shape),
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