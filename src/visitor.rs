use crate::{error::Error, exprstmt::*};

pub trait StmtVisitor<T> {
    fn visit_stmt(&mut self, stmt: Stmt) -> Result<T, Error> {
        match stmt.val {
            StmtType::VarDeclStmt(..) => self.var_decl(stmt),
            StmtType::AssignStmt(..) => self.assignment(stmt),
            StmtType::ExprStmt(..) => self.expr(stmt),
            StmtType::BlockStmt(..) => self.block(stmt),
            StmtType::IfStmt(..) => self.if_else(stmt),
            StmtType::WhileStmt(..) => self.whiles(stmt),
            StmtType::FunDeclStmt(..) => self.fun(stmt),
        }
    }

    fn var_decl(&mut self, stmt: Stmt) -> Result<T, Error>;
    fn assignment(&mut self, stmt: Stmt) -> Result<T, Error>;
    fn expr(&mut self, expr: Stmt) -> Result<T, Error>;
    fn block(&mut self, stmt: Stmt) -> Result<T, Error>;
    fn if_else(&mut self, stmt: Stmt) -> Result<T, Error>;
    fn whiles(&mut self, stmt: Stmt) -> Result<T, Error>;
    fn fun(&mut self, stmt: Stmt) -> Result<T, Error>;
}

pub trait ExprVisitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> Result<T, Error> {
        match &expr.val {
            ExprType::Int(..) => self.int(expr),
            ExprType::Float(..) => self.float(expr),
            ExprType::String(..) => self.string(expr),
            ExprType::Bool(..) => self.bool(expr),
            ExprType::Identifier(..) => self.identifier(expr),
            ExprType::Parens(..) => self.parens(expr),
            ExprType::Call(..) => self.call(expr),
            ExprType::UnaryOperation(..) => self.unary(expr),
            ExprType::BinaryOperation(..) => self.binary(expr),
        }
    }
    fn int(&mut self, expr: &Expr) -> Result<T, Error>;
    fn float(&mut self, expr: &Expr) -> Result<T, Error>;
    fn string(&mut self, expr: &Expr) -> Result<T, Error>;
    fn bool(&mut self, expr: &Expr) -> Result<T, Error>;
    fn identifier(&mut self, expr: &Expr) -> Result<T, Error>;
    fn parens(&mut self, expr: &Expr) -> Result<T, Error>;
    fn call(&mut self, expr: &Expr) -> Result<T, Error>;
    fn unary(&mut self, expr: &Expr) -> Result<T, Error>;
    fn binary(&mut self, expr: &Expr) -> Result<T, Error>;
}
