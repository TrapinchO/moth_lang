use std::collections::HashMap;

use crate::{
    backend::value::{get_builtins, NATIVE_OPERATORS},
    error::Error,
    frontend::lexer::lex,
    frontend::parser::parse,
    frontend::reassoc,
    frontend::token::{Token, TokenType},
    located::Location,
    middle::varcheck,
};

use crate::{
    associativity::Precedence,
    error::ErrorType,
    exprstmt::{LExpr, Expr, Identifier, LStmt, Stmt, Symbol},
};

macro_rules! binop {
    ($left:expr, $op:tt, $right:expr) => {
        Expr::BinaryOperation(
            LExpr {
                val: $left.into(),
                loc: Location { start: 0, end: 0 },
            }
            .into(),
            Symbol {
                val: $op.to_string(),
                loc: Location { start: 0, end: 0 },
            },
            LExpr {
                val: $right.into(),
                loc: Location { start: 0, end: 0 },
            }
            .into(),
        )
    };
}

macro_rules! unop {
    ($op:tt, $expr:expr) => {
        Expr::UnaryOperation(
            Symbol {
                val: $op.to_string(),
                loc: Location { start: 0, end: 0 },
            },
            LExpr {
                val: $expr.into(),
                loc: Location { start: 0, end: 0 },
            }
            .into(),
        )
    };
}

macro_rules! parenop {
    ($e:expr) => {
        Expr::Parens(
            LExpr {
                val: $e.into(),
                loc: Location { start: 0, end: 0 },
            }
            .into(),
        )
    };
}

macro_rules! expr {
    ($e:expr) => {
        LExpr {
            val: $e,
            loc: Location { start: 0, end: 0 },
        }
    };
}

macro_rules! stmt {
    ($e:expr) => {
        LStmt {
            val: $e,
            loc: Location { start: 0, end: 0 },
        }
    };
}

fn compare_elements(left: &LStmt, right: &LStmt) -> bool {
    match (&left.val, &right.val) {
        (Stmt::Expr(expr1), Stmt::Expr(expr2)) => compare_elements_expr(&expr1, &expr2),
        (Stmt::VarDecl(ident1, expr1), Stmt::VarDecl(ident2, expr2)) => {
            ident1 == ident2 && compare_elements_expr(expr1, expr2)
        }
        (Stmt::Assign(ident1, expr1), Stmt::Assign(ident2, expr2)) => {
            ident1 == ident2 && compare_elements_expr(expr1, expr2)
        }
        (s1, s2) => s1 == s2,
    }
}

fn compare_elements_expr(left: &LExpr, right: &LExpr) -> bool {
    match (&left.val, &right.val) {
        (Expr::BinaryOperation(l1, o1, r1), Expr::BinaryOperation(l2, o2, r2)) => {
            compare_elements_expr(&l1, &l2) && o1.val == o2.val && compare_elements_expr(&r1, &r2)
        }
        (Expr::UnaryOperation(o1, e1), Expr::UnaryOperation(o2, e2)) => {
            o1.val == o2.val && compare_elements_expr(&e1, &e2)
        }
        (Expr::Parens(e1), Expr::Parens(e2)) => compare_elements_expr(&e1, &e2),
        (e1, e2) => e1 == e2,
    }
}

