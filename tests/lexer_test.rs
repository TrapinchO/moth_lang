#[cfg(test)]
mod tests {
    use moth_lang::lexer::*;

    #[test]
    fn lex_number() {
        let n = "10";
        let t = lex(n);
        assert_eq!(t[0].typ, TokenType::Number(10))
    }

    #[test]
    fn lex_identifier() {
        let n = "hello";
        let t = lex(n);
        assert_eq!(t[0].typ, TokenType::Identifier("hello".to_string()))
    }

    #[test]
    fn lex_symbols() {
        let symbols = [
            ("+", TokenType::Plus),
            ("-", TokenType::Minus),
            ("*", TokenType::Star),
            ("/", TokenType::Slash),
        ];

        for (s, t) in symbols {
            let lexed = lex(s);
            assert_eq!(
                lexed[0],
                Token {
                    pos: 1,
                    line: 1,
                    typ: t
                }
            )
        }
    }
}
