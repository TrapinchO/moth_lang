use std::collections::HashMap;
use std::fmt::Display;

// TODO: kinda circular import, but it should be fine
use crate::located::Located;
use crate::reassoc::{Associativity, Precedence};

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    Function(fn(Vec<Value>) -> Result<ValueType, String>),
    Unit,
}
impl ValueType {
    fn format(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::Float(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::Function(_) => "<function>".to_string(), // TODO: improve
            Self::Unit => "()".to_string(),
        }
    }

    pub fn compare_variant(&self, other: &ValueType) -> bool {
        // probably courtesy of https://stackoverflow.com/a/32554326
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

pub type Value = Located<ValueType>;

// TODO: this is very smart, as it can currently hold only Functions
// TODO: fix the precedence mess
// PIE anyone?
type NativeFunction = fn(Vec<Value>) -> Result<ValueType, String>;
pub const NATIVE_OPERATORS: [(&str, Precedence, NativeFunction); 14] = [
    (
        "+",
        Precedence {
            prec: 5,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a + b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a + b),
                (ValueType::String(a), ValueType::String(b)) => ValueType::String(a.clone() + b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
                    _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a * b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a * b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => {
                    if *b == 0 {
                        return Err("Attempted division by zero".to_string());
                    }
                    ValueType::Int(a / b)
                }
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a / b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a % b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a % b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a == b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a == b),
                (ValueType::String(a), ValueType::String(b)) => ValueType::Bool(a == b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a == b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a != b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a != b),
                (ValueType::String(a), ValueType::String(b)) => ValueType::Bool(a != b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a == b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a >= b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a >= b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a >= b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a <= b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a <= b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a <= b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a > b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a > b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a > b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a < b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a < b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a < b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [expr] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match &expr.val {
                ValueType::Bool(val) => ValueType::Bool(!*val),
                _ => return Err(format!("Invalid value: \"{}\"", expr.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(*a || *b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(*a && *b),
                _ => return Err(format!("Invalid values: \"{}\" and \"{}\"", left.val, right.val)),
            })
        },
    ),
];


pub const NATIVE_FUNCS: [(&str, NativeFunction); 1] = [
    (
        "print",
        |args| {
            println!("{}", args.iter().map(|a| { format!("{}", a) }).collect::<Vec<_>>().join(" "));
            Ok(ValueType::Unit)
        }
    ),
];

pub fn get_builtins() -> HashMap<String, ValueType> {
    let ops = NATIVE_OPERATORS.map(|(name, _, f)| (name.to_string(), ValueType::Function(f)))
;
    let fns = NATIVE_FUNCS.map(|(name, f)| (name.to_string(), ValueType::Function(f))).to_vec();
    let mut builtins = ops.to_vec();
    builtins.extend(fns);
    builtins.into_iter().collect::<HashMap<_, _>>()
}



