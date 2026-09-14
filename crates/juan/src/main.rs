use std::fs;
use std::path::Path;

use juan_lexer::Lexer;
use juan_lexer::tokens::TokenKind;

fn main() {
    let path = Path::new("./samples/hello.juan");
    let input = fs::read_to_string(path).expect("Invalid input");

    let mut lexer = Lexer::new(input.as_str());

    while let tk = lexer.next_token()
        && tk.kind != TokenKind::Eof
    {
        println!("{:?}, {:?}", tk.kind, tk.span.unpack())
    }
}
