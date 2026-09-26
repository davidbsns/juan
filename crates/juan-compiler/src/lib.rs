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

        match node.expr.clone() {
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

            Expr::Block { statements, tail } => {
                for id in statements {
                    self.compile_expr(id, tree, chunk);
                    chunk.write_opcode(Opcode::Pop);
                }

                if let Some(tail) = tail {
                    self.compile_expr(tail, tree, chunk);
                } else {
                    chunk.write_opcode(Opcode::PushUnit);
                }
            }
        }
    }

    pub fn read_module(&mut self, module: &ParsedModule, src: String) {
        // TODO: only create chunks inside blocks/functions
        //

        let (module_start, module_len) = module.decl.path_span.unpack();
        let module_name = &src[module_start..module_start + module_len];

        for func in module.functions.iter() {
            let (func_start, func_len) = func.name.unpack();
            let func_name = &src[func_start..func_start + func_len];

            let mut chunk = Chunk::new();

            self.compile_expr(func.body, &module.tree, &mut chunk);
            chunk.write_opcode(Opcode::Halt);

            println!("{:?}", chunk);

            self.chunks.insert(
                format!("{}.{}", module_name.to_owned(), func_name.to_owned()),
                chunk,
            );
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

    pub fn get_chunk(&self, name: String) -> Option<&Chunk> {
        self.chunks.get(&name)
    }
}
