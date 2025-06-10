use std::fmt::Display;

#[derive(PartialEq, Clone, Debug)]
pub enum ValueType {
    Nat,
    Product(Box<ValueType>, Box<ValueType>),
    Sum(Box<ValueType>, Box<ValueType>),
    List(Box<ValueType>),
    Thunk(Box<ComputationType>)
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Nat => write!(f, "Nat"),
            ValueType::List(value_type) => write!(f, "[{}]", value_type),
            ValueType::Thunk(computation_type) => write!(f, "THONK"),
            ValueType::Product(value_type, value_type1) => todo!(),
            ValueType::Sum(value_type, value_type1) => todo!(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum ComputationType {
    Return(Box<ValueType>),
    Arrow(Box<ValueType>, Box<ComputationType>)
}