#[test]
fn test_varcheck() {
    let input = "let x = 10; x = 1;".to_string();
    let tokens = lex(&input).unwrap();
    let ast = parse(tokens).unwrap();
    let builtins = get_builtins()
        .keys()
        .map(|name| (name.clone(), (Location { start: 0, end: 0 }, false)))
        .collect::<HashMap<_, _>>();
    let checked = varcheck::varcheck(builtins, &ast);
    assert_eq!(
        checked,
        Err((
            vec![Error {
                msg: ErrorType::ItemNotUsed("x".to_string()),
                lines: vec![Location { start: 4, end: 4 }]
            }],
            vec![]
        ))
    );
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
        vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::Int(1),
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
        vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::Float(1.1),
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
        vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::String("test".to_string()),
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
        vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::Bool(true),
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
        vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::Identifier("test".to_string()),
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
        vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::Parens(
                    LExpr {
                        val: Expr::Int(1),
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
        Err(vec![Error {
            msg: ErrorType::ExpectedToken("Expected a closing parenthesis".to_string()),
            lines: vec![Location { start: 2, end: 2 }]
        }])
    );
}

#[test]
fn parse_unary() {
    assert_eq!(
        parse(lex("-1;").unwrap()).unwrap(),
        vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::UnaryOperation(
                    Symbol {
                        val: "-".to_string(),
                        loc: Location { start: 0, end: 0 },
                    },
                    LExpr {
                        val: Expr::Int(1),
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
        vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::UnaryOperation(
                    Symbol {
                        val: "-".to_string(),
                        loc: Location { start: 0, end: 0 },
                    },
                    LExpr {
                        val: Expr::UnaryOperation(
                            Symbol {
                                val: "-".to_string(),
                                loc: Location { start: 2, end: 2 },
                            },
                            LExpr {
                                val: Expr::Int(1),
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
        vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::BinaryOperation(
                    LExpr {
                        val: Expr::Int(1),
                        loc: Location { start: 0, end: 0 },
                    }
                    .into(),
                    Symbol {
                        val: "+".to_string(),
                        loc: Location { start: 2, end: 2 },
                    },
                    LExpr {
                        val: Expr::Int(1),
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
    let err = vec![Error {
        msg: ErrorType::UnexpectedEof,
        lines: vec![Location { start: 2, end: 2 }],
    }];
    let op = parse(lex("1+").unwrap()).unwrap_err();
    assert_eq!(err, op);
}

#[test]
fn parse_binary2() {
    let ops = [
        ("1+1", binop!(Expr::Int(1), "+", Expr::Int(1))),
        ("1 + 1", binop!(Expr::Int(1), "+", Expr::Int(1))),
        ("1-1", binop!(Expr::Int(1), "-", Expr::Int(1))),
        ("1**1", binop!(Expr::Int(1), "**", Expr::Int(1))),
        (
            "1+1+1",
            binop!(Expr::Int(1), "+", binop!(Expr::Int(1), "+", Expr::Int(1))),
        ),
    ];
    for (s, op) in ops {
        assert!(compare_elements(
            &parse(lex(&(s.to_owned() + ";")).unwrap()).unwrap()[0],
            &stmt!(Stmt::Expr(expr!(op)))
        ));
    }
}

#[test]
fn parse_unary2() {
    let ops = [
        ("-1", unop!("-", Expr::Int(1))),
        ("- - !1", unop!("-", unop!("-", unop!("!", Expr::Int(1))))),
        ("(-1)", parenop!(unop!("-", Expr::Int(1)))),
    ];
    for (s, op) in ops {
        assert!(compare_elements(
            &parse(lex(&(s.to_owned() + ";")).unwrap()).unwrap()[0],
            &stmt!(Stmt::Expr(expr!(op)))
        ));
    }
}

#[test]
fn test_reassoc() {
    let ops = [
        (
            "1 - 1 - 1",
            binop!(binop!(Expr::Int(1), "-", Expr::Int(1)), "-", Expr::Int(1)),
        ),
        (
            "-(1 - 1 - 1)",
            unop!(
                "-",
                parenop!(binop!(
                    binop!(Expr::Int(1), "-", Expr::Int(1)),
                    "-",
                    Expr::Int(1)
                ))
            ),
        ),
    ];
    let symbols: std::collections::HashMap<String, Precedence> = NATIVE_OPERATORS
        .map(|(name, assoc, _)| (name.to_string(), assoc))
        .into();
    for (s, op) in ops {
        assert!(compare_elements(
            &reassoc::reassociate(symbols.clone(), parse(lex(&(s.to_owned() + ";")).unwrap()).unwrap()).unwrap()[0],
            &stmt!(Stmt::Expr(expr!(op)))
        ));
    }
}

#[test]
fn test_nested_call() {
    let src = parse(lex("f()();").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(
                LExpr {
                    val: Expr::Call(
                        LExpr {
                            val: Expr::Call(
                                LExpr {
                                    val: Expr::Identifier("f".to_string()),
                                    loc: Location { start: 0, end: 0 },
                                }
                                .into(),
                                vec![]
                            ),
                            loc: Location { start: 0, end: 2 },
                        }
                        .into(),
                        vec![]
                    ),
                    loc: Location { start: 0, end: 4 },
                }
                .into()
            ),
            loc: Location { start: 0, end: 4 },
        }])
    )
}

#[test]
fn test_nested_index() {
    let src = parse(lex("x[1][1];").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(
                LExpr {
                    val: Expr::Index(
                        LExpr {
                            val: Expr::Index(
                                LExpr {
                                    val: Expr::Identifier("x".to_string()),
                                    loc: Location { start: 0, end: 0 },
                                }
                                .into(),
                                LExpr {
                                    val: Expr::Int(1),
                                    loc: Location { start: 2, end: 2 },
                                }
                                .into()
                            ),
                            loc: Location { start: 0, end: 3 },
                        }
                        .into(),
                        LExpr {
                            val: Expr::Int(1),
                            loc: Location { start: 5, end: 5 },
                        }
                        .into()
                    ),
                    loc: Location { start: 0, end: 6 },
                }
                .into()
            ),
            loc: Location { start: 0, end: 6 },
        }])
    )
}

#[test]
fn test_no_params() {
    let src = parse(lex("fun f() {}").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::FunDecl(
                Identifier {
                    val: "f".to_string(),
                    loc: Location { start: 4, end: 4 },
                },
                vec![], // NO PARAMS
                vec![],
            ),
            loc: Location { start: 0, end: 9 },
        }])
    );
}

#[test]
fn test_one_param() {
    let src = parse(lex("fun f(x) {}").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::FunDecl(
                Identifier {
                    val: "f".to_string(),
                    loc: Location { start: 4, end: 4 },
                },
                vec![Identifier {
                    val: "x".to_string(),
                    loc: Location { start: 6, end: 6 }
                }],
                vec![],
            ),
            loc: Location { start: 0, end: 10 },
        }])
    );
}

