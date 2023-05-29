use crate::error::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    Function(fn(Vec<Value>)->Result<Value, Error>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Value {
    pub typ: ValueType,
    pub start: usize,
    pub end: usize,
}

pub const BUILTINS: [(&str, fn(Vec<Value>)->Result<Value, Error>); 4] = [
    ("+", |args| {
        // TODO: add proper positions for the argument list
        let [left, right] = &args[..] else { return Err(Error { msg: format!("Wrong number of arguemtns {}", args.len()), lines: vec![(0, 0)] }) };
        let typ = match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a + b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a + b),
            (ValueType::String(a), ValueType::String(b)) => ValueType::String(a.clone() + b),
            _ => return Err(Error {
                msg: format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right),
                lines: vec![(left.start, right.end)]
            })
        };
        Ok(Value {
            typ,
            start: left.start,
            end: right.end,
        })
    }),
    ("-", |args| {
        let [left, right] = &args[..] else { return Err(Error { msg: format!("Wrong number of arguemtns {}", args.len()), lines: vec![(0, 0)] }) };
        Ok(Value {
            typ: match (&left.typ, &right.typ) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a - b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a - b),
                _ => return Err(Error {
                    msg: format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right),
                lines: vec![(left.start, right.end)]
                })
            },
            start: left.start,
            end: right.end,
        })
    }),
    ("*", |args| {
        let [left, right] = &args[..] else { return Err(Error { msg: format!("Wrong number of arguemtns {}", args.len()), lines: vec![(0, 0)] }) };
        Ok(Value {
            typ: match (&left.typ, &right.typ) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a * b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a * b),
                _ => return Err(Error {
                    msg: format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right),
                lines: vec![(left.start, right.end)]
                })
            },
            start: left.start,
            end: right.end,
        })
    }),
    ("/", |args| {
        let [left, right] = &args[..] else { return Err(Error { msg: format!("Wrong number of arguemtns {}", args.len()), lines: vec![(0, 0)] }) };
        Ok(Value {
            typ: match (&left.typ, &right.typ) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a / b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a / b),
                _ => return Err(Error {
                    msg: format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right),
                lines: vec![(left.start, right.end)]
                })
            },
            start: left.start,
            end: right.end,
        })
    }),
];

