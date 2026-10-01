use std::{collections::HashMap, fs};

use juan_ast::ParsedModule;
use juan_bytecode::{Chunk, Opcode};

use crate::{error::CompilerError, func_compiler::FunctionCompiler, func_table::FunctionTable};

pub mod error;
mod func_compiler;
mod func_table;

pub struct Compiler {
    chunks: HashMap<String, Chunk>,
    functions: FunctionTable,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            functions: FunctionTable::new(),
        }
    }

    pub fn read_module(&mut self, module: &ParsedModule, src: &str) -> Result<(), CompilerError> {
        let (module_start, module_len) = module.decl.path_span.unpack();
        let module_name = &src[module_start..module_start + module_len];

        // TODO: improve
        // Function names
        for func in module.functions.iter() {
            let (func_start, func_len) = func.name.unpack();
            let func_name = &src[func_start..func_start + func_len];

            let full_name = format!("{}.{}", module_name, func_name);
            self.functions.add(full_name, func.clone())?;
        }

        // Function Initialization
        for func in module.functions.iter() {
            let (func_start, func_len) = func.name.unpack();
            let func_name = &src[func_start..func_start + func_len];

            let full_name = format!("{}.{}", module_name, func_name);

            let mut params: HashMap<&str, u32> = HashMap::new();

            for param in func.params.iter() {
                let (name_start, name_len) = param.name.unpack();
                let param_name = &src[name_start..name_start + name_len];

                if params.contains_key(param_name) {
                    return Err(CompilerError::DuplicateParameterName(param_name.to_owned()));
                }

                params.insert(param_name, params.len() as u32);
            }

            let mut function_compiler =
                FunctionCompiler::new(&module.tree, src, module_name, &self.functions, &params);
            function_compiler.compile_expr(func.body)?;

            let mut chunk = function_compiler.finish();

            chunk.write_opcode(Opcode::Return);

            self.chunks.insert(full_name, chunk);
        }

        Ok(())
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

    pub fn get_function_id(&self, name: String) -> Result<u32, CompilerError> {
        self.functions.get_id(name)
    }
}
