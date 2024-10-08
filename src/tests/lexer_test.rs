use crate::{
    error::{Error, ErrorType},
    frontend::lexer::lex,
    frontend::token::*,
    located::Location,
};

#[test]
fn lex_empty() {
    assert_eq!(
        vec![Token {
            loc: Location { start: 0, end: 0 },
            val: TokenType::Eof
        }],
        lex("").unwrap()
    );
}

#[test]
fn lex_number() {
    let n = "10";
    let t = lex(n).unwrap();
    assert_eq!(t[0].val, TokenType::Int(10))
}

#[test]
fn lex_float() {
    let nums = [("1.1", 1.1), ("10.1", 10.1)];

    for (f, r) in nums {
        let tok = lex(f).unwrap();
        assert_eq!(tok[0].val, TokenType::Float(r));
    }
}

#[test]
fn lex_float_error() {
    assert_eq!(
        lex("1.1.1"),
        Err(vec![Error {
            msg: ErrorType::TwoDecimalPoints,
            lines: vec![Location { start: 0, end: 3 }]
        }])
    );
}

#[test]
fn lex_number_err() {
    let nums = [(
        "1a",
        Error {
            msg: ErrorType::InvalidDigit('a'),
            lines: vec![Location { start: 0, end: 1 }],
        },
    )];
    for (n, r) in nums {
        let tok = lex(n);
        assert_eq!(Err(vec![r]), tok);
    }
}

#[test]
fn lex_identifier() {
    let idents = [("test", "test"), ("TeSt", "TeSt"), ("test123", "test123")];

    for (s, r) in idents {
        let tok = lex(s).unwrap();
        assert_eq!(tok[0].val, TokenType::Identifier(r.to_string()));
    }
    let n = "hello++";
    let t = lex(n).unwrap();
    assert_eq!(t[0].val, TokenType::Identifier("hello".to_string()));
    assert_eq!(t[1].val, TokenType::Symbol("++".to_string()));
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
        assert_eq!(tok[0].val, r);
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
        assert_eq!(tok[0].val, TokenType::String(r.to_string()));
    }

    let tok2 = lex("\"test\"++++").unwrap();
    assert_eq!(tok2[1].val, TokenType::Symbol("++++".to_string()));
}

#[test]
fn lex_string_err() {
    let strings = [
        (
            "\"",
            vec![Error {
                msg: ErrorType::StringEof,
                lines: vec![Location { start: 0, end: 1 }],
            }],
        ),
        (
            "\"test\n\"",
            vec![
                Error {
                    msg: ErrorType::StringEol,
                    lines: vec![Location { start: 0, end: 5 }],
                },
                Error {
                    msg: ErrorType::StringEof,
                    lines: vec![Location { start: 6, end: 7 }],
                },
            ],
        ),
    ];
    for (s, e) in strings {
        let tok = lex(s);
        assert_eq!(Err(e), tok);
    }
}

#[test]
fn lex_symbols() {
    let symbols = [
        "+", "-", "*", "/", "==", "<", ">", "!", "..", "$", "&", "@", "#", "??", "~", "^", ":", "%",
    ];

    for s in symbols {
        let lexed = lex(s).unwrap();
        assert_eq!(lexed[0].val, TokenType::Symbol(s.to_string()));
    }
}

#[test]
fn lex_symbols_special() {
    let symbols = [
        ("=", TokenType::Equals),
        ("?", TokenType::QuestionMark),
        (".", TokenType::Dot),
        ("|", TokenType::Pipe),
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
        assert_eq!(lexed[0].val, r);
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
        let lexed = lex(s).unwrap().iter().map(|t| t.val.clone()).collect::<Vec<_>>();
        assert_eq!(lexed, r);
    }
}

#[test]
fn lex_line_comment() {
    let coms = [
        ("//", vec![TokenType::Eof]),
        ("//test", vec![TokenType::Eof]),
        ("///test", vec![TokenType::Eof]),
        (
            "//-test",
            vec![
                TokenType::Symbol("//-".to_string()),
                TokenType::Identifier("test".to_string()),
                TokenType::Eof,
            ],
        ),
        ("// test", vec![TokenType::Eof]),
        ("/// test", vec![TokenType::Eof]),
        ("// -test", vec![TokenType::Eof]),
        (
            "// test \ntest",
            vec![TokenType::Identifier("test".to_string()), TokenType::Eof],
        ),
        (
            "/// test\ntest",
            vec![TokenType::Identifier("test".to_string()), TokenType::Eof],
        ),
    ];

    for (c, r) in coms {
        let lexed = lex(c).unwrap().iter().map(|t| t.val.clone()).collect::<Vec<_>>();
        assert_eq!(lexed, r);
    }
}

#[test]
fn lex_block_comment() {
    let comms = [
        ("/**/", vec![TokenType::Symbol("/**/".to_string()), TokenType::Eof]),
        ("/* */", vec![TokenType::Eof]),
        ("/* \n */", vec![TokenType::Eof]),
        ("/* test */", vec![TokenType::Eof]),
        (
            "test /* test */",
            vec![TokenType::Identifier("test".to_string()), TokenType::Eof],
        ),
        (
            "/* test */ test",
            vec![TokenType::Identifier("test".to_string()), TokenType::Eof],
        ),
        (
            "test /* test */ test",
            vec![
                TokenType::Identifier("test".to_string()),
                TokenType::Identifier("test".to_string()),
                TokenType::Eof,
            ],
        ),
    ];

    for (c, r) in comms {
        let lexed = lex(c).unwrap().iter().map(|t| t.val.clone()).collect::<Vec<_>>();
        assert_eq!(lexed, r);
    }
}

