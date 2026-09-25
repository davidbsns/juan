use std::collections::HashMap;

use juan_bytecode::{Chunk, Opcode};

use crate::error::VMError;

pub struct VM {
    chunks: HashMap<String, Chunk>,
}

pub mod error;

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

        let mut stack: Vec<i32> = Vec::new();
        let mut ptr: usize = 0;

        let bytes = chunk.bytes();

        while ptr < bytes.len() {
            let byte = bytes[ptr];
            ptr += 1;

            match byte {
                b if b == Opcode::PushInt as u8 => {
                    let Some(numbers) = bytes.get(ptr..ptr + 4) else {
                        return Err(VMError::UnexpectedEndOfBytecode(ptr));
                    };

                    let numbers: &[u8; 4] = numbers.try_into()?;
                    let num = i32::from_le_bytes(*numbers);

                    stack.push(num);
                    ptr += 4;
                }

                b if b == Opcode::Add as u8 => {
                    let Some(rhs) = stack.pop() else {
                        return Err(VMError::StackUnderflow);
                    };
                    let Some(lhs) = stack.pop() else {
                        return Err(VMError::StackUnderflow);
                    };

                    match lhs.checked_add(rhs) {
                        Some(a) => stack.push(a),
                        None => return Err(VMError::IntegerOverflow(lhs, rhs, '+')),
                    }
                }

                b if b == Opcode::Halt as u8 => {
                    println!("{:?}", stack);
                    return Ok(());
                }

                _ => {
                    return Err(VMError::InvalidOpcode(byte));
                }
            }
        }

        Err(VMError::UnexpectedEndOfBytecode(ptr))
    }
}
