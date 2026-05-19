//! Rust ↔ ff interop.
//!
//! `to_value` / `from_value` convert any `serde::Serialize` ↔ `Value` via
//! `serde_json::Value` as an intermediate. `register` turns a plain Rust
//! closure (`Fn(A1, ...) -> R`, with `A_i: DeserializeOwned` and
//! `R: Serialize`) into a curried `Value::Native` and binds it in the env.

use std::rc::Rc;

use rug::{Integer, Rational};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde::ser::{SerializeSeq, Serializer};
use serde_json::{Number as JsonNumber, Value as Json};

use crate::interpreter::{Env, NativeFn, RuntimeError, RuntimeResult, Value, define};

/// ff's tagged result convention: `[:ok, t]` / `[:error, msg]`. Serializes
/// directly to that shape, so natives can return `FfResult<T>` (or just
/// `.into()` a `Result<T, E>`) and stay idiomatic Rust.
pub enum FfResult<T> {
    Ok(T),
    Err(String),
}

impl<T: Serialize> Serialize for FfResult<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(2))?;
        match self {
            FfResult::Ok(v) => {
                seq.serialize_element(":ok")?;
                seq.serialize_element(v)?;
            }
            FfResult::Err(msg) => {
                seq.serialize_element(":error")?;
                seq.serialize_element(msg)?;
            }
        }
        seq.end()
    }
}

impl<T, E: std::fmt::Display> From<Result<T, E>> for FfResult<T> {
    fn from(r: Result<T, E>) -> Self {
        match r {
            Ok(v) => FfResult::Ok(v),
            Err(e) => FfResult::Err(e.to_string()),
        }
    }
}

// Build a `Value::Native` with the given name, arity, and body. The body is
// `Fn(&Ctx, &[Value]) -> Result<Value>`; arity-checking and partial application
// are handled by the interpreter's Call dispatch.
#[macro_export]
macro_rules! native {
    ($name:expr, $arity:expr, $body:expr) => {
        $crate::interpreter::Value::Native {
            name: $name,
            arity: $arity,
            applied: Vec::new(),
            f: $crate::interpreter::NativeFn(std::rc::Rc::new($body)),
        }
    };
}

pub fn to_value<T: Serialize + ?Sized>(t: &T) -> RuntimeResult<Value> {
    let json = serde_json::to_value(t).map_err(|e| RuntimeError::Serialize(e.to_string()))?;
    Ok(json_to_value(json))
}

pub fn from_value<T: DeserializeOwned>(v: Value) -> RuntimeResult<T> {
    let json = value_to_json(v)?;
    serde_json::from_value(json).map_err(|e| RuntimeError::Deserialize(e.to_string()))
}

/// Bind `name` to a serializable Rust value in `env`. Equivalent to
/// `define(env, name, to_value(&v)?)`.
pub fn define_value<T: Serialize + ?Sized>(env: &Env, name: &str, v: &T) -> RuntimeResult<()> {
    let val = to_value(v)?;
    define(env, name, val);
    Ok(())
}

