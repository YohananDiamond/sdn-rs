extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod parser;

use parser::ParserResult;

fn main() {
    let input = r#"("Hello, World!") (:ab c)"#;
    match parser::parse(input) {
        ParserResult::Success(data) => {
            for x in data {
                println!("{:?}", x);
            }
        }
        ParserResult::PestError(e) => println!("Lexer error:\n{}", e),
        ParserResult::StringError(e) => println!("Parser error: {}", e),
    }
}
