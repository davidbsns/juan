use std::fs;
use std::path::Path;

use juan_compiler::Compiler;
use juan_lexer::Lexer;
use juan_parser::Parser;
use juan_vm::VM;

use crate::error::JuanError;

mod error;

fn main() -> Result<(), JuanError> {
    let path = Path::new("./samples/math.juan");
    let input = fs::read_to_string(path).expect("Invalid input");

    let mut vm = VM::new();
    let mut lexer = Lexer::new(input.as_str());
    let module = Parser::new(&mut lexer).parse()?;
    let mut compiler = Compiler::new();

    compiler.read_module(&module, input);
    compiler.emit()?;

    let math = String::from("math");
    let chunk = compiler.get_chunk(math.clone()).unwrap();

    vm.load(math.clone(), chunk.clone())?;
    vm.run(math)?;

    Ok(())
}
