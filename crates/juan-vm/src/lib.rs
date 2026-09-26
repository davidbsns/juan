use std::collections::HashMap;

use juan_bytecode::{Chunk, Opcode};

use crate::{
    error::VMError,
    value::{Value, ValueType},
};

pub struct VM {
    chunks: HashMap<String, Chunk>,
}

pub mod error;
mod value;

impl VM {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn load(&mut self, name: String, chunk: Chunk) -> Result<(), VMError> {
        if self.chunks.get(&name).is_some() {
            return Err(VMError::ChunkAlreadyExists(name));
        }

        self.chunks.insert(name, chunk);

        Ok(())
    }

    pub fn run(&self, name: String) -> Result<(), VMError> {
        let Some(chunk) = self.chunks.get(&name) else {
            return Err(VMError::ChunkDoesntExist(name));
        };

        let mut stack: Vec<Value> = Vec::new();
        let mut ptr: usize = 0;

        let bytes = chunk.bytes();

        while ptr < bytes.len() {
            let byte = bytes[ptr];
            ptr += 1;

            match Opcode::try_from(byte) {
                Ok(Opcode::PushInt) => {
                    let Some(numbers) = bytes.get(ptr..ptr + 4) else {
                        return Err(VMError::UnexpectedEndOfBytecode(ptr));
                    };

                    let numbers: &[u8; 4] = numbers.try_into()?;
                    let num = i32::from_le_bytes(*numbers);

                    stack.push(Value::I32(num));
                    ptr += 4;
                }

                Ok(Opcode::Pop) => match stack.pop() {
                    Some(_) => {}
                    None => return Err(VMError::StackUnderflow),
                },

                Ok(Opcode::PushUnit) => {
                    stack.push(Value::Unit);
                }

                Ok(Opcode::Neg) => {
                    let Some(operand) = stack.pop() else {
                        return Err(VMError::StackUnderflow);
                    };

                    match operand {
                        Value::I32(i) => match i.checked_neg() {
                            Some(a) => stack.push(Value::I32(a)),
                            None => return Err(VMError::UnaryIntegerOverflow(i, '-')),
                        },

                        invalid => {
                            return Err(VMError::InvalidOperandType(
                                invalid.kind(),
                                ValueType::I32,
                            ));
                        }
                    }
                }

                Ok(op @ (Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Rem)) => {
                    let Some(rhs) = stack.pop() else {
                        return Err(VMError::StackUnderflow);
                    };
                    let Some(lhs) = stack.pop() else {
                        return Err(VMError::StackUnderflow);
                    };

                    let rhs = match rhs {
                        Value::I32(i) => i,

                        invalid => {
                            return Err(VMError::InvalidOperandType(
                                invalid.kind(),
                                ValueType::I32,
                            ));
                        }
                    };

                    let lhs = match lhs {
                        Value::I32(i) => i,

                        invalid => {
                            return Err(VMError::InvalidOperandType(
                                invalid.kind(),
                                ValueType::I32,
                            ));
                        }
                    };

                    let (op_char, res) = match op {
                        Opcode::Add => ('+', lhs.checked_add(rhs)),
                        Opcode::Sub => ('-', lhs.checked_sub(rhs)),
                        Opcode::Mul => ('*', lhs.checked_mul(rhs)),
                        Opcode::Rem => {
                            if rhs == 0 {
                                return Err(VMError::RemainderByZero);
                            }

                            ('%', lhs.checked_rem(rhs))
                        }
                        Opcode::Div => {
                            if rhs == 0 {
                                return Err(VMError::DivisionByZero);
                            }

                            ('/', lhs.checked_div(rhs))
                        }
                        _ => unreachable!(),
                    };

                    match res {
                        Some(a) => stack.push(Value::I32(a)),
                        None => return Err(VMError::IntegerOverflow(lhs, rhs, op_char)),
                    }
                }

                Ok(Opcode::Halt) => {
                    println!("{:?}", stack);
                    return Ok(());
                }

                Err(e) => return Err(VMError::TryFromPrimitiveOpcode(e)),
            }
        }

        Err(VMError::UnexpectedEndOfBytecode(ptr))
    }
}
