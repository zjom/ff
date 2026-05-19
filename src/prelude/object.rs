use crate::native;
use crate::prelude::RuntimeError;
use crate::prelude::Value;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        ("get", get()),
        ("put", put()),
        ("keys", keys()),
        ("values", values()),
    ]
}
fn get() -> Value {
    native!("object.get", 2, move |_env, args| {
        if let Value::Object(obj) = &args[0] {
            match obj.get(&args[1]) {
                Some(value) => Ok(value.clone()),
                None => Err(RuntimeError::ObjectMissingKey(format!("{}", args[0]))),
            }
        } else {
            Err(RuntimeError::UnsupportedOperation(format!(
                "object.get expected Object, found {}",
                args[0]
            )))
        }
    })
}
fn put() -> Value {
    native!("object.put", 3, move |_env, args| {
        if let Value::Object(mut obj) = args[0].clone() {
            obj.insert(args[1].clone(), args[2].clone());
            Ok(Value::Object(obj))
        } else {
            Err(RuntimeError::UnsupportedOperation(format!(
                "object.put expected Object, found {}",
                args[0]
            )))
        }
    })
}

fn keys() -> Value {
    native!("object.keys", 1, move |_env, args| {
        if let Value::Object(obj) = &args[0] {
            Ok(Value::List(obj.keys().cloned().collect()))
        } else {
            Err(RuntimeError::UnsupportedOperation(format!(
                "object.keys expected Object, found {}",
                args[0]
            )))
        }
    })
}

fn values() -> Value {
    native!("object.values", 1, move |_env, args| {
        if let Value::Object(obj) = &args[0] {
            Ok(Value::List(obj.values().cloned().collect()))
        } else {
            Err(RuntimeError::UnsupportedOperation(format!(
                "object.values expected Object, found {}",
                args[0]
            )))
        }
    })
}
