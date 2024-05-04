use crate::exprstmt::{Expr, ExprType, Stmt, StmtType};

macro_rules! binop {
    ($left:expr, $op:tt, $right:expr) => {
        ExprType::BinaryOperation(
            Expr {
                val: $left.into(),
                loc: Location { start: 0, end: 0 },
            }
            .into(),
            Token {
                val: TokenType::Symbol($op.to_string()),
                loc: Location { start: 0, end: 0 },
            },
            Expr {
                val: $right.into(),
                loc: Location { start: 0, end: 0 },
            }
            .into(),
        )
    };
}

macro_rules! unop {
    ($op:tt, $expr:expr) => {
        ExprType::UnaryOperation(
            Token {
                val: TokenType::Symbol($op.to_string()),
                loc: Location { start: 0, end: 0 },
            },
            Expr {
                val: $expr.into(),
                loc: Location { start: 0, end: 0 },
            }
            .into(),
        )
    };
}

macro_rules! parenop {
    ($e:expr) => {
        ExprType::Parens(
            Expr {
                val: $e.into(),
                loc: Location { start: 0, end: 0 },
            }
            .into(),
        )
    };
}

macro_rules! expr {
    ($e:expr) => {
        Expr {
            val: $e,
            loc: Location { start: 0, end: 0 },
        }
    };
}

macro_rules! stmt {
    ($e:expr) => {
        Stmt {
            val: $e,
            loc: Location { start: 0, end: 0 },
        }
    };
}

fn compare_elements(left: &Stmt, right: &Stmt) -> bool {
    match (&left.val, &right.val) {
        (StmtType::ExprStmt(expr1), StmtType::ExprStmt(expr2)) => compare_elements_expr(&expr1, &expr2),
        (StmtType::VarDeclStmt(ident1, expr1), StmtType::VarDeclStmt(ident2, expr2)) => {
            ident1 == ident2 && compare_elements_expr(expr1, expr2)
        }
        (StmtType::AssignStmt(ident1, expr1), StmtType::AssignStmt(ident2, expr2)) => {
            ident1 == ident2 && compare_elements_expr(expr1, expr2)
        }
        (s1, s2) => s1 == s2,
    }
}

fn compare_elements_expr(left: &Expr, right: &Expr) -> bool {
    match (&left.val, &right.val) {
        (ExprType::BinaryOperation(l1, o1, r1), ExprType::BinaryOperation(l2, o2, r2)) => {
            compare_elements_expr(&l1, &l2) && o1.val == o2.val && compare_elements_expr(&r1, &r2)
        }
        (ExprType::UnaryOperation(o1, e1), ExprType::UnaryOperation(o2, e2)) => {
            o1.val == o2.val && compare_elements_expr(&e1, &e2)
        }
        (ExprType::Parens(e1), ExprType::Parens(e2)) => compare_elements_expr(&e1, &e2),
        (e1, e2) => e1 == e2,
    }
}

use std::{collections::HashMap, vec};

use crate::{
    error::Error,
    lexer::lex,
    located::Location,
    parser::parse,
    reassoc,
    token::{Token, TokenType},
    value::{get_builtins, NATIVE_OPERATORS},
    varcheck,
};

#[test]
fn test_varcheck() -> Result<(), Error> {
    let input = "let x = 10; x = 1;".to_string();
    let tokens = lex(&input)?;
    let ast = parse(tokens)?;
    let builtins = get_builtins()
        .keys()
        .map(|name| (name.clone(), (Location { start: 0, end: 0 }, false)))
        .collect::<HashMap<_, _>>();
    let checked = varcheck::varcheck(builtins, &ast);
    assert_eq!(
        checked,
        Err((
            vec![Error {
                msg: "Variable \"x\" not used.".to_string(),
                lines: vec![Location { start: 4, end: 4 }]
            }],
            vec![]
        ))
    );
    Ok(())
}

#[test]
fn parse_empty() {
    assert_eq!(
        parse(vec![Token {
            loc: Location { start: 0, end: 0 },
            val: TokenType::Eof
        }]),
        Ok(vec![])
    );
    assert_eq!(parse(vec![]), Ok(vec![]));
}

#[test]
fn parse_int() {
    assert_eq!(
        parse(lex("1;").unwrap()).unwrap(),
        vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::Int(1),
                loc: Location { start: 0, end: 0 },
            }),
            loc: Location { start: 0, end: 0 },
        }]
    );
}

#[test]
fn parse_float() {
    assert_eq!(
        parse(lex("1.1;").unwrap()).unwrap(),
        vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::Float(1.1),
                loc: Location { start: 0, end: 2 },
            }),
            loc: Location { start: 0, end: 2 },
        }]
    );
}

