use std::fmt::Display;

// TODO: kinda circular import, but it should be fine
use crate::{reassoc::{Associativity, Precedence}, located::Located};

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    Function(fn(Vec<Value>) -> Result<ValueType, String>),
}
impl ValueType {
    fn format(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::Float(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Function(_) => "<function>".to_string(),  // TODO: improve
        }
    }
}
impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

pub type Value = Located<ValueType>;

// TODO: this is very smart, as it can currently hold only Functions
// PIE anyone?
pub const BUILTINS: [(
    &str,
    Precedence,
    fn(Vec<Value>) -> Result<ValueType, String>,
); 14] = [
    (
        "+",
        Precedence {
            prec: 5,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a + b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a + b),
                (ValueType::String(a), ValueType::String(b)) => ValueType::String(a.clone() + b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        "-",
        Precedence {
            prec: 5,
            assoc: Associativity::Left,
        },
        |args| {
            Ok(match &args[..] {
                [expr] => match &expr.val {
                    ValueType::Int(n) => ValueType::Int(-n),
                    ValueType::Float(n) => ValueType::Float(-n),
                    _ => return Err(format!("Invalid value: \"{}\"", expr.val)),
                },
                [left, right] => match (&left.val, &right.val) {
                    (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a - b),
                    (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a - b),
                    _ => {
                        return Err(format!(
                            "Invalid values: \"{}\" and \"{}\"",
                            left.val, right.val
                        ))
                    }
                },
                _ => return Err(format!("Wrong number of arguments: {}", args.len())),
            })
        },
    ),
    (
        "*",
        Precedence {
            prec: 6,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a * b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a * b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        "/",
        Precedence {
            prec: 6,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => {
                    if *b == 0 {
                        return Err("Attempted division by zero".to_string());
                    }
                    ValueType::Int(a / b)
                },
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a / b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        "%",
        Precedence {
            prec: 6,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a % b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a % b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        "==",
        Precedence {
            prec: 4,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a == b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a == b),
                (ValueType::String(a), ValueType::String(b)) => ValueType::Bool(a == b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a == b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        "!=",
        Precedence {
            prec: 4,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a != b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a != b),
                (ValueType::String(a), ValueType::String(b)) => ValueType::Bool(a != b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a == b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        ">=",
        Precedence {
            prec: 4,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a >= b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a >= b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a >= b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        "<=",
        Precedence {
            prec: 4,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a <= b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a <= b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a <= b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        ">",
        Precedence {
            prec: 4,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a > b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a > b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a > b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        "<",
        Precedence {
            prec: 4,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a < b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a < b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a < b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        "!",
        Precedence {
            prec: 0,
            assoc: Associativity::Left,
        },
        |args| {
            let [expr] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())); };
            Ok(match &expr.val {
                ValueType::Bool(val) => ValueType::Bool(! *val),
                _ => return Err(format!("Invalid value: \"{}\"", expr.val))
            })
        },
    ),
    (
        "||",
        Precedence {
            prec: 4,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(*a || *b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
    (
        "&&",
        Precedence {
            prec: 4,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else { return Err(format!("Wrong number of arguments: {}", args.len())) };
            Ok(match (&left.val, &right.val) {
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(*a && *b),
                _ => {
                    return Err(format!(
                        "Invalid values: \"{}\" and \"{}\"",
                        left.val, right.val
                    ))
                }
            })
        },
    ),
];
