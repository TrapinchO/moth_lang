use std::collections::HashMap;
use std::fmt::Display;
use std::time::SystemTime;

use crate::associativity::{Associativity, Precedence};
use super::lowexprstmt::{Identifier, Stmt};
use crate::located::Located;
use crate::mref::{MList, MMap};

pub type NativeFunction = fn(Vec<ValueType>) -> Result<ValueType, String>;
pub type Closure = Vec<MMap<ValueType>>;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    List(MList),
    NativeFunction(NativeFunction),
    Function(Vec<String>, Vec<Stmt>, Closure), // fn(params) { block }, closure
    Struct(Identifier, Vec<Identifier>, MMap<ValueType>), // name, fields, methods
    Instance(String, MMap<ValueType>),
    Unit,
}
impl ValueType {
    fn format(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::Float(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::String(s) => format!("\"{s}\""),
            Self::List(ls) => format!(
                "[{}]",
                ls.read(|l| l.clone())
                    .iter()
                    .map(|e| { e.val.format() })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::NativeFunction(_) => "<native function>".to_string(), // TODO: improve
            Self::Function(params, body, _) => format!(
                "fun({}) {{ {} }}",
                params.join(", "),
                body.iter().map(|s| format!("{s}")).collect::<Vec<_>>().join(", ")
            ),
            Self::Unit => "()".to_string(),
            Self::Struct(name, fields, _) => format!("struct {name} {{ {} }}", fields.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ")),
            Self::Instance(name, map) => format!(
                "{}({})",
                name,
                map.read(|m| format!("{{ {} }}", m.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(", ")))
            ),
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
// TODO: fix the precedence mess
// and do this by making a "prelude" module which defines all the functions and whatnot,
// then make them call native functions with "mothNative" or something
// see https://github.com/ElaraLang/elara/blob/master/prim.elr
// PIE anyone?
// 
// TODO: also move the vars in error messages into the string some time
//
// NOTE: btw clippy complains about usize being cast to i32, just so you know
pub const NATIVE_OPERATORS: [(&str, Precedence, NativeFunction); 13] = [
    (
        "+",
        Precedence {
            prec: 5,
            assoc: Associativity::Left,
        },
        |args| {
            let [left, right] = &*args else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a + b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a + b),
                (ValueType::String(a), ValueType::String(b)) => ValueType::String(a.clone() + b),
                (ValueType::List(a), ValueType::List(b)) => {
                    let mut res = vec![];
                    for i in a.iter() {
                        res.push(i);
                    }
                    for i in b.iter() {
                        res.push(i);
                    }
                    ValueType::List(res.into())
                }
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            let [left, right] = &args[..] else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a - b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a - b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a * b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a * b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => {
                    if *b == 0 {
                        return Err("Attempted division by zero".to_string());
                    }
                    ValueType::Int(a / b)
                }
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a / b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a % b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a % b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a == b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a == b),
                (ValueType::String(a), ValueType::String(b)) => ValueType::Bool(a == b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a == b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a != b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a != b),
                (ValueType::String(a), ValueType::String(b)) => ValueType::Bool(a != b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a != b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a >= b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a >= b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a >= b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a <= b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a <= b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a <= b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a > b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a > b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a > b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Bool(a < b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Bool(a < b),
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(a < b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(*a || *b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
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
            Ok(match (left, right) {
                (ValueType::Bool(a), ValueType::Bool(b)) => ValueType::Bool(*a && *b),
                _ => return Err(format!("Invalid values: \"{left}\" and \"{right}\"")),
            })
        },
    ),
];

pub const NATIVE_FUNCS: [(&str, NativeFunction); 5] = [
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
            return Err(format!("Function takes exactly 1 argument, got: {}", args.len()));
        }
        let val = &args.first().unwrap();
        Ok(ValueType::Int(match val {
            ValueType::String(s) => s.len() as i32,
            ValueType::List(ls) => ls.read(|l| l.len()) as i32,
            _ => return Err(format!("Invalid value: {val}")),
        }))
    }),
    ("$$not", |args| {
        if args.len() != 1 {
            return Err(format!("Function takes exactly 1 argument, got: {}", args.len()));
        }
        let val = &args.first().unwrap();
        Ok(ValueType::Bool(match val {
            ValueType::Bool(b) => !b,
            _ => return Err("Expected a bool".to_string()),
        }))
    }),
    ("$$neg", |args| {
        if args.len() != 1 {
            return Err(format!("Function takes exactly 1 argument, got: {}", args.len()));
        }
        let val = &args.first().unwrap();
        Ok(match val {
            ValueType::Int(n) => ValueType::Int(-n),
            ValueType::Float(n) => ValueType::Float(-n),
            _ => return Err("Expected a number".to_string()),
        })
    }),
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
