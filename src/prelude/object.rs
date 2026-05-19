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
            let value = obj.iter().find(|(k, _v)| k == &args[1]).map(|(_k, v)| v);
            match value {
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
            obj.retain(|(k, _)| k != &args[1]);
            obj.push_back((args[1].clone(), args[2].clone()));
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
        if let Value::Object(obj) = args[0].clone() {
            Ok(Value::List(obj.into_iter().map(|(k, _v)| k).collect()))
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
        if let Value::Object(obj) = args[0].clone() {
            Ok(Value::List(obj.into_iter().map(|(_k, v)| v).collect()))
        } else {
            Err(RuntimeError::UnsupportedOperation(format!(
                "object.values expected Object, found {}",
                args[0]
            )))
        }
    })
}