#[test]
fn parse_string() {
    assert_eq!(
        parse(lex("\"test\";").unwrap()).unwrap(),
        vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::String("test".to_string()),
                loc: Location { start: 0, end: 5 },
            }),
            loc: Location { start: 0, end: 5 },
        }]
    );
}

#[test]
fn parse_bool() {
    assert_eq!(
        parse(lex("true;").unwrap()).unwrap(),
        vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::Bool(true),
                loc: Location { start: 0, end: 3 },
            }),
            loc: Location { start: 0, end: 3 },
        }]
    );
}

#[test]
fn parse_identifier() {
    assert_eq!(
        parse(lex("test;").unwrap()).unwrap(),
        vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::Identifier("test".to_string()),
                loc: Location { start: 0, end: 3 },
            }),
            loc: Location { start: 0, end: 3 },
        }]
    );
}

#[test]
fn parse_parens() {
    assert_eq!(
        parse(lex("(1);").unwrap()).unwrap(),
        vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::Parens(
                    Expr {
                        val: ExprType::Int(1),
                        loc: Location { start: 1, end: 1 },
                    }
                    .into()
                ),
                loc: Location { start: 0, end: 2 },
            }),
            loc: Location { start: 0, end: 2 },
        }]
    );
}

#[test]
fn parse_parens_unclosed() {
    assert_eq!(
        parse(lex("(1").unwrap()),
        Err(Error {
            msg: "Expected closing parenthesis".to_string(),
            lines: vec![Location { start: 2, end: 2 }]
        })
    );
}

#[test]
fn parse_unary() {
    assert_eq!(
        parse(lex("-1;").unwrap()).unwrap(),
        vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::UnaryOperation(
                    Token {
                        val: TokenType::Symbol("-".to_string()),
                        loc: Location { start: 0, end: 0 },
                    },
                    Expr {
                        val: ExprType::Int(1),
                        loc: Location { start: 1, end: 1 },
                    }
                    .into()
                ),
                loc: Location { start: 0, end: 1 },
            }),
            loc: Location { start: 0, end: 1 },
        }]
    );
}

#[test]
fn parse_unary_nested() {
    assert_eq!(
        parse(lex("- -1;").unwrap()).unwrap(),
        vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::UnaryOperation(
                    Token {
                        val: TokenType::Symbol("-".to_string()),
                        loc: Location { start: 0, end: 0 },
                    },
                    Expr {
                        val: ExprType::UnaryOperation(
                            Token {
                                val: TokenType::Symbol("-".to_string()),
                                loc: Location { start: 2, end: 2 },
                            },
                            Expr {
                                val: ExprType::Int(1),
                                loc: Location { start: 3, end: 3 },
                            }
                            .into()
                        ),
                        loc: Location { start: 2, end: 3 },
                    }
                    .into()
                ),
                loc: Location { start: 0, end: 3 },
            }),
            loc: Location { start: 0, end: 3 },
        }]
    );
}

#[test]
fn parse_binary() {
    assert_eq!(
        parse(lex("1 + 1;").unwrap()).unwrap(),
        vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::BinaryOperation(
                    Expr {
                        val: ExprType::Int(1),
                        loc: Location { start: 0, end: 0 },
                    }
                    .into(),
                    Token {
                        val: TokenType::Symbol("+".to_string()),
                        loc: Location { start: 2, end: 2 },
                    },
                    Expr {
                        val: ExprType::Int(1),
                        loc: Location { start: 4, end: 4 },
                    }
                    .into()
                ),
                loc: Location { start: 0, end: 4 },
            }),
            loc: Location { start: 0, end: 4 },
        }]
    )
}

#[test]
fn expr_error() {
    let err = Error {
        msg: "Expected an element but reached EOF".to_string(),
        lines: vec![Location { start: 2, end: 2 }],
    };
    let op = parse(lex("1+").unwrap()).unwrap_err();
    assert_eq!(err, op);
}

#[test]
fn parse_binary2() {
    let ops = [
        ("1+1", binop!(ExprType::Int(1), "+", ExprType::Int(1))),
        ("1 + 1", binop!(ExprType::Int(1), "+", ExprType::Int(1))),
        ("1-1", binop!(ExprType::Int(1), "-", ExprType::Int(1))),
        ("1**1", binop!(ExprType::Int(1), "**", ExprType::Int(1))),
        (
            "1+1+1",
            binop!(ExprType::Int(1), "+", binop!(ExprType::Int(1), "+", ExprType::Int(1))),
        ),
    ];
    for (s, op) in ops {
        assert!(compare_elements(
            &parse(lex(&(s.to_owned() + ";")).unwrap()).unwrap()[0],
            &stmt!(StmtType::ExprStmt(expr!(op)))
        ));
    }
}

