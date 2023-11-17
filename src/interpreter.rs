use std::collections::HashMap;

use crate::{
    error::Error,
    exprstmt::{Expr, ExprType, Stmt, StmtType},
    token::*,
    value::*,
    visitor::{ExprVisitor, StmtVisitor},
};

#[derive(Debug, PartialEq, Clone)]
struct Environment {
    scopes: Vec<HashMap<String, ValueType>>,
}

impl Environment {
    pub fn new(defaults: HashMap<String, ValueType>) -> Environment {
        Environment { scopes: vec![defaults] }
    }

    pub fn insert(&mut self, ident: &Token, val: Value) -> Result<(), Error> {
        let TokenType::Identifier(name) = &ident.val else {
            unreachable!()
        };
        let last_scope = self.scopes.last_mut().unwrap();
        if last_scope.contains_key(name) {
            return Err(Error {
                msg: format!("Name \"{}\" already exists", name),
                lines: vec![(ident.start, ident.end)],
            });
        }
        last_scope.insert(name.clone(), val.val);
        Ok(())
    }

    pub fn get(&self, ident: &String, pos: (usize, usize)) -> Result<ValueType, Error> {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(ident) {
                return Ok(scope.get(ident).unwrap().clone())
            }
        }
        Err(Error {
            msg: format!("Name not found: \"{}\"", ident),
            lines: vec![pos],
        })
    }

    pub fn update(&mut self, ident: &Token, val: Value) -> Result<(), Error> {
        let TokenType::Identifier(name) = &ident.val else {
            unreachable!()
        };
        let last_scope = self.scopes.iter_mut().last().unwrap();
        *last_scope.get_mut(name).unwrap() = val.val;
        Ok(())
    }

    fn add_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn remove_scope(&mut self) {
        self.scopes.pop();
    }
}

pub fn interpret(stmts: &Vec<Stmt>) -> Result<(), Error> {
    // TODO: solve positions for builtin stuff
    let defaults = HashMap::from(BUILTINS.map(|(name, _, f)| (name.to_string(), ValueType::Function(f))));
    Interpreter::new(defaults).interpret(stmts)
}

// TODO: just for repl, consider redoing
pub struct Interpreter {
    environment: Environment,
}

// TODO: why do I even need Value and not just ValueType?
impl Interpreter {
    pub fn new(defaults: HashMap<String, ValueType>) -> Self {
        Interpreter {
            environment: Environment::new(defaults),
        }
    }

    pub fn interpret(&mut self, stmts: &Vec<Stmt>) -> Result<(), Error> {
        for s in stmts {
            self.visit_stmt(s)?;
        }
        Ok(())
    }
    fn add_scope(&mut self) {
        self.environment.add_scope()
    }
    fn remove_scope(&mut self) {
        self.environment.remove_scope();
    }
}

impl StmtVisitor<()> for Interpreter {
    fn var_decl(&mut self, stmt: &Stmt) -> Result<(), Error> {
        let StmtType::VarDeclStmt(ident, expr) = &stmt.val else {
            unreachable!()
        };
        let val = self.visit_expr(expr)?;
        self.environment.insert(ident, val)?;
        Ok(())
    }

    fn assignment(&mut self, stmt: &Stmt) -> Result<(), Error> {
        let StmtType::AssignStmt(ident, expr) = &stmt.val else {
            unreachable!()
        };
        let val = self.visit_expr(expr)?;
        self.environment.update(ident, val)?;
        Ok(())
    }

    fn block(&mut self, stmt: &Stmt) -> Result<(), Error> {
        let StmtType::BlockStmt(block) = &stmt.val else {
            unreachable!()
        };
        self.add_scope();
        for s in block {
            self.visit_stmt(s)?;
        }
        self.remove_scope();
        Ok(())
    }

    fn if_else(&mut self, stmt: &Stmt) -> Result<(), Error> {
        let StmtType::IfStmt(blocks) = &stmt.val else {
            unreachable!()
        };
        for (cond, block) in blocks {
            let ValueType::Bool(cond2) = self.visit_expr(cond)?.val else {
                return Err(Error {
                    msg: format!("Expected bool, got {}", cond.val),
                    lines: vec![(cond.start, cond.end)],
                });
            };
            // do not continue
            if cond2 {
                self.add_scope();
                for s in block {
                    self.visit_stmt(s)?;
                }
                self.remove_scope();
                break;
            }
        }

        Ok(())
    }

