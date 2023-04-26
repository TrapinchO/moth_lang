use moth_lang::error::Error;
use moth_lang::lexer;
use moth_lang::parser;
use std::env;

fn main() {
    // courtesy of: https://stackoverflow.com/a/71731489
    env::set_var("RUST_BACKTRACE", "1");
    match run() {
        Ok(_) => {}
        //Err(err) => println!("{:?}", err)
        Err(err) => println!("{}", err.format_message("aaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
    }
}
fn run() -> Result<(), Error> {
    //let input = String::from("1 + 1 // - 2 *3a =\n+ \"Hello World!\" 123");
    //let input = String::from("hello /* fasd \n fsdf sd 4566 */ 1000a");
    let input = String::from("1 / 1 - 1 * (1 + 1)");
    println!("===== source =====\n{}", input);
    match lexer::lex(&input) {
        Err(err) => Err(err)?,
            /*{
            let lines = input.split('\n').map(str::to_string).collect::<Vec<_>>();
            if lines.len() < err.line {
                panic!(
                    "Error line ({}) is greater than the number of lines in the code ({})",
                    err.line,
                    lines.len()
                );
            }
            let line = &lines[err.line];
            Err(err.format_message(line))
        }
            */
        Ok(tokens) => {
            println!("===== lexing =====");
            for t in &tokens {
                println!("{:?}", t);
            }
            let ast = parser::parse(tokens)?;
            println!("===== parsing =====\n{}", ast);
            let resassoc = parser::reassoc(&ast).map_err(|msg| Error {
                msg: msg,
                line: 0,
                start: 0,
                end: 0
            })?;
            println!("===== reassociating =====\n{}", resassoc);
            Ok(())
        }
    }
}