/// Build a curried `Value::Native` from a Rust function without binding it.
/// Argument types must be `DeserializeOwned`; the return type must be
/// `Serialize` (or `()`). Use this when assembling module objects; use
/// `register` to bind directly into an env.
pub fn native_fn<F, Args>(name: &'static str, f: F) -> Value
where
    F: IntoNative<Args>,
{
    f.into_native(name)
}

/// Bind `name` to a Rust function in `env`. Argument types must be
/// `DeserializeOwned`; the return type must be `Serialize` (or `()`).
/// The native is curried — `f(a, b)` and `f(a)(b)` both work.
pub fn register<F, Args>(env: &Env, name: &'static str, f: F)
where
    F: IntoNative<Args>,
{
    define(env, name, native_fn(name, f));
}

pub trait IntoNative<Args> {
    fn into_native(self, name: &'static str) -> Value;
}

fn json_to_value(j: Json) -> Value {
    match j {
        Json::Null => Value::Unit,
        Json::Bool(b) => Value::Bool(b),
        Json::Number(n) => Value::Number(Rc::new(json_number_to_rational(&n))),
        Json::String(s) => match s.strip_prefix(':') {
            Some(rest) => Value::Atom(rest.into()),
            None => Value::String(s.into()),
        },
        Json::Array(arr) => Value::List(arr.into_iter().map(json_to_value).collect()),
        Json::Object(object) => Value::Object(
            object
                .into_iter()
                .map(|(k, v)| (Value::Atom(k.into()), json_to_value(v)))
                .collect(),
        ),
    }
}

fn json_number_to_rational(n: &JsonNumber) -> Rational {
    if let Some(i) = n.as_i64() {
        Rational::from(i)
    } else if let Some(u) = n.as_u64() {
        Rational::from(u)
    } else if let Some(f) = n.as_f64() {
        Rational::from_f64(f).unwrap_or_default()
    } else {
        Rational::new()
    }
}

fn value_to_json(v: Value) -> RuntimeResult<Json> {
    Ok(match v {
        Value::Unit => Json::Null,
        Value::Bool(b) => Json::Bool(b),
        Value::Number(rat) => rational_to_json(&rat)?,
        Value::String(s) => Json::String(s.to_string()),
        Value::Atom(name) => Json::String(format!(":{name}")),
        Value::List(xs) => Json::Array(
            xs.into_iter()
                .map(value_to_json)
                .collect::<RuntimeResult<_>>()?,
        ),
        Value::Set(xs) => Json::Array(
            xs.iter()
                .cloned()
                .map(value_to_json)
                .collect::<RuntimeResult<_>>()?,
        ),
        Value::Object(entries) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in entries.iter() {
                let key = match k {
                    Value::String(s) => s.to_string(),
                    Value::Atom(s) => s.to_string(),
                    other => return Err(RuntimeError::NonStringObjectKey(other.to_string())),
                };
                obj.insert(key, value_to_json(v.clone())?);
            }
            Json::Object(obj)
        }
        Value::Range { .. } | Value::Cons { .. } => return Err(RuntimeError::LazyDeserialize),
        Value::Function { .. } | Value::Native { .. } => {
            return Err(RuntimeError::FunctionDeserialize);
        }
    })
}

fn rational_to_json(rat: &Rational) -> RuntimeResult<Json> {
    let denom = rat.denom();
    if denom == &Integer::from(1) {
        let num = rat.numer();
        if let Some(i) = num.to_i64() {
            return Ok(Json::Number(JsonNumber::from(i)));
        }
        if let Some(u) = num.to_u64() {
            return Ok(Json::Number(JsonNumber::from(u)));
        }
    }
    let f = rat.to_f64();
    let n = JsonNumber::from_f64(f)
        .ok_or_else(|| RuntimeError::CannotRepresentAsFloat(rat.to_string()))?;
    Ok(Json::Number(n))
}

macro_rules! impl_into_native {
    ($n:expr; $($i:tt: $t:ident),*) => {
        impl<F, R, $($t,)*> IntoNative<($($t,)*)> for F
        where
            F: Fn($($t),*) -> R + 'static,
            $($t: DeserializeOwned,)*
            R: Serialize,
        {
            fn into_native(self, name: &'static str) -> Value {
                Value::Native {
                    name,
                    arity: $n,
                    applied: Vec::new(),
                    f: NativeFn(Rc::new(move |_env, _args: &[Value]| {
                        let ret = (self)($(
                            from_value(_args[$i].clone())?,
                        )*);
                        to_value(&ret)
                    })),
                }
            }
        }
    };
}

impl_into_native!(0; );
impl_into_native!(1; 0: A0);
impl_into_native!(2; 0: A0, 1: A1);
impl_into_native!(3; 0: A0, 1: A1, 2: A2);
impl_into_native!(4; 0: A0, 1: A1, 2: A2, 3: A3);
impl_into_native!(5; 0: A0, 1: A1, 2: A2, 3: A3, 4: A4);
impl_into_native!(6; 0: A0, 1: A1, 2: A2, 3: A3, 4: A4, 5: A5);
