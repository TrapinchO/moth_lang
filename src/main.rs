use moth_lang::error::Error;
use moth_lang::interpreter::Interpreter;
use moth_lang::lexer;
use moth_lang::parser;
use moth_lang::reassoc;
use moth_lang::value::BUILTINS;
use moth_lang::value::ValueType;
use std::collections::HashMap;
use std::env;
use std::io;

fn main() {
    // courtesy of: https://stackoverflow.com/a/71731489
    env::set_var("RUST_BACKTRACE", "1");

    let mut interp = Interpreter::new(HashMap::from(
        BUILTINS.map(|(name, _, f)| (name.to_string(), ValueType::Function(f)))
    ));
    loop {
        println!("=================\n===== input =====");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        if input.is_empty() {
            println!("Empty code, exiting program");
            return;
        }
        match run(&mut interp, input.clone()) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.format_message(&input.to_string()));
            }
        }
    }
}

fn run(interp: &mut Interpreter, input: String) -> Result<(), Error> {
    // this are commented in case I wanted to show them
    //println!("===== source =====\n{:?}\n=====        =====", input);
    let tokens = lexer::lex(&input)?;

    /*
    println!("===== lexing =====");
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

    let resassoc = reassoc::reassociate(BUILTINS.map(|(name, assoc, _)| (name.to_string(), assoc)).into(), &ast)?;
    println!("===== reassociating =====");
    // /*
    for s in &resassoc {
        println!("{}", s);
    }
    //*/

    println!("===== evaluating =====");
    interp.interpret(&resassoc)?;

    Ok(())
}