#[test]
fn parse_unary2() {
    let ops = [
        ("-1", unop!("-", ExprType::Int(1))),
        ("- - !1", unop!("-", unop!("-", unop!("!", ExprType::Int(1))))),
    ];
    for (s, op) in ops {
        assert!(compare_elements(
            &parse(lex(&(s.to_owned() + ";")).unwrap()).unwrap()[0],
            &stmt!(StmtType::ExprStmt(expr!(op)))
        ));
    }
}

#[test]
fn test_reassoc() {
    let ops = [
        (
            "1 - 1 - 1",
            binop!(binop!(ExprType::Int(1), "-", ExprType::Int(1)), "-", ExprType::Int(1)),
        ),
        (
            "-(1 - 1 - 1)",
            unop!(
                "-",
                parenop!(binop!(
                    binop!(ExprType::Int(1), "-", ExprType::Int(1)),
                    "-",
                    ExprType::Int(1)
                ))
            ),
        ),
    ];
    let symbols: std::collections::HashMap<String, reassoc::Precedence> = NATIVE_OPERATORS
        .map(|(name, assoc, _)| (name.to_string(), assoc))
        .into();
    for (s, op) in ops {
        assert!(compare_elements(
            &reassoc::reassociate(symbols.clone(), parse(lex(&(s.to_owned() + ";")).unwrap()).unwrap()).unwrap()[0],
            &stmt!(StmtType::ExprStmt(expr!(op)))
        ));
    }
}

#[test]
fn test_nested_call() {
    let src = parse(lex("f()();").unwrap());
    assert_eq!(
        src,
        Ok(vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::Call(Expr {
                    val: ExprType::Call(Expr {
                        val: ExprType::Identifier("f".to_string()),
                        loc: Location { start: 0, end: 0 },
                    }.into(), vec![]),
                    loc: Location { start: 0, end: 2 },
                }.into(), vec![]),
                loc: Location { start: 0, end: 4 },
            }.into()),
            loc: Location { start: 0, end: 4 },
        }])
    )
}


#[test]
fn test_nested_index() {
    let src = parse(lex("x[1][1];").unwrap());
    assert_eq!(
        src,
        Ok(vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::Index(Expr {
                    val: ExprType::Index(Expr {
                        val: ExprType::Identifier("x".to_string()),
                        loc: Location { start: 0, end: 0 },
                    }.into(),
                    Expr {
                        val: ExprType::Int(1),
                        loc: Location { start: 2, end: 2 },
                    }.into()),
                    loc: Location { start: 0, end: 3 },
                }.into(),
                    Expr {
                        val: ExprType::Int(1),
                        loc: Location { start: 5, end: 5 },
                    }.into()),
                loc: Location { start: 0, end: 6 },
            }.into()),
            loc: Location { start: 0, end: 6 },
        }])
    )
}

#[test]
fn test_no_params() {
    let src = parse(lex("fun f() {}").unwrap());
    assert_eq!(
        src,
        Ok(vec![
           Stmt {
               val: StmtType::FunDeclStmt(
                    Token {
                        val: TokenType::Identifier("f".to_string()),
                        loc: Location { start: 4, end: 4 },
                    },
                    vec![],  // NO PARAMS
                    vec![],
               ),
               loc: Location { start: 0, end: 9 },
           }
        ])
    );
}

#[test]
fn test_index_call() {
    let src = parse(lex("[print][0]();").unwrap());
    assert_eq!(
        src,
        Ok(vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::Call(
                    Expr {
                        val: ExprType::Index(
                            Expr {
                                val: ExprType::List(vec![Expr {
                                    val: ExprType::Identifier("print".to_string()),
                                    loc: Location { start: 1, end: 5 }, }]),
                                loc: Location { start: 0, end: 6 },
                                }.into(),
                            Expr {
                                val: ExprType::Int(0),
                                loc: Location { start: 8, end: 8 },
                            }.into()),
                        loc: Location { start: 0, end: 9 },
                    }.into(),
                    vec![]),
                loc: Location { start: 0, end: 11 },
            }),
            loc: Location { start: 0, end: 11 },
        }]),
    );
}

#[test]
fn test_call_index() {
    let src = parse(lex("[print]()[0];").unwrap());
    assert_eq!(
        src,
        Ok(vec![Stmt {
            val: StmtType::ExprStmt(Expr {
                val: ExprType::Index(
                    Expr {
                        val: ExprType::Call(
                            Expr {
                                val: ExprType::List(vec![Expr {
                                    val: ExprType::Identifier("print".to_string()),
                                    loc: Location { start: 1, end: 5 }, }]),
                                loc: Location { start: 0, end: 6 },
                                }.into(),
                            vec![],),
                        loc: Location { start: 0, end: 8 },
                    }.into(),
                    Expr {
                        val: ExprType::Int(0),
                        loc: Location { start: 10, end: 10 },
                    }.into()),
                loc: Location { start: 0, end: 11 },
            }),
            loc: Location { start: 0, end: 11 },
        }]),
    );
}
