#[cfg(test)]
mod tests {
    use moth_lang::error::*;
    use moth_lang::lexer::lex;
    use moth_lang::token::*;

    #[test]
    fn empty() {
        assert_eq!(vec![Token{start:0, end:0, typ:TokenType::Eof}], lex("").unwrap());
    }

    #[test]
    fn lex_number() {
        let n = "10";
        let t = lex(n).unwrap();
        assert_eq!(t[0].typ, TokenType::Int(10))
    }

    #[test]
    fn lex_float() {
        let nums = [
            ("1.1", 1.1),
            ("10.1", 10.1),
        ];

        for (f, r) in nums {
            let tok = lex(f).unwrap();
            assert_eq!(tok[0].typ, TokenType::Float(r));
        }
    }

    #[test]
    fn lex_float_error() {
        assert_eq!(
            lex("1.1.1"),
            Err(Error {
                msg: "Found two floating point number delimiters".to_string(),
                lines: vec![(0, 4)]
            })
        );
    }

    #[test]
    fn lex_number_err() {
        let nums = [
            ("1a", Error { msg: "Invalid digit: \"a\"".to_string(), lines: vec![(0, 1)]})
        ];
        for (n, r) in nums {
            let tok = lex(n);
            assert_eq!(Err(r), tok);
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
        let n = "hello++";
        let t = lex(n).unwrap();
        assert_eq!(t[0].typ, TokenType::Identifier("hello".to_string()));
        assert_eq!(t[1].typ, TokenType::Symbol("++".to_string()));
    }

    #[test]
    fn lex_keyword() {
        let kw = [
            ("let", TokenType::Let),
            ("fun", TokenType::Fun),
            ("true", TokenType::True),
            ("false", TokenType::False),
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
            ("\"test\" test", "test"),
        ];

        for (s, r) in strings {
            let tok = lex(s).unwrap();
            assert_eq!(tok[0].typ, TokenType::String(r.to_string()));
        }

        let tok2 = lex("\"test\"++++").unwrap();
        assert_eq!(tok2[1].typ, TokenType::Symbol("++++".to_string()));
    }

    #[test]
    fn lex_string_err() {
        let strings = [
            ("\"", Error { msg: "EOF while parsing string".to_string(), lines: vec![(0, 1)]}),
            ("\"test\n\"", Error { msg: "EOL while parsing string".to_string(), lines: vec![(0, 5)]}),
        ];
        for (s, e) in strings {
            let tok = lex(s);
            assert_eq!(Err(e), tok);
        }
    }

    #[test]
    fn lex_symbols() {
        let symbols = [
            "+", "-", "*", "/", "==", "<", ">", "!", "|", "..", "$", "&", "@", "#", "??", "~", "^", ":", "%",
        ];

        for s in symbols {
            let lexed = lex(s).unwrap();
            assert_eq!(lexed[0].typ, TokenType::Symbol(s.to_string()));
        }
    }

    #[test]
    fn lex_symbols_special() {
        let symbols = [
            ("=", TokenType::Equals),
            ("?", TokenType::QuestionMark),
            (".", TokenType::Dot),
            (";", TokenType::Semicolon),
            ("(", TokenType::LParen),
            (")", TokenType::RParen),
            ("[", TokenType::LBracket),
            ("]", TokenType::RBracket),
            ("{", TokenType::LBrace),
            ("}", TokenType::RBrace),
        ];

        for (s, r) in symbols {
            let lexed = lex(s).unwrap();
            assert_eq!(lexed[0].typ, r);
        }
    }

    #[test]
    fn lex_symbols_special2() {
        let symbols = [
            ("=1", vec![TokenType::Equals, TokenType::Int(1), TokenType::Eof]),
            ("?1", vec![TokenType::QuestionMark, TokenType::Int(1), TokenType::Eof]),
            (";1", vec![TokenType::Semicolon, TokenType::Int(1), TokenType::Eof]),
            ("(1", vec![TokenType::LParen, TokenType::Int(1), TokenType::Eof]),
            (")1", vec![TokenType::RParen, TokenType::Int(1), TokenType::Eof]),
            ("[1", vec![TokenType::LBracket, TokenType::Int(1), TokenType::Eof]),
            ("]1", vec![TokenType::RBracket, TokenType::Int(1), TokenType::Eof]),
            ("{1", vec![TokenType::LBrace, TokenType::Int(1), TokenType::Eof]),
            ("}1", vec![TokenType::RBrace, TokenType::Int(1), TokenType::Eof]),

            ("= 1", vec![TokenType::Equals, TokenType::Int(1), TokenType::Eof]),
            ("? 1", vec![TokenType::QuestionMark, TokenType::Int(1), TokenType::Eof]),
            ("; 1", vec![TokenType::Semicolon, TokenType::Int(1), TokenType::Eof]),
            ("( 1", vec![TokenType::LParen, TokenType::Int(1), TokenType::Eof]),
            (") 1", vec![TokenType::RParen, TokenType::Int(1), TokenType::Eof]),
            ("[ 1", vec![TokenType::LBracket, TokenType::Int(1), TokenType::Eof]),
            ("] 1", vec![TokenType::RBracket, TokenType::Int(1), TokenType::Eof]),
            ("{ 1", vec![TokenType::LBrace, TokenType::Int(1), TokenType::Eof]),
            ("} 1", vec![TokenType::RBrace, TokenType::Int(1), TokenType::Eof]),
        ];

        for (s, r) in symbols {
            let lexed = lex(s)
                .unwrap()
                .iter()
                .map(|t| t.typ.clone())
                .collect::<Vec<_>>();
            assert_eq!(lexed, r);
        }
    }

    #[test]
    fn lex_line_comment() {
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
            let lexed = lex(c)
                .unwrap()
                .iter()
                .map(|t| t.typ.clone())
                .collect::<Vec<_>>();
            assert_eq!(lexed, r);
        }
    }

