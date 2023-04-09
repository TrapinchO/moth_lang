#[cfg(test)]
mod tests {
    use moth_lang::lexer::*;

    #[test]
    fn empty() {
        assert_eq!(vec![Token{pos:0, line:1, typ:TokenType::Eof}], lex("").unwrap());
    }

    #[test]
    fn lex_number() {
        // TODO: errors
        let n = "10";
        let t = lex(n).unwrap();
        assert_eq!(t[0].typ, TokenType::Number(10))
    }

    #[test]
    fn lex_identifier() {
        let idents = [
            ("test", "test"),
            ("TeSt", "TeSt"),
            ("test123", "test123"),
        ];

        for (s, r) in idents {
            let tok = lex(s).unwrap();
            assert_eq!(tok[0].typ, TokenType::Identifier(r.to_string()));
        }
        let n = "hello";
        let t = lex(n).unwrap();
        assert_eq!(t[0].typ, TokenType::Identifier("hello".to_string()))
    }

    #[test]
    fn lex_string() {
        // TODO: errors
        let strings = [
            ("\"\"", ""),
            ("\"test\"", "test"),
            ("\"hello world\"", "hello world"),
        ];

        for (s, r) in strings {
            let tok = lex(s).unwrap();
            assert_eq!(tok[0].typ, TokenType::String(r.to_string()));
        }
    }

    #[test]
    fn lex_symbols() {
        let symbols = [
            ("+", TokenType::Plus),
            ("-", TokenType::Minus),
            ("*", TokenType::Star),
            ("/", TokenType::Slash),
        ];

        for (s, r) in symbols {
            let lexed = lex(s).unwrap();
            assert_eq!(lexed[0].typ, r)
        }
    }
}
