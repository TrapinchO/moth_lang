use crate::{associativity::Precedence, error::Error, exprstmt::*, located::Location};

pub trait StmtVisitor<T> {
    fn visit_stmt(&mut self, stmt: LStmt) -> Result<T, Error> {
        let loc = stmt.loc;
        match stmt.val {
            Stmt::Expr(expr) => self.expr(loc, expr),
            Stmt::VarDecl(ident, expr) => self.var_decl(loc, ident, expr),
            Stmt::Assign(ident, expr) => self.assignment(loc, ident, expr),
            Stmt::AssignIndex(ls, idx, val) => self.assignindex(loc, ls, idx, val),
            Stmt::Block(block) => self.block(loc, block),
            Stmt::If(blocks, els) => self.if_else(loc, blocks, els),
            Stmt::While(cond, block) => self.whiles(loc, cond, block),
            Stmt::FunDecl(name, params, block) => self.fun(loc, name, params, block),
            Stmt::OperatorDecl(name, params, block, prec) => self.operator(loc, name, params, block, prec),
            Stmt::Return(expr) => self.retur(loc, expr),
            Stmt::Break => self.brek(loc),
            Stmt::Continue => self.cont(loc),
            Stmt::Struct(name, fields) => self.struc(loc, name, fields),
            Stmt::AssignStruct(expr1, name, expr2) => self.assignstruc(loc, expr1, name, expr2),
            Stmt::Impl(name, block) => self.imp(loc, name, block),
        }
    }

    fn expr(&mut self, loc: Location, expr: LExpr) -> Result<T, Error>;
    fn var_decl(&mut self, loc: Location, ident: Identifier, expr: LExpr) -> Result<T, Error>;
    fn assignment(&mut self, loc: Location, ident: Identifier, expr: LExpr) -> Result<T, Error>;
    fn assignindex(&mut self, loc: Location, ls: LExpr, idx: LExpr, val: LExpr) -> Result<T, Error>;
    fn block(&mut self, loc: Location, block: Vec<LStmt>) -> Result<T, Error>;
    fn if_else(&mut self, loc: Location, blocks: Vec<(LExpr, Vec<LStmt>)>, els: Option<Block>) -> Result<T, Error>;
    fn whiles(&mut self, loc: Location, cond: LExpr, block: Vec<LStmt>) -> Result<T, Error>;
    fn fun(&mut self, loc: Location, name: Identifier, params: Vec<Identifier>, block: Vec<LStmt>) -> Result<T, Error>;
    fn operator(
        &mut self,
        loc: Location,
        name: Symbol,
        params: (Identifier, Identifier),
        block: Vec<LStmt>,
        prec: Precedence,
    ) -> Result<T, Error>;
    fn cont(&mut self, loc: Location) -> Result<T, Error>;
    fn brek(&mut self, loc: Location) -> Result<T, Error>;
    fn retur(&mut self, loc: Location, expr: LExpr) -> Result<T, Error>;
    fn struc(&mut self, loc: Location, name: Identifier, fields: Vec<Identifier>) -> Result<T, Error>;
    fn assignstruc(&mut self, loc: Location, expr1: LExpr, name: Identifier, expr2: LExpr) -> Result<T, Error>;
    fn imp(&mut self, loc: Location, name: Identifier, block: Vec<LStmt>) -> Result<T, Error>;
}

pub trait ExprVisitor<T> {
    fn visit_expr(&mut self, expr: LExpr) -> Result<T, Error> {
        let loc = expr.loc;
        match expr.val {
            Expr::Unit => self.unit(loc),
            Expr::Int(n) => self.int(loc, n),
            Expr::Float(n) => self.float(loc, n),
            Expr::String(s) => self.string(loc, s),
            Expr::Bool(b) => self.bool(loc, b),
            Expr::Identifier(ident) => self.identifier(loc, ident),
            Expr::Parens(expr1) => self.parens(loc, *expr1),
            Expr::Call(callee, args) => self.call(loc, *callee, args),
            Expr::UnaryOperation(op, expr1) => self.unary(loc, op, *expr1),
            Expr::BinaryOperation(left, op, right) => self.binary(loc, *left, op, *right),
            Expr::List(ls) => self.list(loc, ls),
            Expr::Index(expr2, idx) => self.index(loc, *expr2, *idx),
            Expr::Lambda(params, body) => self.lambda(loc, params, body),
            Expr::FieldAccess(expr, name) => self.field(loc, *expr, name),
            Expr::MethodAccess(expr, name, args) => self.method(loc, *expr, name, args),
        }
    }
    fn unit(&mut self, loc: Location) -> Result<T, Error>;
    fn int(&mut self, loc: Location, n: i32) -> Result<T, Error>;
    fn float(&mut self, loc: Location, n: f32) -> Result<T, Error>;
    fn string(&mut self, loc: Location, s: String) -> Result<T, Error>;
    fn bool(&mut self, loc: Location, b: bool) -> Result<T, Error>;
    fn identifier(&mut self, loc: Location, ident: String) -> Result<T, Error>;
    fn parens(&mut self, loc: Location, expr: LExpr) -> Result<T, Error>;
    fn call(&mut self, loc: Location, callee: LExpr, args: Vec<LExpr>) -> Result<T, Error>;
    fn unary(&mut self, loc: Location, op: Symbol, expr: LExpr) -> Result<T, Error>;
    fn binary(&mut self, loc: Location, left: LExpr, op: Symbol, right: LExpr) -> Result<T, Error>;
    fn list(&mut self, loc: Location, expr: Vec<LExpr>) -> Result<T, Error>;
    fn index(&mut self, loc: Location, expr2: LExpr, idx: LExpr) -> Result<T, Error>;
    fn lambda(&mut self, loc: Location, params: Vec<Identifier>, body: Vec<LStmt>) -> Result<T, Error>;
    fn field(&mut self, loc: Location, expr: LExpr, name: Identifier) -> Result<T, Error>;
    fn method(&mut self, loc: Location, callee: LExpr, name: Identifier, args: Vec<LExpr>) -> Result<T, Error>;
}
