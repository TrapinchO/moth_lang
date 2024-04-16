use std::collections::HashMap;
use std::fmt::Display;
use std::time::SystemTime;

use crate::exprstmt::Stmt;
// TODO: kinda circular import, but it should be fine
use crate::located::Located;
use crate::reassoc::{Associativity, Precedence};

#[derive(Debug, PartialEq, Clone)]
pub enum ValueType {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    List(Vec<Value>),
    NativeFunction(fn(Vec<Value>) -> Result<ValueType, String>),
    Function(Vec<String>, Vec<Stmt>),
    Unit,
}
impl ValueType {
    fn format(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::Float(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::String(s) => format!("\"{}\"", s),
            Self::List(ls) => format!("[{}]", ls.iter().map(|e| { e.val.format() }).collect::<Vec<_>>().join(", ")),
            Self::NativeFunction(_) => "<function>".to_string(), // TODO: improve
            Self::Function(..) => "<function>".to_string(),
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
                (ValueType::List(a), ValueType::List(b)) => {
                    let mut res = a.clone();
                    res.append(&mut b.clone());
                    ValueType::List(res)
                }
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
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a != b),
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

pub const NATIVE_FUNCS: [(&str, NativeFunction); 3] = [
    ("print", |args| {
        println!(
            "{}",
            args.iter().map(|a| { a.to_string() }).collect::<Vec<_>>().join(" ")
        );
        Ok(ValueType::Unit)
    }),
    ("time", |args| {
        if !args.is_empty() {
            return Err(format!("\"times\" function takes no arguments, got: {}", args.len()));
        }
        Ok(ValueType::Int(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .try_into()
                .unwrap(),
        ))
    }),
    ("len", |args| {
        if args.len() != 1 {
            return Err(format!("Function takes exactly 1 argument, got: {}", args.len()))
        }
        let val = &args.get(0).unwrap().val;
        Ok(ValueType::Int(match val {
            ValueType::String(s) => s.len() as i32,
            ValueType::List(ls) => ls.len() as i32,
            _ => return Err(format!("Invalid value: {}", val))
        }))
    })
];

pub fn get_builtins() -> HashMap<String, ValueType> {
    let ops = NATIVE_OPERATORS.map(|(name, _, f)| (name.to_string(), ValueType::NativeFunction(f)));
    let fns = NATIVE_FUNCS
        .map(|(name, f)| (name.to_string(), ValueType::NativeFunction(f)))
        .to_vec();
    let mut builtins = ops.to_vec();
    builtins.extend(fns);
    builtins.into_iter().collect::<HashMap<_, _>>()
}
