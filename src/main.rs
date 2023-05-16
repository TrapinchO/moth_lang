use moth_lang::error::Error;
use moth_lang::interpreter;
use moth_lang::lexer;
use moth_lang::parser;
use moth_lang::reassoc;
use std::env;
use std::io;

fn main() {
    // courtesy of: https://stackoverflow.com/a/71731489
    env::set_var("RUST_BACKTRACE", "1");

    //let input = String::from("1 + 1 // - 2 *3a =\n+ \"Hello World!\" 123");
    //let input = String::from("hello /* fasd \n fsdf sd 4566 */ 1000a");
    //let input = String::from("(1 * 1 + 1) * 1 + 1");
    loop {
        println!("=================\n===== input =====");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        //let input = "1\n+\n\n\n\n\n\n\n\n\n\n\n1-1; // and this is how ya do it".to_string();
        //let input = "let 10 = 10;".to_string();
        //let input = "let x = 10-1--1;".to_string();
        if input.is_empty() {
            println!("Empty code, exiting program");
            return;
        }
        match run(input.clone()) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.format_message(&input.to_string()));
            }
        }
    }
}
fn run(input: String) -> Result<(), Error> {
    println!("===== source =====\n{:?}\n=====        =====", input);
    let tokens = lexer::lex(&input)?;

    println!("===== lexing =====");
    /*
    for t in &tokens {
        println!("{:?}", t);
    }
    */

    // TODO: unknown operator does not report unless reassociated in binary operation
    let ast = parser::parse(tokens)?;
    /*
    println!("===== parsing =====");
    for s in &ast {
        println!("{}", s);
    }
    */

    let resassoc = reassoc::reassociate(&ast)?;
    println!("===== reassociating =====");
    ///*
    for s in &resassoc {
        println!("{}", s);
    }
    //*/

    println!("===== evaluating =====");
    interpreter::interpret(&resassoc)?;

    Ok(())
}
