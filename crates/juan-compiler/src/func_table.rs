use std::collections::HashMap;

use juan_ast::FunctionDecl;

use crate::error::CompilerError;

pub(crate) struct FunctionTable {
    decls: Vec<FunctionDecl>,
    lookup: HashMap<String, u32>,
}

impl FunctionTable {
    pub fn new() -> Self {
        Self {
            decls: vec![],
            lookup: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, func: FunctionDecl) -> Result<(), CompilerError> {
        if self.lookup.contains_key(&name) {
            return Err(CompilerError::DuplicateFunctionName(name));
        }

        self.lookup.insert(name, self.decls.len() as u32);
        self.decls.push(func);

        Ok(())
    }

    pub fn get_from_name(&self, name: String) -> Result<&FunctionDecl, CompilerError> {
        let func_id = self.get_id(name.clone())?;
        self.decls
            .get(func_id as usize)
            .ok_or(CompilerError::NonExistentFunction(name))
    }

    pub fn get_id(&self, name: String) -> Result<u32, CompilerError> {
        self.lookup
            .get(&name)
            .cloned()
            .ok_or(CompilerError::NonExistentFunction(name))
    }
}
