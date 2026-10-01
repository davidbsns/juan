use std::collections::HashMap;

use juan_ast::{Expr, Literal, NodeId, Op, SyntaxTree, UnaryOp};
use juan_bytecode::{Chunk, Opcode};

use crate::{error::CompilerError, func_table::FunctionTable};

pub(crate) struct FunctionCompiler<'a> {
    chunk: Chunk,
    ast: &'a SyntaxTree,
    src: &'a str,
    module_name: &'a str,
    functions: &'a FunctionTable,
    locals: &'a HashMap<&'a str, u32>,
}

impl<'a> FunctionCompiler<'a> {
    pub fn new(
        ast: &'a SyntaxTree,
        src: &'a str,
        module_name: &'a str,
        functions: &'a FunctionTable,
        locals: &'a HashMap<&'a str, u32>,
    ) -> Self {
        Self {
            chunk: Chunk::new(),
            ast,
            src,
            module_name,
            functions,
            locals,
        }
    }

    pub fn compile_expr(&mut self, id: NodeId) -> Result<(), CompilerError> {
        let node = &self.ast[id];

        match node.expr.clone() {
            Expr::Literal(literal) => match literal {
                Literal::Int(i) => {
                    self.chunk.write_opcode(Opcode::PushInt);
                    self.chunk.write_i32(i as i32);
                }
            },

            Expr::BinaryOp { op, left, right } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;

                let opcode = match op {
                    Op::Add => Opcode::Add,
                    Op::Sub => Opcode::Sub,
                    Op::Mul => Opcode::Mul,
                    Op::Div => Opcode::Div,
                    Op::Rem => Opcode::Rem,
                };

                self.chunk.write_opcode(opcode);
            }

            Expr::UnaryOp { op, operand } => {
                self.compile_expr(operand)?;

                let opcode = match op {
                    UnaryOp::Neg => Opcode::Neg,
                };

                self.chunk.write_opcode(opcode);
            }

            Expr::Block { statements, tail } => {
                for id in statements {
                    self.compile_expr(id)?;
                    self.chunk.write_opcode(Opcode::Pop);
                }

                if let Some(tail) = tail {
                    self.compile_expr(tail)?;
                } else {
                    self.chunk.write_opcode(Opcode::PushUnit);
                }
            }

            Expr::Identifier => {
                let (name_start, name_len) = node.span.unpack();
                let ident_name = &self.src[name_start..name_start + name_len];

                // TODO: error handling
                let param_id = self.locals[ident_name];

                self.chunk.write_opcode(Opcode::LoadLocal);
                self.chunk.write_u32(param_id);
            }

            // TODO: this currently only works for same-module calls
            Expr::Call { callee, args } => {
                let func_name = self.get_func_name(callee);
                let func_id = self.functions.get_id(func_name.clone())?;

                let func = self.functions.get_from_name(func_name.clone())?;

                if args.len() != func.params.len() {
                    return Err(CompilerError::ArgumentCountMismatch(
                        func_name,
                        func.params.len() as u32,
                        args.len() as u32,
                    ));
                }

                for arg in &args {
                    self.compile_expr(*arg)?;
                }

                self.chunk.write_opcode(Opcode::Call);
                self.chunk.write_u32(func_id);
                self.chunk.write_u32(args.len() as u32);
            }
        }

        Ok(())
    }

    fn get_func_name(&self, id: NodeId) -> String {
        let node = &self.ast[id];
        let (func_start, func_len) = node.span.unpack();

        let func_name = &self.src[func_start..func_start + func_len];
        let module_name = self.module_name;

        format!("{module_name}.{func_name}")
    }

    pub fn finish(self) -> Chunk {
        self.chunk
    }
}
