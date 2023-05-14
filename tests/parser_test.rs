use moth_lang::parser::{Expr, ExprType, Stmt};

macro_rules! binop {
    ($left:expr, $op:tt, $right:expr) => {
        ExprType::BinaryOperation(
            Expr {
                typ: $left.into(),
                start: 0,
                end: 0,
                line: 0,
            }.into(),
            Token {
                typ: TokenType::Symbol($op.to_string()),
                start: 0,
                end: 0,
                line: 0,
            },
            Expr {
                typ: $right.into(),
                start: 0,
                end: 0,
                line: 0
            }.into(),
        )
    };
}

macro_rules! unop {
    ($op:tt, $expr:expr) => {
        ExprType::UnaryOperation(
            Token {
                typ: TokenType::Symbol($op.to_string()),
                start: 0,
                end: 0,
                line: 0,
            },
            Expr {
                typ: $expr.into(),
                start: 0,
                end: 0,
                line: 0,
            }.into()
        )
    };
}

macro_rules! parenop {
    ($e:expr) => {
        ExprType::Parens(
            Expr {
                typ: $e.into(),
                start: 0,
                end: 0,
                line: 0,
            }.into()
        )
    };
}

macro_rules! expr {
    ($e:expr) => {
        Expr {
            typ: $e,
            start: 0,
            end: 0,
            line: 0,
        }
    };
}

fn compare_elements(left: &Stmt, right: &Stmt) -> bool {
    match (&left, &right) {
        (Stmt::ExprStmt(expr1), Stmt::ExprStmt(expr2)) => {
            compare_elements_expr(&expr1, &expr2)
        }
        (Stmt::AssingmentStmt(ident1, expr1), Stmt::AssingmentStmt(ident2, expr2)) => {
            ident1 == ident2 && compare_elements_expr(expr1, expr2)
        }
        (s1, s2) => s1 == s2
    }
}

fn compare_elements_expr(left: &Expr, right: &Expr) -> bool {
    match (&left.typ, &right.typ) {
        (ExprType::BinaryOperation(l1, o1, r1), ExprType::BinaryOperation(l2, o2, r2)) => {
            compare_elements_expr(&l1, &l2) && o1.typ == o2.typ && compare_elements_expr(&r1, &r2)
        }
        (ExprType::UnaryOperation(o1, e1), ExprType::UnaryOperation(o2, e2)) => {
            o1.typ == o2.typ && compare_elements_expr(&e1, &e2)
        }
        (ExprType::Parens(e1), ExprType::Parens(e2)) => compare_elements_expr(&e1, &e2),
        (e1, e2) => e1 == e2,
    }
}

#[cfg(test)]
mod tests {
    use moth_lang::error::Error;
    use moth_lang::lexer::{lex, Token, TokenType};
    use moth_lang::parser::*;
    use moth_lang::reassoc;

    use crate::compare_elements;

    #[test]
    #[should_panic(expected = "Expected code to parse")]
    fn empty() {
        parse(vec![Token {
            start: 0,
            end: 0,
            line: 0,
            typ: TokenType::Eof,
        }])
        .unwrap();
    }

    #[test]
    fn parse_primary() {
        let ops = [
            ("1", expr!(ExprType::Number(1))),
            ("1234", expr!(ExprType::Number(1234))),
            ("\"\"", expr!(ExprType::String("".to_string()))),
            ("\"test\"", expr!(ExprType::String("test".to_string()))),
            ("(1)", expr!(ExprType::Parens(expr!(ExprType::Number(1)).into()))),
            ("(1 + 1)", expr!(parenop!(binop!(ExprType::Number(1), "+", ExprType::Number(1))))),
        ];
        for (s, op) in ops {
            assert!(compare_elements(
                    &parse(lex(s).unwrap()).unwrap(),
                    &Stmt::ExprStmt(op)
            ));
        }
    }

    #[test]
    fn expr_error() {
        let err = Error {
            msg: "Expected an element".to_string(),
            lines: vec![(0, 2, 2)],
        };
        let op = parse(lex("1+").unwrap()).unwrap_err();
        assert_eq!(err, op);
    }

    #[test]
    fn parens_missing() {
        let err = Error {
            msg: "Expected closing parenthesis".to_string(),
            lines: vec![(0, 2, 2)],
        };
        let op = parse(lex("(1").unwrap()).unwrap_err();
        assert_eq!(err, op);
    }

    #[test]
    fn parens_empty() {
        let err = Error {
            msg: "Expected an expression".to_string(),
            lines: vec![(0, 1, 1)],
        };
        let op = parse(lex("()").unwrap()).unwrap_err();
        assert_eq!(err, op);
    }

    #[test]
    fn parse_binary() {
        let ops = [
            ("1+1", binop!(ExprType::Number(1), "+", ExprType::Number(1))),
            ("1 + 1", binop!(ExprType::Number(1), "+", ExprType::Number(1))),
            ("1-1", binop!(ExprType::Number(1), "-", ExprType::Number(1))),
            ("1**1", binop!(ExprType::Number(1), "**", ExprType::Number(1))),

            ("1+1+1", binop!(ExprType::Number(1), "+", binop!(ExprType::Number(1), "+", ExprType::Number(1)))),
        ];
        for (s, op) in ops {
            assert!(compare_elements(
                &parse(lex(s).unwrap()).unwrap(),
                &Stmt::ExprStmt(expr!(op))
            ));
        }
    }

    #[test]
    fn parse_unary() {
        let ops = [
            ("+1", unop!("+", ExprType::Number(1))),
            ("* - +1", unop!("*", unop!("-", unop!("+", ExprType::Number(1))))),
            ("*-+1", unop!("*-+", ExprType::Number(1))),
        ];
        for (s, op) in ops {
            assert!(compare_elements(
                &parse(lex(s).unwrap()).unwrap(),
                &Stmt::ExprStmt(expr!(op))
            ));
        }
    }

    #[test]
    fn test_reassoc() {
        let ops = [
            ("1 - 1 - 1", binop!(
                    binop!(ExprType::Number(1), "-", ExprType::Number(1)),
                    "-",
                    ExprType::Number(1))),
            ("+(1 - 1 - 1)", unop!("+", parenop!(binop!(
                    binop!(ExprType::Number(1), "-", ExprType::Number(1)),
                    "-",
                    ExprType::Number(1))))),
        ];
        for (s, op) in ops {
            assert!(compare_elements(
                &reassoc::reassociate(&parse(lex(s).unwrap()).unwrap()).unwrap(),
                &Stmt::ExprStmt(expr!(op))
            ))
        }
    }
}
