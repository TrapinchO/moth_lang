#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Number(i32),
    String(String),
    UnaryOperation(String, Box<Expr>),
    BinaryOperation(String, Box<Expr>, Box<Expr>),
}