#[test]
fn test_more_params() {
    let src = parse(lex("fun f(x, y, z) {}").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::FunDecl(
                Identifier {
                    val: "f".to_string(),
                    loc: Location { start: 4, end: 4 },
                },
                vec![
                    Identifier {
                        val: "x".to_string(),
                        loc: Location { start: 6, end: 6 }
                    },
                    Identifier {
                        val: "y".to_string(),
                        loc: Location { start: 9, end: 9 }
                    },
                    Identifier {
                        val: "z".to_string(),
                        loc: Location { start: 12, end: 12 }
                    },
                ],
                vec![],
            ),
            loc: Location { start: 0, end: 16 },
        }])
    );
}

#[test]
fn test_index_call() {
    let src = parse(lex("[print][0]();").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::Call(
                    LExpr {
                        val: Expr::Index(
                            LExpr {
                                val: Expr::List(vec![LExpr {
                                    val: Expr::Identifier("print".to_string()),
                                    loc: Location { start: 1, end: 5 },
                                }]),
                                loc: Location { start: 0, end: 6 },
                            }
                            .into(),
                            LExpr {
                                val: Expr::Int(0),
                                loc: Location { start: 8, end: 8 },
                            }
                            .into()
                        ),
                        loc: Location { start: 0, end: 9 },
                    }
                    .into(),
                    vec![]
                ),
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
        Ok(vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::Index(
                    LExpr {
                        val: Expr::Call(
                            LExpr {
                                val: Expr::List(vec![LExpr {
                                    val: Expr::Identifier("print".to_string()),
                                    loc: Location { start: 1, end: 5 },
                                }]),
                                loc: Location { start: 0, end: 6 },
                            }
                            .into(),
                            vec![],
                        ),
                        loc: Location { start: 0, end: 8 },
                    }
                    .into(),
                    LExpr {
                        val: Expr::Int(0),
                        loc: Location { start: 10, end: 10 },
                    }
                    .into()
                ),
                loc: Location { start: 0, end: 11 },
            }),
            loc: Location { start: 0, end: 11 },
        }]),
    );
}

#[test]
fn test_symbol_ident() {
    let src = parse(lex("(-);").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::Identifier("-".to_string()),
                loc: Location { start: 0, end: 2 },
            }),
            loc: Location { start: 0, end: 2 },
        }]),
    )
}

#[test]
fn test_paren_unary() {
    let src = parse(lex("(-1);").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::Parens(
                    LExpr {
                        val: Expr::UnaryOperation(
                            Symbol {
                                val: "-".to_string(),
                                loc: Location { start: 1, end: 1 }
                            },
                            LExpr {
                                val: Expr::Int(1),
                                loc: Location { start: 2, end: 2 },
                            }
                            .into(),
                        ),
                        loc: Location { start: 1, end: 2 },
                    }
                    .into()
                ),
                loc: Location { start: 0, end: 3 },
            }),
            loc: Location { start: 0, end: 3 },
        }]),
    )
}

#[test]
fn test_assingment() {
    let src = parse(lex("x = 1;").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Assign(
                Identifier {
                    val: "x".to_string(),
                    loc: Location { start: 0, end: 0 },
                },
                LExpr {
                    val: Expr::Int(1),
                    loc: Location { start: 4, end: 4 },
                }
            ),
            loc: Location { start: 0, end: 4 },
        }])
    )
}

