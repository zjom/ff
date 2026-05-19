use std::collections::HashMap;

use crate::{
    interop::native_fn,
    interpreter::Value,
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
    // Accept either a string key (`"PATH"`) or an atom (`:PATH`, which arrives
    // as the serialized form `":PATH"`); strip the leading `:` to recover the
    // env-var name in both cases.
    native_fn("Env.var", |key: String| {
        let name = key.strip_prefix(':').unwrap_or(&key);
        match std::env::var(name) {
            Ok(value) => (":ok", value),
            Err(e) => (":error", e.to_string()),
        }
    })
}
