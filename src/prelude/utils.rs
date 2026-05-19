use crate::interpreter::Value;
use im::{HashMap, vector};
use std::rc::Rc;
pub(super) fn ok(v: Value) -> Value {
    Value::List(vector![Value::Atom("ok".into()), v])
}

pub(super) fn err(msg: impl Into<Rc<str>>) -> Value {
    Value::List(vector![
        Value::Atom("error".into()),
        Value::String(msg.into())
    ])
}

pub(super) fn object(entries: impl IntoIterator<Item = (impl Into<Rc<str>>, Value)>) -> Value {
    let mut out: HashMap<Value, Value> = HashMap::new();
    for (k, v) in entries {
        out.insert(Value::Atom(k.into()), v);
    }
    Value::Object(out)
}
