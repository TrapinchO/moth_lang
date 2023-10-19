use crate::error::Error;
use crate::exprstmt::*;
use std::collections::HashMap;

struct VarCheck;

impl StmtVisitor<Stmt> for VarCheck {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<T, Error> {
        match &stmt.val {
            StmtType::VarDeclStmt(..) => self.var_decl(stmt),
            StmtType::AssignStmt(..) => self.assignment(stmt),
            StmtType::ExprStmt(..) => self.expr(stmt),
            StmtType::BlockStmt(..) => self.block(stmt),
            StmtType::IfStmt(..) => self.if_else(stmt),
            StmtType::WhileStmt(..) => self.whiles(stmt),
            StmtType::PrintStmt(..) => self.print(stmt),
        }
    }

    fn var_decl(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
    fn assignment(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
    fn expr(&mut self, expr: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
    fn block(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        let StmtType::BlockStmt(block) = &stmt.val else { unreachable!() };
        let mut vars = HashMap::new();
        for s in block {
            match &s.val {
                StmtType::VarDeclStmt(Token { val: TokenType::Identifier(name) }, _) => vars.insert(name),
                StmtType::AssignStmt(Token { val: TokenType::Identifier(name) }, _) => {
                    if !vars.contains_key(name) {
                        return Err(Error {
                            msg: "Unknown variable".to_string(),
                            lines: vec![],
                        });
                    }
                },
                s2 & StmtType::BlockStmt(..) => self.visit_stmt(s2),
                else => {}
            }
        }
        Ok(stmt)
    }
    fn if_else(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
    fn whiles(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
    fn print(&mut self, stmt: &Stmt) -> Result<Stmt, Error> {
        Ok(stmt)
    }
}

impl ExprVisitor<Expr> for VarCheck {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Expr, Error> {
        match &expr.val {
            ExprType::Int(..) => self.int(expr),
            ExprType::Float(..) => self.float(expr),
            ExprType::String(..) => self.string(expr),
            ExprType::Bool(..) => self.bool(expr),
            ExprType::Identifier(..) => self.identifier(expr),
            ExprType::Parens(..) => self.parens(expr),
            ExprType::UnaryOperation(..) => self.unary(expr),
            ExprType::BinaryOperation(..) => self.binary(expr),
        }
    }
    fn int(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn float(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn string(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn bool(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn identifier(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn parens(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn unary(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
    fn binary(&mut self, expr: &Expr) -> Result<Expr, Error> {
        Ok(expr)
    }
}