    // TODO: make like if_else
    fn whiles(&mut self, stmt: &Stmt) -> Result<(), Error> {
        let StmtType::WhileStmt(cond, block) = &stmt.val else {
            unreachable!()
        };
        while let ValueType::Bool(true) = self.visit_expr(cond)?.val {
            self.add_scope();
            for s in block {
                self.visit_stmt(s)?;
            }
            self.remove_scope();
        }
        Ok(())
    }

    fn expr(&mut self, stmt: &Stmt) -> Result<(), Error> {
        let StmtType::ExprStmt(expr) = &stmt.val else {
            unreachable!()
        };
        let val = self.visit_expr(expr)?;
        //println!("{:?}", val.val);
        Ok(())
    }

    fn print(&mut self, stmt: &Stmt) -> Result<(), Error> {
        let StmtType::PrintStmt(expr) = &stmt.val else {
            unreachable!()
        };
        let val = self.visit_expr(expr)?;
        println!("{:?}", val.val);
        Ok(())
    }
}

impl ExprVisitor<Value> for Interpreter {
    fn int(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::Int(n) = &expr.val.clone() else {
            unreachable!()
        };
        Ok(Value {
            val: ValueType::Int(*n),
            start: expr.start,
            end: expr.end,
        })
    }
    fn float(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::Float(f) = &expr.val.clone() else {
            unreachable!()
        };
        Ok(Value {
            val: ValueType::Float(*f),
            start: expr.start,
            end: expr.end,
        })
    }
    fn string(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::String(s) = &expr.val.clone() else {
            unreachable!()
        };
        Ok(Value {
            val: ValueType::String(s.clone()),
            start: expr.start,
            end: expr.end,
        })
    }
    fn identifier(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::Identifier(name) = &expr.val.clone() else {
            unreachable!()
        };
        Ok(Value {
            val: self.environment.get(name, (expr.start, expr.end))?,
            start: expr.start,
            end: expr.end,
        })
    }
    fn bool(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::Bool(b) = &expr.val.clone() else {
            unreachable!()
        };
        Ok(Value {
            val: ValueType::Bool(*b),
            start: expr.start,
            end: expr.end,
        })
    }
    fn parens(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::Parens(expr2) = &expr.val else {
            unreachable!()
        };
        Ok(Value {
            start: expr.start,
            end: expr.end,
            val: self.visit_expr(expr2)?.val,
        })
    }
    fn unary(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::UnaryOperation(op, expr2) = &expr.val else {
            unreachable!()
        };
        let val = self.visit_expr(expr2)?;
        let TokenType::Symbol(op_name) = &op.val else {
            panic!("Expected a symbol, found {}", op.val);
        };
        let ValueType::Function(func) = self.environment.get(op_name, (op.start, op.end))? else {
            return Err(Error {
                msg: format!("Symbol\"{}\" is not a function", op_name),
                lines: vec![(op.start, op.end)],
            });
        };
        Ok(Value {
            val: func(vec![val]).map_err(|msg| Error {
                msg,
                lines: vec![(expr.start, expr.end)],
            })?,
            start: expr.start,
            end: expr.end,
        })
    }
    fn binary(&mut self, expr: &Expr) -> Result<Value, Error> {
        let ExprType::BinaryOperation(left, op, right) = &expr.val else {
            unreachable!()
        };
        let left2 = self.visit_expr(left)?;
        let right2 = self.visit_expr(right)?;
        let TokenType::Symbol(op_name) = &op.val else {
            panic!("Expected a symbol, found {}", op.val)
        };
        let ValueType::Function(func) = self.environment.get(op_name, (op.start, op.end))? else {
            return Err(Error {
                msg: format!("Symbol\"{}\" is not a function", op_name),
                lines: vec![(op.start, op.end)],
            });
        };
        Ok(Value {
            val: func(vec![left2, right2]).map_err(|msg| Error {
                msg,
                lines: vec![(left.start, right.end)],
            })?,
            start: left.start,
            end: right.end,
        })
    }
}
