use crate::interpreter::Value;
use im::{HashMap, vector};
pub(super) fn ok(v: Value) -> Value {
    Value::List(vector![Value::Atom("ok".into()), v])
}

pub(super) fn err(msg: impl Into<String>) -> Value {
    Value::List(vector![
        Value::Atom("error".into()),
        Value::String(msg.into().into())
    ])
}

pub(super) fn object(entries: Vec<(&str, Value)>) -> Value {
    let entries = entries
        .into_iter()
        .map(|(name, value)| (Value::String(name.into()), value))
        .collect();
    Value::Object(entries)
}

pub(super) fn object_with_atom_keys(entries: impl IntoIterator<Item = (String, Value)>) -> Value {
    let mut out: HashMap<Value, Value> = HashMap::new();
    for (k, v) in entries {
        out.insert(Value::Atom(k.into()), v);
    }
    Value::Object(out)
}
