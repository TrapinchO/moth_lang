use std::{collections::HashMap, time::Instant};

use backend::interpreter::Interpreter;
use backend::value::{get_builtins, NATIVE_OPERATORS};
use error::Error;
use located::Location;

pub mod associativity;
pub mod backend;
pub mod environment;
pub mod error;
pub mod exprstmt;
pub mod frontend;
pub mod located;
pub mod middle;
pub mod mref;
mod visitor;

#[cfg(test)]
mod tests;

pub fn run(interp: &mut Interpreter, input: &str, time: bool) -> Result<(), Vec<Error>> {
    let compile_start = Instant::now();
    // the prints are commented in case I wanted to show them
    //eprintln!("===== source =====\n{:?}\n=====        =====", input);
    let tokens = frontend::lexer::lex(input)?;
    /*
    eprintln!("===== lexing =====");
    for t in &tokens {
        eprintln!("{:?}", t);
    }
    */

    let ast = frontend::parser::parse(tokens)?;
    /*
    eprintln!("===== parsing =====");
    for s in &ast {
        eprintln!("{:?}", s);
    }
    */

    let ast2 = frontend::reassoc::reassociate(
        NATIVE_OPERATORS
            .map(|(name, assoc, _)| (name.to_string(), assoc))
            .into(),
        ast,
    )
    .map_err(|e| vec![e])?;
    /*
    eprintln!("===== reassociating =====");
    for s in &resassoc {
        eprintln!("{}", s);
    }
    */

    //eprintln!("===== varchecking =====");
    let builtins = get_builtins()
        .keys()
        .map(|name| (name.clone(), (Location { start: 0, end: 0 }, false)))
        .collect::<HashMap<_, _>>();
    match middle::varcheck::varcheck(builtins, &ast2) {
        Ok(()) => {}
        Err((warns, errs)) => {
            // apparently they are in the reverse order...
            for w in warns.iter().rev() {
                eprintln!("{}\n", w.format_message(input));
            }
            if !errs.is_empty() {
                return Err(errs);
            }
        }
    }

    let simple_ast = backend::simplify::simplify(ast2).map_err(|e| vec![e])?;
    //eprintln!("===== simplifying =====");
    /*
    for s in &simple_ast {
        eprintln!("{}", s);
    }
    */

    let compile_end = compile_start.elapsed();
    let eval_start = Instant::now();
    //println!("===== evaluating =====");
    interp.interpret(simple_ast).map_err(|e| vec![e])?;
    let eval_end = eval_start.elapsed();

    if time {
        eprintln!("Compiled in: {compile_end:?}");
        eprintln!("Evaluated in: {eval_end:?}");
    }
    Ok(())
}