    #[test]
    fn lex_block_comment() {
        let comms = [
            ("/**/", vec![TokenType::Symbol("/**/".to_string()),
                          TokenType::Eof]),
            ("/* */", vec![TokenType::Eof]),
            ("/* \n */", vec![TokenType::Eof]),
            ("/* test */", vec![TokenType::Eof]),
            ("test /* test */", vec![TokenType::Identifier("test".to_string()),
                                     TokenType::Eof]),
            ("/* test */ test", vec![TokenType::Identifier("test".to_string()),
                                     TokenType::Eof]),
            ("test /* test */ test", vec![TokenType::Identifier("test".to_string()),
                                          TokenType::Identifier("test".to_string()),
                                          TokenType::Eof]),
        ];

        for (c, r) in comms {
            let lexed = lex(c)
                .unwrap()
                .iter()
                .map(|t| t.typ.clone())
                .collect::<Vec<_>>();
            assert_eq!(lexed, r);
        }
    }

    #[test]
    fn lex_example() {
        // tests positions and whether lexer advances properly
        let exprs = [
            ("1 + 12", vec![
                Token { start: 0, end:0, typ: TokenType::Int(1) },
                Token { start: 2, end:2, typ: TokenType::Symbol("+".to_string()) },
                Token { start: 4, end:5, typ: TokenType::Int(12) },
                Token { start: 6, end:6, typ: TokenType::Eof }]),

            ("1+12", vec![
                Token { start: 0, end:0, typ: TokenType::Int(1) },
                Token { start: 1, end:1, typ: TokenType::Symbol("+".to_string()) },
                Token { start: 2, end:3, typ: TokenType::Int(12) },
                Token { start: 4, end:4, typ: TokenType::Eof }]),
            
            ("test2+test", vec![
                Token { start: 0, end:4, typ: TokenType::Identifier("test2".to_string()) },
                Token { start: 5, end:5, typ: TokenType::Symbol("+".to_string()) },
                Token { start: 6, end:9, typ: TokenType::Identifier("test".to_string()) },
                Token { start: 10, end:10, typ: TokenType::Eof }]),
            
            ("\"test\"+\"test\"", vec![
                Token { start: 0, end:5, typ: TokenType::String("test".to_string()) },
                Token { start: 6, end:6, typ: TokenType::Symbol("+".to_string()) },
                Token { start: 7, end:12, typ: TokenType::String("test".to_string()) },
                Token { start: 13, end:13, typ: TokenType::Eof }]),
            
            ("\"test\"\n+\"test\"", vec![
                Token { start: 0, end:5, typ: TokenType::String("test".to_string()) },
                Token { start: 7, end:7, typ: TokenType::Symbol("+".to_string()) },
                Token { start: 8, end:13, typ: TokenType::String("test".to_string()) },
                Token { start: 14, end:14, typ: TokenType::Eof }]),

            ("\"test\" /* test */ \"test\"", vec![
                Token { start: 0, end:5, typ: TokenType::String("test".to_string()) },
                Token { start: 18, end:23, typ: TokenType::String("test".to_string()) },
                Token { start: 24, end:24, typ: TokenType::Eof }]),

            ("\"test\" /* test \ntest */ \"test\"", vec![
                Token { start: 0, end:5, typ: TokenType::String("test".to_string()) },
                Token { start: 24, end:29, typ: TokenType::String("test".to_string()) },
                Token { start: 30, end:30, typ: TokenType::Eof }]),
        ];

        for (ex, tok) in exprs {
            let lexed = lex(ex).unwrap();
            assert_eq!(tok, lexed, "\n{:?}", ex);
        }
    }
}
