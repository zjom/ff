use im::{HashMap, Vector};

use crate::{
    interpreter::{RuntimeError, Value, type_name},
    native,
    prelude::{err, ok},
};

pub fn members() -> Vec<(&'static str, Value)> {
    vec![("args", args()), ("vars", vars()), ("var", var())]
}

fn args() -> Value {
    native!("Env.args", 0, |_, _| {
        let args: Vector<Value> = std::env::args().map(|a| Value::String(a.into())).collect();
        Ok(Value::List(args))
    })
}

fn vars() -> Value {
    native!("Env.vars", 0, |_, _| {
        let mut vars = HashMap::new();
        std::env::vars().for_each(|(key, value)| {
            vars.insert(Value::String(key.into()), Value::String(value.into()));
        });

        Ok(Value::Object(vars))
    })
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
