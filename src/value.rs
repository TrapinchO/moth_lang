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

pub const BUILTINS: [(&str, fn(Vec<Value>)->Result<ValueType, String>); 13] = [
    ("+", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a + b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a + b),
            (ValueType::String(a), ValueType::String(b)) => ValueType::String(a.clone() + b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("-", |args| {
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
    ("*", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a * b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a * b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("/", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a / b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a / b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("%", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a % b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a % b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),

    ("==", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a == b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a == b),
            (ValueType::String(a), ValueType::String(b)) => ValueType::Bool(a == b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a == b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("!=", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a != b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a != b),
            (ValueType::String(a), ValueType::String(b)) => ValueType::Bool(a != b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a == b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    (">=", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a >= b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a >= b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a >= b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("<=", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a <= b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a <= b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a <= b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    (">", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a > b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a > b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a > b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("<", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a < b),
            (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a < b),
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a < b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),


    ("||", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(*a || *b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
    ("&&", |args| {
        let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
        Ok(match (&left.typ, &right.typ) {
            (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(*a && *b),
            _ => return Err(format!("Invalid values: \"{:?}\" and \"{:?}\"", left, right))
        })
    }),
];

