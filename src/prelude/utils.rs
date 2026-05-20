use crate::interpreter::Value;
use im::{HashMap, vector};
use std::sync::Arc;
pub(super) fn ok_tuple(v: Value) -> Value {
    Value::List(vector![Value::Atom("ok".into()), v])
}

/// constructs a len 2 list: `[:error, msg]` where msg is String
pub(super) fn err_str_tuple(msg: impl Into<Arc<str>>) -> Value {
    Value::List(vector![
        Value::Atom("error".into()),
        Value::String(msg.into())
    ])
}

// `[:error, :tag]` — distinct from the string-bearing `err` helper so
// reasons like `:no_proc` stay matchable as atoms.
pub(super) fn err_atom_tuple(tag: &str) -> Value {
    let trimmed = tag.strip_prefix(':').unwrap_or(tag);
    Value::List(vector![
        Value::Atom("error".into()),
        Value::Atom(trimmed.into()),
    ])
}

pub(super) fn object(entries: impl IntoIterator<Item = (impl Into<Arc<str>>, Value)>) -> Value {
    let mut out: HashMap<Value, Value> = HashMap::new();
    for (k, v) in entries {
        out.insert(Value::Atom(k.into()), v);
    }
    Value::Object(out)
}

/// Build a `pub fn members() -> Vec<(&'static str, Value)>` for a prelude
/// module. Each entry is `name => body` where `body` is anything that
/// implements [`IntoNative`](crate::interop::IntoNative) — either a closure
/// (serde-style) or a `native!` expression.
///
/// The name may be a Rust identifier (`len`) or a string literal (`"::"`,
/// `"typeof"`) — the literal form lets you expose names that aren't valid
/// idents or that collide with Rust keywords. The native's display name is
/// `concat!($module, ".", $name)`; an empty `$module` produces just the bare
/// name.
#[macro_export]
macro_rules! members {
    ($module:literal, $($name:tt => $body:expr),* $(,)?) => {
        pub fn members() -> Vec<(&'static str, $crate::interpreter::Value)> {
            vec![$(
                $crate::__member_entry!($module, $name, $body),
            )*]
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __member_entry {
    ("", $name:literal, $body:expr) => {
        ($name, $crate::interop::native_fn($name, $body))
    };
    ("", $name:ident, $body:expr) => {
        (
            stringify!($name),
            $crate::interop::native_fn(stringify!($name), $body),
        )
    };
    ($module:literal, $name:literal, $body:expr) => {
        (
            $name,
            $crate::interop::native_fn(concat!($module, ".", $name), $body),
        )
    };
    ($module:literal, $name:ident, $body:expr) => {
        (
            stringify!($name),
            $crate::interop::native_fn(
                concat!($module, ".", stringify!($name)),
                $body,
            ),
        )
    };
}
