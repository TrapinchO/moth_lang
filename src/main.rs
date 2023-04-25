use moth_lang::lexer;
use moth_lang::parser;
use moth_lang::parser::reassoc;
use std::env;

fn main() {
    // courtesy of: https://stackoverflow.com/a/71731489
    env::set_var("RUST_BACKTRACE", "1");

    //let input = String::from("1 + 1 // - 2 *3a =\n+ \"Hello World!\" 123");
    //let input = String::from("hello /* fasd \n fsdf sd 4566 */ 1000a");
    let input = String::from("1 + 1 - 1 * 2 / 2");
    let y = lexer::lex(&input);
    match y {
        Err(err) => {
            let lines = input.split('\n').map(str::to_string).collect::<Vec<_>>();
            if lines.len() < err.line {
                panic!(
                    "Error line ({}) is greater than the number of lines in the code ({})",
                    err.line,
                    lines.len()
                );
            }
            let line = &lines[err.line];
            println!("{}", err.format_message(line));
            return;
        }
        Ok(tokens) => {
            println!("===== source =====");
            println!("{:?}", input);
            println!("===== lexing =====");
            for tok in &tokens {
                println!("{:?}", tok);
            }

            let ast = parser::parse(tokens);
            match ast {
                Err(err) => println!("[MOTH] Error: {}", err),
                Ok(expr) => {
                    println!("{:?}", expr);
                    let ast = parser::Expr::BinaryOperation(
                            Box::new(parser::Expr::Number(1)),
                            "*".to_string(),
                            Box::new(parser::Expr::BinaryOperation(
                                    Box::new(parser::Expr::Number(2)),
                                    "+".to_string(),
                                    Box::new(parser::Expr::Number(3)))
                            )
                    );
                    println!("### {:?}", ast);
                    println!("{:?}", reassoc(ast));
                }
            }
        }
    }

}
