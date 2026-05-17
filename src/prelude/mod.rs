use crate::interpreter::{Env, Value, define};
use std::collections::HashMap;

mod global;
mod io;

pub fn install(env: &Env) {
    for (name, f) in global::members() {
        define(env, name, f);
    }
}

pub(crate) fn native_module(name: &str) -> Option<Value> {
    let members = match name {
        "io" => io::members(),
        _ => return None,
    };
    Some(Value::Module {
        name: name.to_string(),
        members: members
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<HashMap<_, _>>(),
    })
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
