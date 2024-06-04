use moth_lang::backend::{interpreter::Interpreter, value::get_builtins};
use moth_lang::run;

use std::{
    env, fs,
    io::{self, Write},
};

fn main() {
    // courtesy of: https://stackoverflow.com/a/71731489
    // provides a backtrace in case of error
    env::set_var("RUST_BACKTRACE", "1");

    let args = env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        let file_name = &args[1];
        let Ok(src) = fs::read_to_string(file_name) else {
            println!("File \"{file_name}\" not found.");
            return;
        };
        // TODO: windows newlines have \r which messes up the lexer
        let src = src.trim_end().replace('\r', "");

        let mut interp = Interpreter::new(get_builtins());
        match run(&mut interp, &src, true) {
            Ok(()) => {}
            Err(errs) => {
                for e in errs {
                    println!("{}\n", e.format_message(&src));
                }
            }
        }
    } else {
        println!("Unknown amount of arguments: {}", args.len());
    }
}

// TODO: declared things are not preserved between runs
// caused by varcheck
fn repl() {
    let mut interp = Interpreter::new(get_builtins());
    loop {
        print!(">>> ");
        io::stdout().flush().unwrap(); // and  hope it never fails
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        match run(&mut interp, &input, false) {
            Ok(()) => {}
            Err(errs) => {
                for e in errs {
                    println!("{}", e.format_message(&input));
                }
            }
        }
    }
}
