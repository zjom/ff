//! Rust ↔ ff interop.
//!
//! `to_value` / `from_value` convert any `serde::Serialize` ↔ `Value` via
//! `serde_json::Value` as an intermediate. `register` turns a plain Rust
//! closure (`Fn(A1, ...) -> R`, with `A_i: DeserializeOwned` and
//! `R: Serialize`) into a curried `Value::Native` and binds it in the env.

use std::rc::Rc;

use anyhow::{Result, anyhow, bail};
use rug::{Integer, Rational};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Number as JsonNumber, Value as Json};

use crate::interpreter::{Env, NativeFn, Value, define};

pub fn to_value<T: Serialize + ?Sized>(t: &T) -> Result<Value> {
    let json = serde_json::to_value(t).map_err(|e| anyhow!("serialize: {}", e))?;
    Ok(json_to_value(json))
}

pub fn from_value<T: DeserializeOwned>(v: Value) -> Result<T> {
    let json = value_to_json(v)?;
    serde_json::from_value(json).map_err(|e| anyhow!("deserialize: {}", e))
}

/// Bind `name` to a serializable Rust value in `env`. Equivalent to
/// `define(env, name, to_value(&v)?)`.
pub fn define_value<T: Serialize + ?Sized>(env: &Env, name: &str, v: &T) -> Result<()> {
    let val = to_value(v)?;
    define(env, name, val);
    Ok(())
}

/// Bind `name` to a Rust function in `env`. Argument types must be
/// `DeserializeOwned`; the return type must be `Serialize` (or `()`).
/// The native is curried — `f(a, b)` and `f(a)(b)` both work.
pub fn register<F, Args>(env: &Env, name: &'static str, f: F)
where
    F: IntoNative<Args>,
{
    define(env, name, f.into_native(name));
}

pub trait IntoNative<Args> {
    fn into_native(self, name: &'static str) -> Value;
}

fn json_to_value(j: Json) -> Value {
    match j {
        Json::Null => Value::Unit,
        Json::Bool(b) => Value::Bool(b),
        Json::Number(n) => Value::Number(Rc::new(json_number_to_rational(&n))),
        Json::String(s) => Value::String(s.into()),
        Json::Array(arr) => Value::List(arr.into_iter().map(json_to_value).collect()),
        Json::Object(map) => Value::Dict(
            map.into_iter()
                .map(|(k, v)| (Value::String(k.into()), json_to_value(v)))
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

fn value_to_json(v: Value) -> Result<Json> {
    Ok(match v {
        Value::Unit => Json::Null,
        Value::Bool(b) => Json::Bool(b),
        Value::Number(rat) => rational_to_json(&rat)?,
        Value::String(s) => Json::String(s.to_string()),
        // Atoms degrade to plain strings on the way out — that's the most
        // useful default for serde-tagged enums (`:Ok` ↔ `"Ok"`).
        Value::Atom(name) => Json::String(name.to_string()),
        Value::List(xs) => Json::Array(
            xs.into_iter()
                .map(value_to_json)
                .collect::<Result<_>>()?,
        ),
        Value::Set(xs) => Json::Array(
            xs.into_iter()
                .map(value_to_json)
                .collect::<Result<_>>()?,
        ),
        Value::Dict(entries) => {
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                let key = match k {
                    Value::String(s) => s.to_string(),
                    Value::Atom(s) => s.to_string(),
                    other => bail!("cannot deserialize dict with non-string key: {}", other),
                };
                map.insert(key, value_to_json(v)?);
            }
            Json::Object(map)
        }
        Value::Range { .. } | Value::Cons { .. } => {
            bail!("cannot deserialize lazy values; collect into a list first")
        }
        Value::Function { .. } | Value::Native { .. } | Value::Module { .. } => {
            bail!("cannot deserialize a function or module")
        }
    })
}

fn rational_to_json(rat: &Rational) -> Result<Json> {
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
    let n =
        JsonNumber::from_f64(f).ok_or_else(|| anyhow!("cannot represent {} as a float", rat))?;
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
