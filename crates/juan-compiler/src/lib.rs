use std::{collections::HashMap, fs};

use juan_ast::{Expr, Literal, Op, SyntaxTree};
use juan_bytecode::{Chunk, Opcode};

use crate::error::CompilerError;

pub struct Compiler {
    chunks: HashMap<String, Chunk>,
}

pub mod error;

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn read_module(&mut self, tree: SyntaxTree, src: String) {
        // TODO: only create chunks inside blocks/functions

        let mut chunk = Chunk::new();
        let mut module_name = None;

        for (_, node) in tree {
            match node.expr {
                Expr::Module => {
                    let (start, len) = node.span.unpack();
                    module_name = Some(&src[start..start + len]);
                }

                Expr::Literal(literal) => match literal {
                    Literal::Int(i) => {
                        chunk.write_opcode(Opcode::PushInt);
                        chunk.write_i32(i as i32);
                    }
                },

                Expr::BinaryOp { op, .. } => {
                    let opcode = match op {
                        Op::Add => Opcode::Add,
                    };

                    chunk.write_opcode(opcode);
                }
            }
        }

        chunk.write_opcode(Opcode::Halt);

        println!("{:?}", chunk);

        // TODO: probably error
        if let Some(module_name) = module_name {
            // TODO: append a generation if it already exists
            self.chunks.insert(module_name.to_owned(), chunk);
        }
    }

    pub fn emit(&self) -> Result<(), CompilerError> {
        fs::create_dir_all(".jbin")?;

        for (mod_name, chunk) in self.chunks.iter() {
            // TODO: make it create subdirectories based on the submodules (game.player -> game/player.jbin)
            fs::write(format!("./.jbin/{}.jbin", mod_name), chunk.bytes())?;
        }

        Ok(())
    }
}
