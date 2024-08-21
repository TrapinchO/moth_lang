use crate::{associativity::Precedence, error::Error, exprstmt::*, located::Location};

pub trait StmtVisitor<T> {
    fn visit_stmt(&mut self, stmt: Stmt) -> Result<T, Error> {
        let loc = stmt.loc;
        match stmt.val {
            StmtType::Expr(expr) => self.expr(loc, expr),
            StmtType::VarDecl(ident, expr) => self.var_decl(loc, ident, expr),
            StmtType::Assign(ident, expr) => self.assignment(loc, ident, expr),
            StmtType::AssignIndex(ls, idx, val) => self.assignindex(loc, ls, idx, val),
            StmtType::Block(block) => self.block(loc, block),
            StmtType::If(blocks) => self.if_else(loc, blocks),
            StmtType::While(cond, block) => self.whiles(loc, cond, block),
            StmtType::FunDecl(name, params, block) => self.fun(loc, name, params, block),
            StmtType::OperatorDecl(name, params, block, prec) => self.operator(loc, name, params, block, prec),
            StmtType::Return(expr) => self.retur(loc, expr),
            StmtType::Break => self.brek(loc),
            StmtType::Continue => self.cont(loc),
            StmtType::Struct(name, fields) => self.struc(loc, name, fields),
            StmtType::AssignStruct(expr1, name, expr2) => self.assignstruc(loc, expr1, name, expr2),
            StmtType::Impl(name, block) => self.imp(loc, name, block),
        }
    }

    fn expr(&mut self, loc: Location, expr: Expr) -> Result<T, Error>;
    fn var_decl(&mut self, loc: Location, ident: Identifier, expr: Expr) -> Result<T, Error>;
    fn assignment(&mut self, loc: Location, ident: Identifier, expr: Expr) -> Result<T, Error>;
    fn assignindex(&mut self, loc: Location, ls: Expr, idx: Expr, val: Expr) -> Result<T, Error>;
    fn block(&mut self, loc: Location, block: Vec<Stmt>) -> Result<T, Error>;
    fn if_else(&mut self, loc: Location, blocks: Vec<(Expr, Vec<Stmt>)>) -> Result<T, Error>;
    fn whiles(&mut self, loc: Location, cond: Expr, block: Vec<Stmt>) -> Result<T, Error>;
    fn fun(&mut self, loc: Location, name: Identifier, params: Vec<Identifier>, block: Vec<Stmt>) -> Result<T, Error>;
    fn operator(
        &mut self,
        loc: Location,
        name: Symbol,
        params: (Identifier, Identifier),
        block: Vec<Stmt>,
        prec: Precedence,
    ) -> Result<T, Error>;
    fn cont(&mut self, loc: Location) -> Result<T, Error>;
    fn brek(&mut self, loc: Location) -> Result<T, Error>;
    fn retur(&mut self, loc: Location, expr: Expr) -> Result<T, Error>;
    fn struc(&mut self, loc: Location, name: Identifier, fields: Vec<Identifier>) -> Result<T, Error>;
    fn assignstruc(&mut self, loc: Location, expr1: Expr, name: Identifier, expr2: Expr) -> Result<T, Error>;
    fn imp(&mut self, loc: Location, name: Identifier, block: Vec<Stmt>) -> Result<T, Error>;
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
            ExprType::List(ls) => self.list(loc, ls),
            ExprType::Index(expr2, idx) => self.index(loc, *expr2, *idx),
            ExprType::Lambda(params, body) => self.lambda(loc, params, body),
            ExprType::FieldAccess(expr, name) => self.field(loc, *expr, name),
            ExprType::MethodAccess(expr, name, args) => self.method(loc, *expr, name, args),
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
    fn unary(&mut self, loc: Location, op: Symbol, expr: Expr) -> Result<T, Error>;
    fn binary(&mut self, loc: Location, left: Expr, op: Symbol, right: Expr) -> Result<T, Error>;
    fn list(&mut self, loc: Location, expr: Vec<Expr>) -> Result<T, Error>;
    fn index(&mut self, loc: Location, expr2: Expr, idx: Expr) -> Result<T, Error>;
    fn lambda(&mut self, loc: Location, params: Vec<Identifier>, body: Vec<Stmt>) -> Result<T, Error>;
    fn field(&mut self, loc: Location, expr: Expr, name: Identifier) -> Result<T, Error>;
    fn method(&mut self, loc: Location, callee: Expr, name: Identifier, args: Vec<Expr>) -> Result<T, Error>;
}
