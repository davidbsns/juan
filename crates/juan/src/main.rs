use std::fs;
use std::path::Path;

use juan_compiler::Compiler;
use juan_lexer::Lexer;
use juan_parser::Parser;
use juan_vm::VM;

use crate::error::JuanError;

mod error;

fn run() -> Result<(), JuanError> {
    let path = Path::new("./samples/math.juan");
    let input = fs::read_to_string(path).expect("Invalid input");

    let mut vm = VM::new();
    let mut lexer = Lexer::new(input.as_str());
    let module = Parser::new(&mut lexer).parse()?;
    let mut compiler = Compiler::new();

    compiler.read_module(&module, input.as_str())?;
    compiler.emit()?;

    let math = String::from("math.calculate");
    let get_num = String::from("math.get_num");

    let chunk = compiler.get_chunk(get_num.clone()).unwrap();
    let func_id = compiler.get_function_id(get_num.clone())?;

    vm.load(get_num.clone(), func_id, chunk.clone())?;

    let chunk = compiler.get_chunk(math.clone()).unwrap();
    let func_id = compiler.get_function_id(math.clone())?;

    vm.load(math.clone(), func_id, chunk.clone())?;

    let value = vm.run(func_id)?;
    println!("{:?}", value);

    Ok(())
}

fn main() {
    let result = run();

    match result {
        Ok(_) => (),
        Err(err) => eprintln!("{err}"),
    }
}