#[test]
fn lex_example() {
    // tests positions and whether lexer advances properly
    let exprs = [
        (
            "1 + 12",
            vec![
                Token {
                    loc: Location { start: 0, end: 0 },
                    val: TokenType::Int(1),
                },
                Token {
                    loc: Location { start: 2, end: 2 },
                    val: TokenType::Symbol("+".to_string()),
                },
                Token {
                    loc: Location { start: 4, end: 5 },
                    val: TokenType::Int(12),
                },
                Token {
                    loc: Location { start: 6, end: 6 },
                    val: TokenType::Eof,
                },
            ],
        ),
        (
            "1+12",
            vec![
                Token {
                    loc: Location { start: 0, end: 0 },
                    val: TokenType::Int(1),
                },
                Token {
                    loc: Location { start: 1, end: 1 },
                    val: TokenType::Symbol("+".to_string()),
                },
                Token {
                    loc: Location { start: 2, end: 3 },
                    val: TokenType::Int(12),
                },
                Token {
                    loc: Location { start: 4, end: 4 },
                    val: TokenType::Eof,
                },
            ],
        ),
        (
            "test2+test",
            vec![
                Token {
                    loc: Location { start: 0, end: 4 },
                    val: TokenType::Identifier("test2".to_string()),
                },
                Token {
                    loc: Location { start: 5, end: 5 },
                    val: TokenType::Symbol("+".to_string()),
                },
                Token {
                    loc: Location { start: 6, end: 9 },
                    val: TokenType::Identifier("test".to_string()),
                },
                Token {
                    loc: Location { start: 10, end: 10 },
                    val: TokenType::Eof,
                },
            ],
        ),
        (
            "\"test\"+\"test\"",
            vec![
                Token {
                    loc: Location { start: 0, end: 5 },
                    val: TokenType::String("test".to_string()),
                },
                Token {
                    loc: Location { start: 6, end: 6 },
                    val: TokenType::Symbol("+".to_string()),
                },
                Token {
                    loc: Location { start: 7, end: 12 },
                    val: TokenType::String("test".to_string()),
                },
                Token {
                    loc: Location { start: 13, end: 13 },
                    val: TokenType::Eof,
                },
            ],
        ),
        (
            "\"test\"\n+\"test\"",
            vec![
                Token {
                    loc: Location { start: 0, end: 5 },
                    val: TokenType::String("test".to_string()),
                },
                Token {
                    loc: Location { start: 7, end: 7 },
                    val: TokenType::Symbol("+".to_string()),
                },
                Token {
                    loc: Location { start: 8, end: 13 },
                    val: TokenType::String("test".to_string()),
                },
                Token {
                    loc: Location { start: 14, end: 14 },
                    val: TokenType::Eof,
                },
            ],
        ),
        (
            "\"test\" /* test */ \"test\"",
            vec![
                Token {
                    loc: Location { start: 0, end: 5 },
                    val: TokenType::String("test".to_string()),
                },
                Token {
                    loc: Location { start: 18, end: 23 },
                    val: TokenType::String("test".to_string()),
                },
                Token {
                    loc: Location { start: 24, end: 24 },
                    val: TokenType::Eof,
                },
            ],
        ),
        (
            "\"test\" /* test \ntest */ \"test\"",
            vec![
                Token {
                    loc: Location { start: 0, end: 5 },
                    val: TokenType::String("test".to_string()),
                },
                Token {
                    loc: Location { start: 24, end: 29 },
                    val: TokenType::String("test".to_string()),
                },
                Token {
                    loc: Location { start: 30, end: 30 },
                    val: TokenType::Eof,
                },
            ],
        ),
    ];

    for (ex, tok) in exprs {
        let lexed = lex(ex).unwrap();
        assert_eq!(tok, lexed, "\n{:?}", ex);
    }
}

#[test]
fn comment_operator() {
    assert_eq!(
        lex("/* fun -*/() {} */").unwrap(),
        vec![Token {
            val: TokenType::Eof,
            loc: Location { start: 18, end: 18 }
        }],
    )
}

#[test]
fn comment_stars() {
    assert_eq!(
        lex("/*** fun -*/() {} ***/").unwrap(),
        vec![Token {
            val: TokenType::Eof,
            loc: Location { start: 22, end: 22 }
        }],
    )
}

#[test]
fn comment_operator_invalid() {
    assert_eq!(
        lex("fun */() {}"),
        Err(vec![Error {
            msg: ErrorType::CommentSymbol,
            lines: vec![Location { start: 4, end: 5 }],
        }])
    )
}

#[test]
fn float_point_letter() {
    assert_eq!(
        lex("1.a1"),
        Err(vec![Error {
            msg: ErrorType::InvalidDigit('a'),
            lines: vec![Location { start: 0, end: 2 }],
        }])
    )
}
