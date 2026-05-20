//! Rust ↔ ff interop.
//!
//! Bridges Rust and ff in both directions: lift Rust values and functions into
//! a ff [`Env`] so scripts can call them, and pull ff results back into Rust
//! types. All conversion goes through `serde` — anything with a
//! [`Serialize`] / [`DeserializeOwned`] impl works without writing glue.
//!
//! # The two halves
//!
//! **Values** — [`to_value`] and [`from_value`] convert any
//! [`Serialize`] ↔ [`Value`] via `serde_json::Value` as the intermediate
//! representation. [`define_value`] is the same as `to_value` followed by
//! binding the result into an env under a name.
//!
//! **Functions** — [`register`] takes a plain Rust closure
//! (`Fn(A1, ...) -> R`, with `Ai: DeserializeOwned` and `R: Serialize`) and
//! installs it as a curried [`Value::Native`]. [`native_fn`] is the same
//! without the binding step — useful when you're assembling a module object
//! (a [`Value::Object`] of names → natives) rather than populating the global
//! env.
//!
//! For natives that need to inspect raw [`Value`]s (e.g. to build a lazy
//! stream or accept polymorphic arguments), reach for the [`native!`](crate::native) macro
//! instead — it skips serde and hands you the [execution environment](Env) and args (a slice of [`Value`]s) directly.
//!
//! # Conventions
//!
//! - **Atoms** — strings whose first character is `:` round-trip as
//!   [`Value::Atom`]. `to_value(&":ok")` produces `:ok`, not `":ok"`.
//! - **Tagged results** — fallible natives return [`FfResult<T>`], which
//!   serializes as `[:ok, t]` / `[:error, msg]` so ff code can pattern-match
//!   on the shape. Any `Result<T, E: Display>` converts in with `.into()`.
//! - **Currying** — `register(env, "add", |a, b| a + b)` lets scripts write
//!   either `add(1, 2)` or `add(1)(2)`. The parser/Call dispatch handles
//!   partial application; you don't need to think about it.
//!
//! # End-to-end example
//!
//! ```
//! use ff::interop::{FfResult, define_value, register};
//! use ff::interpreter::{Scope, eval_program};
//! use ff::parser::parse;
//! use ff::prelude;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct User { name: String, age: u32 }
//!
//! let env = Scope::new();
//! prelude::install(&env);
//!
//! // Push a Rust value into the script's env.
//! define_value(&env, "me", &User { name: "Ada".into(), age: 36 }).unwrap();
//!
//! // Expose a Rust function — args and return value (de)serialize via serde.
//! register(&env, "greet", |u: User| format!("Hello, {}!", u.name));
//!
//! // Fallible natives use FfResult so ff sees a tagged pair.
//! register(&env, "checked_div", |a: i64, b: i64| -> FfResult<i64> {
//!     if b == 0 { FfResult::Err("divide by zero".into()) } else { FfResult::Ok(a / b) }
//! });
//!
//! let prog = parse("greet(me)").unwrap();
//! assert_eq!(eval_program(&prog, &env).unwrap().to_string(), "\"Hello, Ada!\"");
//!
//! let prog = parse("checked_div(10, 0)").unwrap();
//! assert_eq!(
//!     eval_program(&prog, &env).unwrap().to_string(),
//!     "[:error, \"divide by zero\"]"
//! );
//! ```
//!
//! See [`prelude::fs`](crate::prelude::fs) for a worked example that mixes
//! [`native_fn`] (for the simple cases) with [`native!`](crate::native) (for `file.lines`,
//! which returns a lazy stream).

use std::sync::Arc;

use im::vector;
use rug::{Integer, Rational};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde::ser::{SerializeSeq, Serializer};
use serde_json::{Number as JsonNumber, Value as Json};

use crate::interpreter::{Env, NativeFn, RuntimeError, RuntimeResult, Value, define};

