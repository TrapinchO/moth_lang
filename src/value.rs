// TODO: kinda circular import, but it should be fine
use crate::reassoc::{Precedence, Associativity};

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    Function(fn(Vec<Value>)->Result<ValueType, String>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Value {
    pub typ: ValueType,
    pub start: usize,
    pub end: usize,
}

// TODO: this is very smart, as it can currently hold only Functions
// PIE anyone?
pub const BUILTINS: [(&str, Precedence, fn(Vec<Value>)->Result<ValueType, String>); 13] = [
    ("+",
     Precedence {prec: 5, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a + b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a + b),
            (ValueType::String(a), ValueType::String(b)) => ValueType::String(a.clone() + b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("-",
     Precedence {prec: 5, assoc: Associativity::Left },
     |args| {
        Ok(match &args[..] {
            [expr] => match &expr.typ {
                ValueType::Int(n) => ValueType::Int(-n),
                ValueType::Float(n) => ValueType::Float(-n),
                _ => return Err(format!("Invalid value: \"{:?}\"", expr)),
            },
            [left, right] => match (&left.typ, &right.typ) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a - b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a - b),
                _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right)),
            },
            _ => return Err(format!("Wrong number of arguments: {}", args.len()))
        })
    }),
    ("*", 
     Precedence {prec: 6, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a * b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a * b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("/", 
     Precedence {prec: 6, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a / b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a / b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("%", 
     Precedence {prec: 6, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a % b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a % b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),

    ("==", 
     Precedence {prec: 4, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a == b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a == b),
            (ValueType::String(a), ValueType::String(b)) => ValueType::Bool(a == b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a == b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("!=", 
     Precedence {prec: 4, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a != b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a != b),
            (ValueType::String(a), ValueType::String(b)) => ValueType::Bool(a != b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a == b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    (">=", 
     Precedence {prec: 4, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a >= b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a >= b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a >= b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("<=", 
     Precedence {prec: 4, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a <= b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a <= b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a <= b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    (">", 
     Precedence {prec: 4, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a > b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a > b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a > b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("<", 
     Precedence {prec: 4, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a < b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a < b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a < b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),


    ("||", 
     Precedence {prec: 4, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(*a || *b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("&&", 
     Precedence {prec: 4, assoc: Associativity::Left },
     |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(*a && *b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
];

