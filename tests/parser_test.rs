use moth_lang::exprstmt::{Expr, ExprType, Stmt, StmtType};

macro_rules! binop {
    ($left:expr, $op:tt, $right:expr) => {
        ExprType::BinaryOperation(
            Expr {
                val: $left.into(),
                start: 0,
                end: 0,
            }
            .into(),
            Token {
                val: TokenType::Symbol($op.to_string()),
                start: 0,
                end: 0,
            },
            Expr {
                val: $right.into(),
                start: 0,
                end: 0,
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
                start: 0,
                end: 0,
            },
            Expr {
                val: $expr.into(),
                start: 0,
                end: 0,
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
                start: 0,
                end: 0,
            }
            .into(),
        )
    };
}

macro_rules! expr {
    ($e:expr) => {
        Expr {
            val: $e,
            start: 0,
            end: 0,
        }
    };
}

macro_rules! stmt {
    ($e:expr) => {
        Stmt {
            val: $e,
            start: 0,
            end: 0,
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

#[cfg(test)]
mod tests {
    use moth_lang::{
        error::Error,
        exprstmt::*,
        lexer::lex,
        parser::parse,
        reassoc,
        token::{Token, TokenType},
        value::{get_builtins, NATIVE_OPERATORS},
        varcheck,
    };

    use crate::compare_elements;

    #[test]
    fn test_varcheck() -> Result<(), Error> {
        let input = "let x = 10; x = 1;".to_string();
        let tokens = lex(&input)?;
        let ast = parse(tokens)?;
        let checked = varcheck::varcheck(get_builtins(), ast);
        if let Ok(_) = checked {
            return Ok(());
        }
        panic!("not ok {:?}", checked);
    }

    #[test]
    fn parse_empty() {
        assert_eq!(
            parse(vec![Token {
                start: 0,
                end: 0,
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
                    start: 0,
                    end: 0
                }),
                start: 0,
                end: 0
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
                    start: 0,
                    end: 2
                }),
                start: 0,
                end: 2
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
                    start: 0,
                    end: 5
                }),
                start: 0,
                end: 5
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
                    start: 0,
                    end: 3
                }),
                start: 0,
                end: 3
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
                    start: 0,
                    end: 3
                }),
                start: 0,
                end: 3
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
                            start: 1,
                            end: 1
                        }
                        .into()
                    ),
                    start: 0,
                    end: 2
                }),
                start: 0,
                end: 2
            }]
        );
    }

    #[test]
    fn parse_parens_unclosed() {
        assert_eq!(
            parse(lex("(1").unwrap()),
            Err(Error {
                msg: "Expected closing parenthesis".to_string(),
                lines: vec![(2, 2)]
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
                            start: 0,
                            end: 0
                        },
                        Expr {
                            val: ExprType::Int(1),
                            start: 1,
                            end: 1
                        }
                        .into()
                    ),
                    start: 0,
                    end: 1
                }),
                start: 0,
                end: 1
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
                            start: 0,
                            end: 0
                        },
                        Expr {
                            val: ExprType::UnaryOperation(
                                Token {
                                    val: TokenType::Symbol("-".to_string()),
                                    start: 2,
                                    end: 2
                                },
                                Expr {
                                    val: ExprType::Int(1),
                                    start: 3,
                                    end: 3
                                }
                                .into()
                            ),
                            start: 2,
                            end: 3
                        }
                        .into()
                    ),
                    start: 0,
                    end: 3
                }),
                start: 0,
                end: 3
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
                            start: 0,
                            end: 0
                        }
                        .into(),
                        Token {
                            val: TokenType::Symbol("+".to_string()),
                            start: 2,
                            end: 2
                        },
                        Expr {
                            val: ExprType::Int(1),
                            start: 4,
                            end: 4
                        }
                        .into()
                    ),
                    start: 0,
                    end: 4
                }),
                start: 0,
                end: 4
            }]
        )
    }

    #[test]
    fn expr_error() {
        let err = Error {
            msg: "Expected an element but reached EOF".to_string(),
            lines: vec![(2, 2)],
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
            ("+1", unop!("+", ExprType::Int(1))),
            ("* - +1", unop!("*", unop!("-", unop!("+", ExprType::Int(1))))),
            ("*-+1", unop!("*-+", ExprType::Int(1))),
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
                "+(1 - 1 - 1)",
                unop!(
                    "+",
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
                &reassoc::reassociate(symbols.clone(), parse(lex(&(s.to_owned() + ";")).unwrap()).unwrap()).unwrap()
                    [0],
                &stmt!(StmtType::ExprStmt(expr!(op)))
            ));
        }
    }
}
