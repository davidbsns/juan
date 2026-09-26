use std::collections::HashMap;

use juan_bytecode::{Chunk, Opcode};

use crate::{
    call_frame::CallFrame,
    error::VMError,
    value::{Value, ValueType},
};

pub struct VM {
    chunks: HashMap<u32, Chunk>,
    chunks_lookup: HashMap<String, u32>,
}

mod call_frame;
pub mod error;
mod value;

impl VM {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            chunks_lookup: HashMap::new(),
        }
    }

    pub fn load(&mut self, name: String, id: u32, chunk: Chunk) -> Result<(), VMError> {
        if self.chunks.get(&id).is_some() {
            return Err(VMError::ChunkAlreadyExists(name));
        }

        self.chunks.insert(id, chunk);
        self.chunks_lookup.insert(name, id);

        Ok(())
    }

    pub fn run(&mut self, id: u32) -> Result<Value, VMError> {
        let mut stack = Vec::new();
        let mut call_frames: Vec<CallFrame> = Vec::new();

        call_frames.push(CallFrame {
            function_id: id,
            ip: 0,
            stack_base: 0,
        });

        while let Some(call_frame) = call_frames.last_mut() {
            let Some(chunk) = self.chunks.get(&call_frame.function_id) else {
                return Err(VMError::ChunkDoesntExist(format!(
                    "Id: {}",
                    call_frame.function_id
                )));
            };

            let bytes = chunk.bytes();

            if call_frame.ip < bytes.len() {
                let byte = bytes[call_frame.ip];
                call_frame.ip += 1;

                match Opcode::try_from(byte) {
                    Ok(Opcode::PushInt) => {
                        let Some(numbers) = bytes.get(call_frame.ip..call_frame.ip + 4) else {
                            return Err(VMError::UnexpectedEndOfBytecode(call_frame.ip));
                        };

                        let numbers: &[u8; 4] = numbers.try_into()?;
                        let num = i32::from_le_bytes(*numbers);

                        stack.push(Value::I32(num));
                        call_frame.ip += 4;
                    }

                    Ok(Opcode::Call) => {
                        let Some(numbers) = bytes.get(call_frame.ip..call_frame.ip + 4) else {
                            return Err(VMError::UnexpectedEndOfBytecode(call_frame.ip));
                        };

                        let numbers: &[u8; 4] = numbers.try_into()?;
                        let func_id = u32::from_le_bytes(*numbers);
                        call_frame.ip += 4;

                        call_frames.push(CallFrame {
                            function_id: func_id,
                            ip: 0,
                            stack_base: stack.len(),
                        });
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

                    Ok(
                        op @ (Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Rem),
                    ) => {
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

                    Ok(Opcode::Return) => {
                        let Some(frame) = call_frames.pop() else {
                            return Err(VMError::StackUnderflow);
                        };

                        if stack.len() <= frame.stack_base {
                            return Err(VMError::StackUnderflow);
                        }

                        let Some(value) = stack.pop() else {
                            return Err(VMError::StackUnderflow);
                        };

                        stack.truncate(frame.stack_base);

                        if call_frames.is_empty() {
                            return Ok(value);
                        } else {
                            stack.push(value);
                        }
                    }

                    Err(e) => return Err(VMError::TryFromPrimitiveOpcode(e)),
                }
            } else {
                return Err(VMError::UnexpectedEndOfBytecode(call_frame.ip));
            }
        }

        Ok(Value::Unit)
    }
}
