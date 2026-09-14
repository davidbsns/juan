use std::fs;
use std::path::Path;

use juan_lexer::Lexer;
use juan_parser::Parser;

use crate::error::JuanError;

mod error;

// TODO: convert this into a .lib with a Juan struct and then
// create like a juan-emulator main.rs that uses said Juan struct
// as this is embedded with rust and shouldnt be like this... lol

fn main() -> Result<(), JuanError> {
    let path = Path::new("./samples/math.juan");
    let input = fs::read_to_string(path).expect("Invalid input");

    let mut lexer = Lexer::new(input.as_str());
    let mut parser = Parser::new(&mut lexer);

    parser.parse()?;

    Ok(())
}
