use moth_lang::lexer;
use std::env;

fn main() {
    // courtesy of: https://stackoverflow.com/a/71731489
    env::set_var("RUST_BACKTRACE", "1");

    let x = String::from("1 + 1 - 2 *3 \"Hello World!\" 123");
    let y = lexer::lex(&x);
    match y {
        Err(err) => println!("[MOTH] {}", err),
        Ok(tokens) => {
            println!("===== source =====");
            println!("{:?}", x);
            println!("===== lexing =====");
            for tok in tokens {
                println!("{:?}", tok);
            }
        }
    }
}
