#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    I32(i32),
    Unit,
}

impl Value {
    pub fn kind(&self) -> ValueType {
        match self {
            Value::I32(_) => ValueType::I32,
            Value::Unit => ValueType::Unit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueType {
    I32,
    Unit,
}
