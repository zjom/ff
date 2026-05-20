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
