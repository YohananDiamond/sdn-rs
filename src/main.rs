extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod parser;

fn main() {
    let input = r#":hello"#;
    match parser::parse(input) {
        Ok(data) => {
            println!("Data:");
            for x in data {
                println!("* {:?}", x);
            }
        }
        Err(e) => println!("Parser error:\n{}", e),
    }
}
