use std::collections::HashMap;

use crate::{
    interop::native_fn,
    interpreter::{RuntimeError, Value, type_name},
    native,
    prelude::{err, ok},
};

pub fn members() -> Vec<(&'static str, Value)> {
    vec![("args", args()), ("vars", vars()), ("var", var())]
}

fn args() -> Value {
    native_fn("Env.args", || std::env::args().collect::<Vec<String>>())
}

fn vars() -> Value {
    native_fn("Env.vars", || std::env::vars().collect::<HashMap<String, String>>())
}

fn var() -> Value {
    native!("Env.var", 1, |_, args| {
        match &args[0] {
            Value::String(key) | Value::Atom(key) => match std::env::var(key.to_string()) {
                Ok(value) => Ok(ok(Value::String(value.into()))),
                Err(e) => Ok(err(e.to_string())),
            },
            v => Err(RuntimeError::UnsupportedOperation(format!(
                "Env.var expected arg of type String, got: {}",
                type_name(v),
            ))),
        }
    })
}
