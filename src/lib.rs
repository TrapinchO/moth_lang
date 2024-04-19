use std::{collections::HashMap, time::Instant};

use error::Error;
use interpreter::Interpreter;
use located::Location;
use value::{get_builtins, NATIVE_OPERATORS};

pub mod environment;
pub mod error;
pub mod exprstmt;
pub mod interpreter;
pub mod lexer;
pub mod located;
pub mod parser;
pub mod reassoc;
pub mod token;
pub mod value;
pub mod varcheck;
mod visitor;

#[cfg(test)]
mod tests;

pub fn run(interp: &mut Interpreter, input: String, time: bool) -> Result<(), Vec<Error>> {
    let compile_start = Instant::now();
    // the prints are commented in case I wanted to show them
    //println!("===== source =====\n{:?}\n=====        =====", input);
    let tokens = lexer::lex(&input).map_err(|e| vec![e])?;
    /*
    println!("===== lexing =====");
    for t in &tokens {
        println!("{:?}", t);
    }
    */

    // TODO: unknown operator is not reported unless reassociated in binary operation
    let ast = parser::parse(tokens).map_err(|e| vec![e])?;
    /*
    println!("===== parsing =====");
    for s in &ast {
        println!("{:?}", s);
    }
    */

    let resassoc = reassoc::reassociate(
        NATIVE_OPERATORS
            .map(|(name, assoc, _)| (name.to_string(), assoc))
            .into(),
        ast,
    )
    .map_err(|e| vec![e])?;
    /*
    println!("===== reassociating =====");
    for s in &resassoc {
        println!("{}", s);
    }
    */

    // TODO: change back to reference, less cloning
    let builtins = get_builtins()
        .keys()
        .map(|name| (name.clone(), (Location { start: 0, end: 0 }, false)))
        .collect::<HashMap<_, _>>();
    match varcheck::varcheck(builtins, &resassoc) {
        Ok(()) => {}
        Err((warns, errs)) => {
            for w in warns {
                println!("{}", w.format_message(&input));
            }
            let has_errors = !errs.is_empty();
            if has_errors {
                return Err(errs);
            }
        }
    }

    let compile_end = compile_start.elapsed();
    let eval_time = Instant::now();
    //println!("===== evaluating =====");
    interp.interpret(resassoc).map_err(|e| vec![e])?;
    //interp.interpret(&resassoc)?;

    if time {
        println!("Compiled in: {:?}", compile_end);
        println!("Evaluated in: {:?}", eval_time.elapsed());
    }
    Ok(())
}