#[test]
fn test_assingment_index() {
    let src = parse(lex("x[0] = 1;").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::AssignIndex(
                LExpr {
                    val: Expr::Identifier("x".to_string()),
                    loc: Location { start: 0, end: 0 },
                },
                LExpr {
                    val: Expr::Int(0),
                    loc: Location { start: 2, end: 2 },
                },
                LExpr {
                    val: Expr::Int(1),
                    loc: Location { start: 7, end: 7 },
                }
            ),
            loc: Location { start: 0, end: 7 },
        }])
    )
}

#[test]
fn test_assingment_error() {
    let src = parse(lex("1 = 1;").unwrap());
    assert_eq!(
        src,
        Err(vec![Error {
            msg: ErrorType::InvalidAssignmentTarget,
            lines: vec![Location { start: 0, end: 0 }],
        }])
    )
}

#[test]
fn list_empty() {
    let src = parse(lex("[];").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::List(vec![]),
                loc: Location { start: 0, end: 1 },
            }),
            loc: Location { start: 0, end: 1 },
        }])
    )
}

#[test]
fn list_one() {
    let src = parse(lex("[1];").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::List(vec![LExpr {
                    val: Expr::Int(1),
                    loc: Location { start: 1, end: 1 },
                }]),
                loc: Location { start: 0, end: 2 },
            }),
            loc: Location { start: 0, end: 2 },
        }])
    );
    let src = parse(lex("[1, ];").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::List(vec![LExpr {
                    val: Expr::Int(1),
                    loc: Location { start: 1, end: 1 },
                }]),
                loc: Location { start: 0, end: 4 },
            }),
            loc: Location { start: 0, end: 4 },
        }])
    )
}

#[test]
fn list_more() {
    let src = parse(lex("[1, 2];").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::List(vec![
                    LExpr {
                        val: Expr::Int(1),
                        loc: Location { start: 1, end: 1 },
                    },
                    LExpr {
                        val: Expr::Int(2),
                        loc: Location { start: 4, end: 4 },
                    },
                ]),
                loc: Location { start: 0, end: 5 },
            }),
            loc: Location { start: 0, end: 5 },
        }])
    )
}

#[test]
fn unary_parenthesis() {
    let src = parse(lex("(-10 * 10);").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(LExpr {
                val: Expr::Parens(
                    LExpr {
                        val: Expr::BinaryOperation(
                            LExpr {
                                val: Expr::UnaryOperation(
                                    Symbol {
                                        val: "-".to_string(),
                                        loc: Location { start: 1, end: 1 }
                                    },
                                    LExpr {
                                        val: Expr::Int(10),
                                        loc: Location { start: 2, end: 3 }
                                    }
                                    .into()
                                ),
                                loc: Location { start: 1, end: 3 },
                            }
                            .into(),
                            Symbol {
                                val: "*".to_string(),
                                loc: Location { start: 5, end: 5 }
                            },
                            LExpr {
                                val: Expr::Int(10),
                                loc: Location { start: 7, end: 8 },
                            }
                            .into()
                        ),
                        loc: Location { start: 1, end: 8 }
                    }
                    .into()
                ),
                loc: Location { start: 0, end: 9 },
            }),
            loc: Location { start: 0, end: 9 },
        }])
    )
}

#[test]
fn lambda() {
    let src = parse(lex("|| 1;").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(
                LExpr {
                    val: Expr::Lambda(
                        vec![],
                        vec![LStmt {
                            val: Stmt::Return(
                                LExpr {
                                    val: Expr::Int(1),
                                    loc: Location { start: 3, end: 3 },
                                }
                                .into()
                            ),
                            loc: Location { start: 3, end: 3 },
                        }]
                    ),
                    loc: Location { start: 0, end: 3 },
                }
                .into()
            ),
            loc: Location { start: 0, end: 3 },
        }])
    )
}

#[test]
fn lambda_params() {
    let src = parse(lex("|x, y| ();").unwrap());
    assert_eq!(
        src,
        Ok(vec![LStmt {
            val: Stmt::Expr(
                LExpr {
                    val: Expr::Lambda(
                        vec![
                            Identifier {
                                val: "x".to_string(),
                                loc: Location { start: 1, end: 1 }
                            },
                            Identifier {
                                val: "y".to_string(),
                                loc: Location { start: 4, end: 4 }
                            },
                        ],
                        vec![LStmt {
                            val: Stmt::Return(
                                LExpr {
                                    val: Expr::Unit,
                                    loc: Location { start: 7, end: 8 },
                                }
                                .into()
                            ),
                            loc: Location { start: 7, end: 8 },
                        }]
                    ),
                    loc: Location { start: 0, end: 8 },
                }
                .into()
            ),
            loc: Location { start: 0, end: 8 },
        }])
    )
}
