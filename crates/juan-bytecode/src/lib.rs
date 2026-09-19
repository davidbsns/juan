pub use juan_ast::SyntaxTree;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Opcode {
    PushInt,
    Add,
    Halt,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Chunk {
    bytes: Vec<u8>,
    generation: u32,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            bytes: vec![],
            generation: 1,
        }
    }

    pub fn write_opcode(&mut self, op: Opcode) {
        self.bytes.push(op as u8);
    }

    pub fn write_i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
