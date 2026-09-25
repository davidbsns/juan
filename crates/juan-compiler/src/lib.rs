use std::{collections::HashMap, fs};

use juan_ast::{Expr, Literal, NodeId, Op, ParsedModule, SyntaxTree, UnaryOp};
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

    fn compile_expr(&mut self, id: NodeId, tree: &SyntaxTree, chunk: &mut Chunk) {
        let node = &tree[id];

        match node.expr {
            Expr::Literal(literal) => match literal {
                Literal::Int(i) => {
                    chunk.write_opcode(Opcode::PushInt);
                    chunk.write_i32(i as i32);
                }
            },

            Expr::BinaryOp { op, left, right } => {
                self.compile_expr(left, tree, chunk);
                self.compile_expr(right, tree, chunk);

                let opcode = match op {
                    Op::Add => Opcode::Add,
                    Op::Sub => Opcode::Sub,
                    Op::Mul => Opcode::Mul,
                    Op::Div => Opcode::Div,
                    Op::Rem => Opcode::Rem,
                };

                chunk.write_opcode(opcode);
            }

            Expr::UnaryOp { op, operand } => {
                self.compile_expr(operand, tree, chunk);

                let opcode = match op {
                    UnaryOp::Neg => Opcode::Neg,
                };

                chunk.write_opcode(opcode);
            }
        }
    }

    pub fn read_module(&mut self, module: &ParsedModule, src: String) {
        // TODO: only create chunks inside blocks/functions

        let mut chunk = Chunk::new();

        for id in module.expr_roots.iter() {
            self.compile_expr(*id, &module.tree, &mut chunk);
        }

        chunk.write_opcode(Opcode::Halt);

        println!("{:?}", chunk);

        let (module_start, module_len) = module.decl.path_span.unpack();
        let module_name = &src[module_start..module_start + module_len];

        // TODO: append a generation if it already exists
        self.chunks.insert(module_name.to_owned(), chunk);
    }

    pub fn emit(&self) -> Result<(), CompilerError> {
        fs::create_dir_all(".jbin")?;

        for (mod_name, chunk) in self.chunks.iter() {
            // TODO: make it create subdirectories based on the submodules (game.player -> game/player.jbin)
            fs::write(format!("./.jbin/{}.jbin", mod_name), chunk.bytes())?;
        }

        Ok(())
    }

    pub fn get_chunk(&self, name: String) -> Option<&Chunk> {
        self.chunks.get(&name)
    }
}
