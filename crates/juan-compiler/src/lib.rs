use std::{collections::HashMap, fs};

use juan_ast::{Expr, FunctionDecl, Literal, NodeId, Op, ParsedModule, SyntaxTree, UnaryOp};
use juan_bytecode::{Chunk, Opcode};

use crate::error::CompilerError;

pub struct Compiler {
    chunks: HashMap<String, Chunk>,

    // TODO: this should probably be its own struct
    functions: Vec<FunctionDecl>,
    functions_lookup: HashMap<String, u32>,
}

pub mod error;

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            functions: vec![],
            functions_lookup: HashMap::new(),
        }
    }

    fn compile_expr(
        &mut self,
        id: NodeId,
        tree: &SyntaxTree,
        chunk: &mut Chunk,
        module_name: &str,
        src: &str,
    ) {
        let node = &tree[id];

        match node.expr.clone() {
            Expr::Literal(literal) => match literal {
                Literal::Int(i) => {
                    chunk.write_opcode(Opcode::PushInt);
                    chunk.write_i32(i as i32);
                }
            },

            Expr::BinaryOp { op, left, right } => {
                self.compile_expr(left, tree, chunk, module_name, src);
                self.compile_expr(right, tree, chunk, module_name, src);

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
                self.compile_expr(operand, tree, chunk, module_name, src);

                let opcode = match op {
                    UnaryOp::Neg => Opcode::Neg,
                };

                chunk.write_opcode(opcode);
            }

            Expr::Block { statements, tail } => {
                for id in statements {
                    self.compile_expr(id, tree, chunk, module_name, src);
                    chunk.write_opcode(Opcode::Pop);
                }

                if let Some(tail) = tail {
                    self.compile_expr(tail, tree, chunk, module_name, src);
                } else {
                    chunk.write_opcode(Opcode::PushUnit);
                }
            }

            Expr::Identifier => todo!("Yeah"),

            // TODO: this currently only works for same-module calls
            Expr::Call { callee } => {
                let node = &tree[callee];
                let (func_start, func_len) = node.span.unpack();

                // TODO: should not always assume callee is an identifier expression
                let func_name = &src[func_start..func_start + func_len];
                let full_name = format!("{module_name}.{func_name}");

                let func_id = self.functions_lookup[&full_name];

                chunk.write_opcode(Opcode::Call);
                chunk.write_u32(func_id);
            }
        }
    }

    pub fn read_module(&mut self, module: &ParsedModule, src: &str) {
        let (module_start, module_len) = module.decl.path_span.unpack();
        let module_name = &src[module_start..module_start + module_len];

        // TODO: improve
        // Function names
        for func in module.functions.iter() {
            let (func_start, func_len) = func.name.unpack();
            let func_name = &src[func_start..func_start + func_len];

            let full_name = format!("{}.{}", module_name, func_name);

            self.functions_lookup
                .insert(full_name, self.functions.len() as u32);

            self.functions.push(func.clone());
        }

        // Function Initialization
        for func in module.functions.iter() {
            let (func_start, func_len) = func.name.unpack();
            let func_name = &src[func_start..func_start + func_len];

            let full_name = format!("{}.{}", module_name, func_name);

            let mut chunk = Chunk::new();

            self.compile_expr(func.body, &module.tree, &mut chunk, module_name, src);
            chunk.write_opcode(Opcode::Return);

            self.chunks.insert(full_name, chunk);
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

    pub fn get_function_id(&self, name: String) -> u32 {
        self.functions_lookup[&name]
    }
}
