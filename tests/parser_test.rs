use moth_lang::parser::{Expr, ExprType};

#[cfg(test)]
mod tests {
    use moth_lang::lexer::{Token, TokenType, lex};
    use moth_lang::parser::*;

    use crate::compare_elements;

    #[test]
    #[should_panic(expected = "Expected code to parse")]
    fn empty() {
        parse(vec![Token {start: 0, end: 0, line: 0, typ: TokenType::Eof }]).unwrap();
    }

    #[test]
    fn test() {
        assert!(compare_elements(
            &parse(lex("1+1").unwrap()).unwrap(),
            &Expr { start: 1, end: 1, line: 1, typ: ExprType::BinaryOperation(Expr {start:1, end:1, line: 1, typ: ExprType::Number(1)}.into(), Token { start: 1, end: 1, line: 1, typ: TokenType::Symbol("+".to_string()) }, Expr {start:1, end:1, line: 1, typ: ExprType::Number(1)}.into()) }
        ))
    }
}

fn compare_elements(left: &Expr, right: &Expr) -> bool {
    match (&left.typ, &right.typ) {
        (ExprType::BinaryOperation(l1, o1, r1), ExprType::BinaryOperation(l2, o2, r2)) => {
            compare_elements(&l1, &l2) && o1.typ == o2.typ && compare_elements(&r1, &r2)
        },
        (ExprType::UnaryOperation(o1, e1), ExprType::UnaryOperation(o2, e2)) => {
            o1.typ == o2.typ && compare_elements(&e1, &e2)
        },
        (ExprType::Parens(e1), ExprType::Parens(e2)) => compare_elements(&e1, &e2),
        (e1, e2) => e1 == e2,
    }
}
