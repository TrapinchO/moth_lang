use moth_lang::exprstmt::*;

macro_rules! binop {
    ($left:expr, $op:tt, $right:expr) => {
        ExprType::BinaryOperation(
            Expr {
                val: $left.into(),
                start: 0,
                end: 0,
            }.into(),
            Token {
                val: TokenType::Symbol($op.to_string()),
                start: 0,
                end: 0,
            },
            Expr {
                val: $right.into(),
                start: 0,
                end: 0,
            }.into(),
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
            }.into()
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
            }.into()
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
        (StmtType::ExprStmt(expr1), StmtType::ExprStmt(expr2)) => {
            compare_elements_expr(&expr1, &expr2)
        },
        (StmtType::VarDeclStmt(ident1, expr1), StmtType::VarDeclStmt(ident2, expr2)) => {
            ident1 == ident2 && compare_elements_expr(expr1, expr2)
        },
        (StmtType::AssignStmt(ident1, expr1), StmtType::AssignStmt(ident2, expr2)) => {
            ident1 == ident2 && compare_elements_expr(expr1, expr2)
        }
        (s1, s2) => s1 == s2
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
    use moth_lang::error::Error;
    use moth_lang::lexer::lex;
    use moth_lang::token::{Token, TokenType};
    use moth_lang::parser::parse;
    use moth_lang::exprstmt::*;
    use moth_lang::reassoc;
    use moth_lang::value::BUILTINS;

    use crate::compare_elements;

    #[test]
    fn empty() {
        assert_eq!(
            parse(vec![Token { start: 0, end: 0, val: TokenType::Eof }]),
            Ok(vec![])
        );
        assert_eq!(
            parse(vec![]),
            Ok(vec![])
        );
    }

    #[test]
    fn parse_primary() {
        let ops = [
            ("1", expr!(ExprType::Int(1))),
            ("1234", expr!(ExprType::Int(1234))),
            ("\"\"", expr!(ExprType::String("".to_string()))),
            ("\"test\"", expr!(ExprType::String("test".to_string()))),
            ("(1)", expr!(ExprType::Parens(expr!(ExprType::Int(1)).into()))),
            ("(1 + 1)", expr!(parenop!(binop!(ExprType::Int(1), "+", ExprType::Int(1))))),
        ];
        for (s, op) in ops {
            assert!(compare_elements(
                    &parse(lex(&(s.to_owned()+";")).unwrap()).unwrap()[0],
                    &stmt!(StmtType::ExprStmt(op))
            ));
        }
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
    fn parens_missing() {
        let err = Error {
            msg: "Expected closing parenthesis".to_string(),
            lines: vec![(2, 2)],
        };
        let op = parse(lex("(1").unwrap()).unwrap_err();
        assert_eq!(err, op);
    }

    #[test]
    fn parens_empty() {
        let err = Error {
            msg: "Unknown element: RParen".to_string(),
            lines: vec![(1, 1)],
        };
        let op = parse(lex("()").unwrap()).unwrap_err();
        assert_eq!(err, op);
    }

    #[test]
    fn parse_binary() {
        let ops = [
            ("1+1", binop!(ExprType::Int(1), "+", ExprType::Int(1))),
            ("1 + 1", binop!(ExprType::Int(1), "+", ExprType::Int(1))),
            ("1-1", binop!(ExprType::Int(1), "-", ExprType::Int(1))),
            ("1**1", binop!(ExprType::Int(1), "**", ExprType::Int(1))),

                ("1+1+1", binop!(ExprType::Int(1), "+", binop!(ExprType::Int(1), "+", ExprType::Int(1)))),
            ];
            for (s, op) in ops {
                assert!(compare_elements(
                    &parse(lex(&(s.to_owned()+";")).unwrap()).unwrap()[0],
                    &stmt!(StmtType::ExprStmt(expr!(op)))
                ));
            }
        }

    #[test]
    fn parse_unary() {
        let ops = [
            ("+1", unop!("+", ExprType::Int(1))),
            ("* - +1", unop!("*", unop!("-", unop!("+", ExprType::Int(1))))),
            ("*-+1", unop!("*-+", ExprType::Int(1))),
        ];
        for (s, op) in ops {
            assert!(compare_elements(
                &parse(lex(&(s.to_owned()+";")).unwrap()).unwrap()[0],
                &stmt!(StmtType::ExprStmt(expr!(op)))
            ));
        }
    }

    #[test]
    fn test_reassoc() {
        let ops = [
            ("1 - 1 - 1", binop!(
                    binop!(ExprType::Int(1), "-", ExprType::Int(1)),
                    "-",
                    ExprType::Int(1))),
            ("+(1 - 1 - 1)", unop!("+", parenop!(binop!(
                    binop!(ExprType::Int(1), "-", ExprType::Int(1)),
                "-",
                ExprType::Int(1))))),
        ];
        let symbols: std::collections::HashMap<String, reassoc::Precedence> = BUILTINS.map(|(name, assoc, _)| (name.to_string(), assoc)).into();
        for (s, op) in ops {
            assert!(compare_elements(
                &reassoc::reassociate(symbols.clone(), &parse(lex(&(s.to_owned()+";")).unwrap()).unwrap()).unwrap()[0],
                &stmt!(StmtType::ExprStmt(expr!(op)))
            ));
        }
    }
}
