use moth_lang::{
    error::Error,
    interpreter::Interpreter,
    lexer, parser, reassoc,
    value::{get_builtins, NATIVE_OPERATORS},
    varcheck,
};

use std::{
    env, fs,
    io::{self, Write},
    time::Instant,
};

fn main() {
    // courtesy of: https://stackoverflow.com/a/71731489
    // provides a backtace in case of error
    env::set_var("RUST_BACKTRACE", "1");

    let args = env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        let file_name = &args[1];
        let Ok(src) = fs::read_to_string(file_name) else {
            println!("File \"{}\" not found.", file_name);
            return;
        };
        let src = src.trim_end().replace('\r', ""); // TODO: windows newlines have \r which messes up the lexer

        let mut interp = Interpreter::new(get_builtins());
        match run(&mut interp, src.to_string()) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.format_message(&src));
            }
        }
    } else {
        println!("Unknown amount of arguments: {}", args.len());
    }
}

fn repl() {
    let mut interp = Interpreter::new(get_builtins());
    loop {
        print!(">>> ");
        std::io::stdout().flush().unwrap(); // and  hope it never fails
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_string();

        match run(&mut interp, input.clone()) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.format_message(&input.to_string()));
            }
        }
    }
}

fn run(interp: &mut Interpreter, input: String) -> Result<(), Error> {
    let compile_start = Instant::now();
    // the prints are commented in case I wanted to show them
    //println!("===== source =====\n{:?}\n=====        =====", input);
    let tokens = lexer::lex(&input)?;
    /*
    println!("===== lexing =====");
    for t in &tokens {
        println!("{:?}", t);
    }
    */

    // TODO: unknown operator is not reported unless reassociated in binary operation
    let ast = parser::parse(tokens)?;
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
    )?;
    /*
    println!("===== reassociating =====");
    for s in &resassoc {
        println!("{}", s);
    }
    */

    // TODO: change back to reference, less cloning
    match varcheck::varcheck(get_builtins(), resassoc.clone()) {
        Ok(()) => {},
        Err(errs) => {
            for e in errs {
                println!("{}", e.format_message(&input));
            }
            return Ok(())
        }
    }

    let compile_end = compile_start.elapsed();
    let eval_time = Instant::now();
    //println!("===== evaluating =====");
    interp.interpret(resassoc)?;
    //interp.interpret(&resassoc)?;

    println!("Compiled in: {:?}", compile_end);
    println!("Evaluated in: {:?}", eval_time.elapsed());

    Ok(())
}