/// ff's tagged result convention: `[:ok, t]` / `[:error, msg]`. Serializes
/// directly to that shape, so natives can return `FfResult<T>` (or just
/// `.into()` a `Result<T, E>`) and stay idiomatic Rust.
///
/// # Example
///
/// ```
/// use ff::interop::{FfResult, register};
/// use ff::interpreter::{Scope, eval_program};
/// use ff::parser::parse;
/// use ff::prelude;
///
/// let env = Scope::new();
/// prelude::install(&env);
/// register(&env, "checked_div", |a: i64, b: i64| -> FfResult<i64> {
///     if b == 0 {
///         Err::<i64, _>("divide by zero").into()
///     } else {
///         FfResult::Ok(a / b)
///     }
/// });
///
/// let prog = parse("checked_div(10, 2)").unwrap();
/// assert_eq!(eval_program(&prog, &env).unwrap().to_string(), "[:ok, 5]");
///
/// let prog = parse("checked_div(10, 0)").unwrap();
/// assert_eq!(
///     eval_program(&prog, &env).unwrap().to_string(),
///     "[:error, \"divide by zero\"]"
/// );
/// ```
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

impl From<FfResult<Value>> for Value {
    fn from(value: FfResult<Value>) -> Self {
        match value {
            FfResult::Ok(val) => Value::List(vector![Value::Atom(":ok".into()), val]),
            FfResult::Err(e) => {
                Value::List(vector![Value::Atom(":err".into()), Value::String(e.into())])
            }
        }
    }
}

/// Build a `Value::Native` with the given name, arity, and body. The body is
/// `Fn(&Env, &[Value]) -> RuntimeResult<Value>`; arity-checking and partial
/// application are handled by the interpreter's Call dispatch.
///
/// Prefer [`native_fn`] / [`register`] when arguments and return values can be
/// (de)serialized — use this only when the native needs to inspect raw
/// [`Value`]s or build lazy/streaming results.
///
/// # Example
///
/// ```
/// use ff::interpreter::{Scope, Value, eval_program};
/// use ff::parser::parse;
/// use ff::prelude;
/// use ff::native;
///
/// let env = Scope::new();
/// prelude::install(&env);
///
/// let pong = native!("pong", 0, |_env, _args| Ok(Value::String("pong".into())));
/// ff::interpreter::define(&env, "pong", pong);
///
/// let prog = parse("pong()").unwrap();
/// assert_eq!(eval_program(&prog, &env).unwrap().to_string(), "\"pong\"");
/// ```
#[macro_export]
macro_rules! native {
    ($name:expr, $arity:expr, $body:expr) => {
        $crate::interpreter::Value::Native {
            name: $name,
            arity: $arity,
            applied: Vec::new(),
            f: $crate::interpreter::NativeFn(std::sync::Arc::new($body)),
        }
    };
}

/// Convert any [`serde::Serialize`] value into a ff [`Value`].
///
/// Vecs become `Value::List`, maps and structs become `Value::Object`, and
/// strings starting with `:` are interpreted as atoms.
///
/// # Example
///
/// ```
/// use ff::interop::to_value;
///
/// let v = to_value(&vec![1u32, 2, 3]).unwrap();
/// assert_eq!(v.to_string(), "[1, 2, 3]");
/// ```
pub fn to_value<T: Serialize + ?Sized>(t: &T) -> RuntimeResult<Value> {
    let json = serde_json::to_value(t).map_err(|e| RuntimeError::Serialize(e.to_string()))?;
    Ok(json_to_value(json))
}

/// Convert a ff [`Value`] back into any [`serde::de::DeserializeOwned`] type.
/// Inverse of [`to_value`]; lazy values (ranges, cons spines) and functions
/// cannot be deserialized.
///
/// # Example
///
/// ```
/// use ff::interop::{from_value, to_value};
///
/// let v = to_value(&vec![1u32, 2, 3]).unwrap();
/// let back: Vec<u32> = from_value(v).unwrap();
/// assert_eq!(back, vec![1, 2, 3]);
/// ```
pub fn from_value<T: DeserializeOwned>(v: Value) -> RuntimeResult<T> {
    let json = value_to_json(v)?;
    serde_json::from_value(json).map_err(|e| RuntimeError::Deserialize(e.to_string()))
}

