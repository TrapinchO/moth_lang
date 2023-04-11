#[cfg(test)]
mod tests {
    use moth_lang::lexer::*;

    #[test]
    fn empty() {
        assert_eq!(vec![Token{pos:0, line:1, typ:TokenType::Eof}], lex("").unwrap());
    }

    #[test]
    fn lex_number() {
        let n = "10";
        let t = lex(n).unwrap();
        assert_eq!(t[0].typ, TokenType::Number(10))
    }

    #[test]
    fn lex_number_err() {
        let nums = [
            ("1a", "Invalid digit: \"a\""),
        ];
        for (n, r) in nums {
            let tok = lex(n);
            assert_eq!(Err(r.to_string()), tok);
        }
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
    fn lex_keyword() {
        let kw = [
            ("let", TokenType::Let),
            ("fun", TokenType::Fun),
        ];
        for (k, r) in kw {
            let tok = lex(k).unwrap();
            assert_eq!(tok[0].typ, r);
        }
    }


    #[test]
    fn lex_string() {
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
    fn lex_string_err() {
        let strings = [
            ("\"", "EOF while parsing string"),
            ("\"test\n\"", "EOL while parsing string"),
        ];
        for (s, e) in strings {
            let tok = lex(s);
            assert_eq!(Err(e.to_string()), tok);
        }
    }

    #[test]
    fn lex_symbols() {
        let symbols = ["+", "-", "*", "/", "<", ">", "!"];

        for s in symbols {
            let lexed = lex(s).unwrap();
            assert_eq!(lexed[0].typ, TokenType::Symbol(s.to_string()));
        }
    }

    #[test]
    fn lex_symbols_special() {
        let symbols = ["="];

        for s in symbols {
            let lexed = lex(s).unwrap();
            assert_eq!(lexed[0].typ, TokenType::Equals);
        }
    }

    #[test]
    fn lex_comment() {
        // TODO: DEAL WITH TESTING
        let coms = [
            ("//", vec![TokenType::Eof]),
            ("//test", vec![TokenType::Eof]),
            ("///test", vec![TokenType::Eof]),
            ("//-test", vec![TokenType::Symbol("//-".to_string()),
                             TokenType::Identifier("test".to_string()),
                             TokenType::Eof]),
            ("// test", vec![TokenType::Eof]),
            ("/// test", vec![TokenType::Eof]),
            ("// -test", vec![TokenType::Eof]),
            ("// test \ntest", vec![TokenType::Identifier("test".to_string()),
                                    TokenType::Eof]),
            ("/// test\ntest", vec![TokenType::Identifier("test".to_string()),
                                    TokenType::Eof]),
        ];

        for (c, r) in coms {
            let lexed = lex(c).unwrap()
                .iter()
                .map(|t| t.typ.clone())
                .collect::<Vec<_>>();
            assert_eq!(lexed, r);
        }
    }
}
