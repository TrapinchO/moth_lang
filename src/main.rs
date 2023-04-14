use moth_lang::lexer;
use std::env;

fn main() {
    // courtesy of: https://stackoverflow.com/a/71731489
    env::set_var("RUST_BACKTRACE", "1");

    let input = String::from("1 + 1 // - 2 *3 =\n \"Hello World!\" 123");
    let y = lexer::lex(&input);
    match y {
        Err(err) => {
            let lines = input.split('\n').map(str::to_string).collect::<Vec<_>>();
            if lines.len() < err.line as usize {
                panic!("Error line ({}) is greater than the number of lines in the code ({})", err.line, lines.len());
            }
            let line = &lines[(err.line-1) as usize];
            println!("Error on line {}:\n{}\n{}{}^ {}", err.line, line,
                     " ".repeat(err.start), "-".repeat(err.pos-err.start),
                     err.msg
            );
        },
        Ok(tokens) => {
            println!("===== source =====");
            println!("{:?}", input);
            println!("===== lexing =====");
            for tok in tokens {
                println!("{:?}", tok);
            }
        }
    }
}