/// Bind `name` to a serializable Rust value in `env`. Equivalent to
/// `define(env, name, to_value(&v)?)`.
///
/// # Example
///
/// ```
/// use ff::interop::define_value;
/// use ff::interpreter::{Scope, eval_program};
/// use ff::parser::parse;
/// use ff::prelude;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config { host: String, port: u16 }
///
/// let env = Scope::new();
/// prelude::install(&env);
/// define_value(&env, "config", &Config { host: "localhost".into(), port: 8080 }).unwrap();
///
/// let prog = parse("[config.host, config.port]").unwrap();
/// assert_eq!(eval_program(&prog, &env).unwrap().to_string(), "[\"localhost\", 8080]");
/// ```
pub fn define_value<T: Serialize + ?Sized>(env: &Env, name: &str, v: &T) -> RuntimeResult<()> {
    let val = to_value(v)?;
    define(env, name, val);
    Ok(())
}

/// Build a curried `Value::Native` from a Rust function without binding it.
/// Argument types must be `DeserializeOwned`; the return type must be
/// `Serialize` (or `()`). Use this when assembling module objects; use
/// [`register`] to bind directly into an env.
///
/// # Example
///
/// ```
/// use ff::interop::native_fn;
/// use ff::interpreter::Value;
///
/// let double = native_fn("double", |x: i64| x * 2);
/// match double {
///     Value::Native { name, arity, .. } => {
///         assert_eq!(name, "double");
///         assert_eq!(arity, 1);
///     }
///     _ => panic!("expected a Native"),
/// }
/// ```
pub fn native_fn<F, Args>(name: &'static str, f: F) -> Value
where
    F: IntoNative<Args>,
{
    f.into_native(name)
}

/// Bind `name` to a Rust function in `env`. Argument types must be
/// `DeserializeOwned`; the return type must be `Serialize` (or `()`).
/// The native is curried — `f(a, b)` and `f(a)(b)` both work.
///
/// # Example
///
/// ```
/// use ff::interop::register;
/// use ff::interpreter::{Scope, eval_program};
/// use ff::parser::parse;
/// use ff::prelude;
///
/// let env = Scope::new();
/// prelude::install(&env);
/// register(&env, "add", |a: i64, b: i64| a + b);
///
/// let prog = parse("add(3, 4)").unwrap();
/// assert_eq!(eval_program(&prog, &env).unwrap().to_string(), "7");
///
/// // Curried application works too.
/// let prog = parse("add(3)(4)").unwrap();
/// assert_eq!(eval_program(&prog, &env).unwrap().to_string(), "7");
/// ```
pub fn register<F, Args>(env: &Env, name: &'static str, f: F)
where
    F: IntoNative<Args>,
{
    define(env, name, native_fn(name, f));
}

pub trait IntoNative<Args> {
    fn into_native(self, name: &'static str) -> Value;
}

/// Marker for the [`IntoNative`] impl on [`Value`] — lets a [`native!`]
/// (which already produces a `Value::Native`) flow through [`members!`]
/// alongside serde-style closures. The name supplied by [`native_fn`] /
/// [`members!`] overrides whatever name the `native!` invocation used.
pub struct RawNative;

impl IntoNative<RawNative> for Value {
    fn into_native(self, name: &'static str) -> Value {
        match self {
            Value::Native {
                arity, applied, f, ..
            } => Value::Native {
                name,
                arity,
                applied,
                f,
            },
            other => other,
        }
    }
}

fn json_to_value(j: Json) -> Value {
    match j {
        Json::Null => Value::Unit,
        Json::Bool(b) => Value::Bool(b),
        Json::Number(n) => Value::Number(Arc::new(json_number_to_rational(&n))),
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
        Value::Pid(_) => return Err(RuntimeError::PidDeserialize),
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
            F: Fn($($t),*) -> R + Send + Sync + 'static,
            $($t: DeserializeOwned,)*
            R: Serialize,
        {
            fn into_native(self, name: &'static str) -> Value {
                Value::Native {
                    name,
                    arity: $n,
                    applied: Vec::new(),
                    f: NativeFn(Arc::new(move |_env, _args: &[Value]| {
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
