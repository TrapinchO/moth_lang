use moth_lang::lexer;

fn main() {
    let x = String::from("1 + 1 - 2 *3 \"Hello\n World!\" 123");
    let y = lexer::lex(&x);

    println!("===== source =====");
    println!("{:?}", x);
    println!("===== lexing =====");
    for tok in y {
        println!("{:?}", tok);
    }
}
