use moth_lang::lexer;
use std::env;

fn main() {
    // courtesy of: https://stackoverflow.com/a/71731489
    env::set_var("RUST_BACKTRACE", "1");

    let input = String::from("1 + 1 // - 2 *3 =\n \"Hello World!\" 123");
    let y = lexer::lex(&input);
    match y {
        Err(err) => {
            println!("[MOTH] {}", err);
            let lines = input.split('\n').collect::<Vec<_>>();
            if lines.len() < err.line {
                panic!("Error line ({}) is greater than the number of lines in the code ({})", err.line, lines.len());
            }
            let line = lines[err.line];
            println!("Error on line: {}\nPos: {}", line, err.pos);
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
