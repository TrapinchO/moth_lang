use std::{collections::HashMap, time::Instant};

use error::Error;
use backend::interpreter::Interpreter;
use located::Location;
use backend::value::{get_builtins, NATIVE_OPERATORS};

pub mod associativity;
pub mod environment;
pub mod error;
pub mod exprstmt;
pub mod frontend;
pub mod middle;
pub mod backend;
pub mod located;
pub mod mref;
mod visitor;

#[cfg(test)]
mod tests;

pub fn run(interp: &mut Interpreter, input: String, time: bool) -> Result<(), Vec<Error>> {
    let compile_start = Instant::now();
    // the prints are commented in case I wanted to show them
    //println!("===== source =====\n{:?}\n=====        =====", input);
    let tokens = frontend::lexer::lex(&input).map_err(|e| vec![e])?;
    /*
    println!("===== lexing =====");
    for t in &tokens {
        println!("{:?}", t);
    }
    */

    let ast = frontend::parser::parse(tokens).map_err(|e| vec![e])?;
    /*
    println!("===== parsing =====");
    for s in &ast {
        println!("{:?}", s);
    }
    */

    let ast2 = frontend::reassoc::reassociate(
        NATIVE_OPERATORS
            .map(|(name, assoc, _)| (name.to_string(), assoc))
            .into(),
        ast,
    ).map_err(|e| vec![e])?;
    /*
    println!("===== reassociating =====");
    for s in &resassoc {
        println!("{}", s);
    }
    */

    let builtins = get_builtins()
        .keys()
        .map(|name| (name.clone(), (Location { start: 0, end: 0 }, false)))
        .collect::<HashMap<_, _>>();
    match middle::varcheck::varcheck(builtins, &ast2) {
        Ok(()) => {}
        Err((warns, errs)) => {
            // apparently they are in the reverse order...
            for w in warns.iter().rev() {
                println!("{}\n", w.format_message(&input));
            }
            if !errs.is_empty() {
                return Err(errs);
            }
        }
    }

    let compile_end = compile_start.elapsed();
    let eval_start = Instant::now();
    //println!("===== evaluating =====");
    interp.interpret(ast2).map_err(|e| vec![e])?;
    //interp.interpret(&resassoc)?;

    if time {
        println!("Compiled in: {:?}", compile_end);
        println!("Evaluated in: {:?}", eval_start.elapsed());
    }
    Ok(())
}
