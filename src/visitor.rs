use crate::error::Error;
use crate::exprstmt::*;
use crate::token::Token;

pub trait StmtVisitor<T> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<T, Error> {
        match &stmt.val {
            StmtType::VarDeclStmt(ident, expr) => self.var_decl(ident, expr),
            StmtType::AssignStmt(name, expr) => self.assignment(name, expr),
            StmtType::ExprStmt(expr) => self.expr(expr),
            StmtType::IfStmt(blocks) => self.if_else(blocks),
            StmtType::WhileStmt(cond, block) => self.whiles(cond, block),
        }
    }

    fn var_decl(&mut self, ident: &Token, expr: &Expr) -> Result<T, Error>;
    fn assignment(&mut self, ident: &Token, expr: &Expr) -> Result<T, Error>;
    fn expr(&mut self, expr: &Expr) -> Result<T, Error>;
    fn if_else(&mut self, blocks: &Vec<(Expr, Vec<Stmt>)>) -> Result<T, Error>;
    fn whiles(&mut self, cond: &Expr, block: &Vec<Stmt>) -> Result<T, Error>;
}

pub trait ExprVisitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> Result<T, Error> {
        match &expr.val {
            ExprType::Int(_) => self.int(expr),
            ExprType::Float(_) => self.float(expr),
            ExprType::String(_) => self.string(expr),
            ExprType::Bool(_) => self.bool(expr),
            ExprType::Identifier(_) => self.identifier(expr),
            ExprType::Parens(new_expr) => self.parens(new_expr),
            ExprType::UnaryOperation(op, new_expr) => self.unary(op, new_expr),
            ExprType::BinaryOperation(left, op, right) => self.binary(left, op, right),
        }
    }
    fn int(&mut self, expr: &Expr) -> Result<T, Error>;
    fn float(&mut self, expr: &Expr) -> Result<T, Error>;
    fn string(&mut self, expr: &Expr) -> Result<T, Error>;
    fn bool(&mut self, expr: &Expr) -> Result<T, Error>;
    fn identifier(&mut self, expr: &Expr) -> Result<T, Error>;
    fn parens(&mut self, expr: &Expr) -> Result<T, Error>;
    fn unary(&mut self, op: &Token, expr: &Expr) -> Result<T, Error>;
    fn binary(&mut self, left: &Expr, op: &Token, right: &Expr) -> Result<T, Error>;
}
