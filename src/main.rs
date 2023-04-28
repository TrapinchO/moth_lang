use moth_lang::error::Error;
use moth_lang::lexer;
use moth_lang::parser;
use std::env;

fn main() {
    // courtesy of: https://stackoverflow.com/a/71731489
    env::set_var("RUST_BACKTRACE", "1");


    //let input = String::from("1 + 1 // - 2 *3a =\n+ \"Hello World!\" 123");
    //let input = String::from("hello /* fasd \n fsdf sd 4566 */ 1000a");
    let input = String::from("(1 * 1 + 1) * 1 + 1");
    match run(input.clone()) {
        Ok(_) => {}
        Err(err) => {
            let lines = input.split('\n').map(str::to_string).collect::<Vec<_>>();
            if lines.len() < err.line {
                panic!(
                    "Error line ({}) is greater than the number of lines in the code ({})",
                    err.line,
                    lines.len()
                );
            }
            let line = &lines[err.line];
            println!("{}", err.format_message(line));
        }
    }
}
fn run(input: String) -> Result<(), Error> {
    println!("===== source =====\n{}\n=====        =====", input);
    let tokens = lexer::lex(&input)?;
    println!("===== lexing =====");
    for t in &tokens {
        println!("{:?}", t);
    }
    let ast = parser::parse(tokens)?;
    println!("===== parsing =====\n{}", ast);
    //let resassoc = parser::reassoc(&ast)?;
    //println!("===== reassociating =====\n{}", resassoc);
    Ok(())
}
