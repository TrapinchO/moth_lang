use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;
use std::time::SystemTime;

use crate::exprstmt::Stmt;
use crate::located::Located;
use crate::reassoc::{Associativity, Precedence};

// stands for MothReference
#[derive(Debug, Clone)]
pub struct MRef<T>(Rc<UnsafeCell<T>>);
impl<T> MRef<T> {
    pub fn new(val: T) -> Self {
        MRef(Rc::new(UnsafeCell::new(val)))
    }

    pub fn read<V: 'static>(&self, f: impl FnOnce(&T) -> V) -> V {
        unsafe {
            f(&*self.0.get())
        }
    }

    pub fn write(&mut self, val: T) {
        unsafe {
            *(self.0.get()) = val;
        }
    }
}

impl<T> From<T> for MRef<T> {
    fn from(value: T) -> Self {
        MRef::new(value)
    }
}

impl<T: PartialEq> PartialEq for MRef<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            *self.0.get() == *other.0.get()
        }
    }
}


pub type MList = MRef<Vec<Value>>;
impl MList {
    pub fn modify(&mut self, idx: usize, val: Value) {
        unsafe {
            let ls = &mut *self.0.get();
            ls[idx] = val;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=Value> {
        MListIter::new(self.clone())
    }

    // checks whether it is in the possible range (even if negative)
    // and returns it as a positive index
    pub fn check_index(idx: i32, length: usize) -> Option<usize> {
        if length as i32 <= idx || idx < -(length as i32) {
            return None;
        }
        Some(if idx < 0 { length as i32 + idx } else { idx } as usize)
    }
}
struct MListIter {
    idx: usize,
    len: usize,
    ls: MList,
}
impl MListIter {
    pub fn new(ls: MList) -> Self {
        let len = ls.read(|l| l.len());
        MListIter {
            ls, len, idx: 0,
        }
    }
}
impl Iterator for MListIter {
    type Item = Value;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }
        let item = self.ls.read(|l| l[self.idx].clone());
        self.idx += 1;
        Some(item)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    String(String),
    Bool(bool),
    Int(i32),
    Float(f32),
    List(MList),
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
            Self::List(ls) => format!("[{}]",
                                      ls.read(|l| l.clone()).iter()
                                      .map(|e| { e.val.format() })
                                      .collect::<Vec<_>>().join(", ")),
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
            let [left, right] = &*args else {
                return Err(format!("Wrong number of arguments: {}", args.len()));
            };
            Ok(match (&left.val, &right.val) {
                (ValueType::Int(a), ValueType::Int(b)) => ValueType::Int(a + b),
                (ValueType::Float(a), ValueType::Float(b)) => ValueType::Float(a + b),
                (ValueType::String(a), ValueType::String(b)) => ValueType::String(a.clone() + b),
                (ValueType::List(a), ValueType::List(b)) => {
                    let mut res = vec![];
                    for i in a.iter() { res.push(i); }
                    for i in b.iter() { res.push(i); }
                    ValueType::List(res.into())
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
        let val = &args.first().unwrap().val;
        Ok(ValueType::Int(match val {
            ValueType::String(s) => s.len() as i32,
            ValueType::List(ls) => ls.read(|l| l.len()) as i32,
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
