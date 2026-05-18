use im::Vector;
use rug::Rational;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Expr;

use super::number::format_rational;
use super::scope::Env;

#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Number(Rc<Rational>),
    String(Rc<str>),
    Bool(bool),
    List(Vector<Value>),
    Tuple(Vector<Value>),
    Dict(Vector<(Value, Value)>),
    Set(Vector<Value>),
    // Lazy integer-step range. `end == None` is infinite (`[start..]`);
    // `inclusive` distinguishes `[a..b]` from `[a..=b]`. Step is always +1.
    Range {
        start: Rc<Rational>,
        end: Option<Rc<Rational>>,
        inclusive: bool,
    },
    Function {
        params: Vec<String>,
        body: Expr,
        env: Env,
    },
    Native {
        name: &'static str,
        arity: usize,
        applied: Vec<Value>,
        f: NativeFn,
    },
    Module {
        name: String,
        members: HashMap<String, Value>,
    },
}

pub type NativeFunction = Rc<dyn Fn(&Env, &[Value]) -> anyhow::Result<Value>>;

#[derive(Clone)]
pub struct NativeFn(pub NativeFunction);

impl std::fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<native fn>")
    }
}

pub fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Unit => "unit",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Bool(_) => "bool",
        Value::List(_) => "list",
        Value::Tuple(_) => "tuple",
        Value::Dict(_) => "dict",
        Value::Set(_) => "set",
        Value::Range { .. } => "range",
        Value::Function { .. } => "function",
        Value::Native { .. } => "native",
        Value::Module { .. } => "module",
    }
}

fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Unit, Value::Unit) => true,
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::List(x), Value::List(y)) | (Value::Tuple(x), Value::Tuple(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| value_eq(a, b))
        }
        (Value::Dict(x), Value::Dict(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.iter().any(|(k2, v2)| value_eq(k, k2) && value_eq(v, v2)))
        }
        (Value::Set(x), Value::Set(y)) => {
            x.len() == y.len() && x.iter().all(|a| y.iter().any(|b| value_eq(a, b)))
        }
        (
            Value::Range {
                start: s1,
                end: e1,
                inclusive: i1,
            },
            Value::Range {
                start: s2,
                end: e2,
                inclusive: i2,
            },
        ) => {
            s1 == s2
                && i1 == i2
                && match (e1, e2) {
                    (None, None) => true,
                    (Some(a), Some(b)) => a == b,
                    _ => false,
                }
        }
        _ => false,
    }
}

impl core::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        value_eq(self, other)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => Ok(()),
            Value::Number(n) => write!(f, "{}", format_rational(n)),
            Value::String(s) => write!(f, "{:?}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::List(xs) => {
                write!(f, "[")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", x)?;
                }
                write!(f, "]")
            }
            Value::Tuple(xs) => {
                write!(f, "(")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", x)?;
                }
                if xs.len() == 1 {
                    write!(f, ",")?;
                }
                write!(f, ")")
            }
            Value::Dict(es) => {
                write!(f, "{{")?;
                for (i, (k, v)) in es.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Set(xs) => {
                write!(f, "{{")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", x)?;
                }
                write!(f, "}}")
            }
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                write!(f, "[{}..", format_rational(start))?;
                if *inclusive {
                    write!(f, "=")?;
                }
                if let Some(e) = end {
                    write!(f, "{}", format_rational(e))?;
                }
                write!(f, "]")
            }
            Value::Function { params, .. } => write!(f, "<fn ({})>", params.join(", ")),
            Value::Native { name, .. } => write!(f, "<native {}>", name),
            Value::Module { name, .. } => write!(f, "<module {}>", name),
        }
    }
}
