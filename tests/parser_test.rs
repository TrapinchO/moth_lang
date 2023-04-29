#[cfg(test)]
mod tests {
    use moth_lang::lexer::{Token, TokenType, lex};
    use moth_lang::parser::*;

    #[test]
    #[should_panic(expected = "Expected code to parse")]
    fn empty() {
        parse(vec![Token {start: 0, end: 0, line: 0, typ: TokenType::Eof }]).unwrap();
    }

    /*
    #[test]
    fn parse_primary() {
        assert!(matches!(
            parse(lex("1+1").unwrap()).unwrap().typ,
            ExprType::BinaryOperation(Expr {typ: ExprType::Number(1), ..}, Token { typ: TokenType::Symbol("+"), .. }, Expr {typ: ExprType::Number(1), ..}),
        ))
    }
    */
}
