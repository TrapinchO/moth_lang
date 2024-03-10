use crate::{error::Error, exprstmt::*, token::Token, located::Location};

pub trait StmtVisitor<T> {
    fn visit_stmt(&mut self, stmt: Stmt) -> Result<T, Error> {
        let loc = stmt.loc;
        match stmt.val {
            StmtType::ExprStmt(expr) => self.expr(loc, expr),
            StmtType::VarDeclStmt(ident, expr) => self.var_decl(loc, ident, expr),
            StmtType::AssignStmt(ident, expr) => self.assignment(loc, ident, expr),
            StmtType::BlockStmt(block) => self.block(loc, block),
            StmtType::IfStmt(blocks) => self.if_else(loc, blocks),
            StmtType::WhileStmt(cond, block) => self.whiles(loc, cond, block),
            StmtType::FunDeclStmt(name, params, block) => self.fun(loc, name, params, block),
            StmtType::ReturnStmt(expr) => self.retur(loc, expr),
            StmtType::BreakStmt => self.brek(loc),
            StmtType::ContinueStmt => self.cont(loc),
        }
    }

    fn expr(&mut self, loc: Location, expr: Expr) -> Result<T, Error>;
    fn var_decl(&mut self, loc: Location, ident: Token, expr: Expr) -> Result<T, Error>;
    fn assignment(&mut self, loc: Location, ident: Token, expr: Expr) -> Result<T, Error>;
    fn block(&mut self, loc: Location, block: Vec<Stmt>) -> Result<T, Error>;
    fn if_else(&mut self, loc: Location, blocks: Vec<(Expr, Vec<Stmt>)>) -> Result<T, Error>;
    fn whiles(&mut self, loc: Location, cond: Expr, block: Vec<Stmt>) -> Result<T, Error>;
    fn fun(&mut self, loc: Location, name: Token, params: Vec<Token>, block: Vec<Stmt>) -> Result<T, Error>;
    fn cont(&mut self, loc: Location) -> Result<T, Error>;
    fn brek(&mut self, loc: Location) -> Result<T, Error>;
    fn retur(&mut self, loc: Location, expr: Expr) -> Result<T, Error>;
}

pub trait ExprVisitor<T> {
    fn visit_expr(&mut self, expr: Expr) -> Result<T, Error> {
        let loc = expr.loc;
        match expr.val {
            ExprType::Unit => self.unit(loc),
            ExprType::Int(n) => self.int(loc, n),
            ExprType::Float(n) => self.float(loc, n),
            ExprType::String(s) => self.string(loc, s),
            ExprType::Bool(b) => self.bool(loc, b),
            ExprType::Identifier(ident) => self.identifier(loc, ident),
            ExprType::Parens(expr1) => self.parens(loc, *expr1),
            ExprType::Call(callee, args) => self.call(loc, *callee, args),
            ExprType::UnaryOperation(op, expr1) => self.unary(loc, op, *expr1),
            ExprType::BinaryOperation(left, op, right) => self.binary(loc, *left, op, *right),
        }
    }
    fn unit(&mut self, loc: Location) -> Result<T, Error>;
    fn int(&mut self, loc: Location, n: i32) -> Result<T, Error>;
    fn float(&mut self, loc: Location, n: f32) -> Result<T, Error>;
    fn string(&mut self, loc: Location, s: String) -> Result<T, Error>;
    fn bool(&mut self, loc: Location, b: bool) -> Result<T, Error>;
    fn identifier(&mut self, loc: Location, ident: String) -> Result<T, Error>;
    fn parens(&mut self, loc: Location, expr: Expr) -> Result<T, Error>;
    fn call(&mut self, loc: Location, callee: Expr, args: Vec<Expr>) -> Result<T, Error>;
    fn unary(&mut self, loc: Location, op: Token, expr: Expr) -> Result<T, Error>;
    fn binary(&mut self, loc: Location, left: Expr, op: Token, right: Expr) -> Result<T, Error>;
}
