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
            let chunk = self.get_chunk(call_frame.function_id)?;

            let bytes = chunk.bytes();

            if call_frame.ip >= bytes.len() {
                return Err(VMError::UnexpectedEndOfBytecode(call_frame.ip));
            }

            let byte = bytes[call_frame.ip];
            call_frame.ip += 1;

            match Opcode::try_from(byte) {
                Ok(Opcode::PushInt) => {
                    let num = self.read_i32(&mut call_frame.ip, bytes)?;
                    stack.push(Value::I32(num));
                }

                Ok(Opcode::Call) => {
                    let func_id = self.read_u32(&mut call_frame.ip, bytes)?;
                    let arg_count = self.read_u32(&mut call_frame.ip, bytes)?;

                    let stack_base = stack
                        .len()
                        .checked_sub(arg_count as usize)
                        .ok_or(VMError::StackUnderflow)?;

                    call_frames.push(CallFrame {
                        function_id: func_id,
                        ip: 0,
                        stack_base,
                    });
                }

                Ok(Opcode::LoadLocal) => {
                    let id = self.read_u32(&mut call_frame.ip, bytes)?;
                    let pos = call_frame.stack_base + id as usize;
                    let value = stack.get(pos).ok_or(VMError::InvalidSlot(pos))?;

                    stack.push(*value);
                }

                Ok(Opcode::Pop) => {
                    stack.pop().ok_or(VMError::StackUnderflow)?;
                }

                Ok(Opcode::PushUnit) => {
                    stack.push(Value::Unit);
                }

                Ok(Opcode::Neg) => {
                    let operand = self.pop_i32(&mut stack)?;

                    match operand.checked_neg() {
                        Some(i) => stack.push(Value::I32(i)),
                        None => return Err(VMError::UnaryIntegerOverflow(operand, '-')),
                    }
                }

                Ok(op @ (Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Rem)) => {
                    let rhs = self.pop_i32(&mut stack)?;
                    let lhs = self.pop_i32(&mut stack)?;

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
                    let frame = call_frames.pop().ok_or(VMError::StackUnderflow)?;

                    if stack.len() <= frame.stack_base {
                        return Err(VMError::StackUnderflow);
                    }

                    let value = stack.pop().ok_or(VMError::StackUnderflow)?;
                    stack.truncate(frame.stack_base);

                    if call_frames.is_empty() {
                        return Ok(value);
                    }

                    stack.push(value);
                }

                Err(e) => return Err(VMError::TryFromPrimitiveOpcode(e)),
            }
        }

        Ok(Value::Unit)
    }
}

impl VM {
    fn fetch_4_bytes(&self, ip: &mut usize, bytes: &[u8]) -> Result<[u8; 4], VMError> {
        let Some(slice) = bytes.get(*ip..*ip + 4) else {
            return Err(VMError::UnexpectedEndOfBytecode(*ip));
        };

        *ip += 4;
        Ok(slice.try_into()?)
    }

    fn read_u32(&self, ip: &mut usize, bytes: &[u8]) -> Result<u32, VMError> {
        Ok(u32::from_le_bytes(self.fetch_4_bytes(ip, bytes)?))
    }

    fn read_i32(&self, ip: &mut usize, bytes: &[u8]) -> Result<i32, VMError> {
        Ok(i32::from_le_bytes(self.fetch_4_bytes(ip, bytes)?))
    }

    fn pop_i32(&self, stack: &mut Vec<Value>) -> Result<i32, VMError> {
        match stack.pop().ok_or(VMError::StackUnderflow)? {
            Value::I32(i) => Ok(i),
            invalid => Err(VMError::InvalidOperandType(invalid.kind(), ValueType::I32)),
        }
    }

    fn get_chunk(&self, func_id: u32) -> Result<&Chunk, VMError> {
        self.chunks
            .get(&func_id)
            .ok_or_else(|| VMError::ChunkDoesntExist(format!("Id: {func_id}")))
    }
}
