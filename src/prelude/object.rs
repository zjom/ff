use crate::prelude::RuntimeError;
use crate::prelude::Value;
use crate::{members, native};

members! {
    "Object",
    get => native!(2, move |_env, args| {
        if let Value::Object(obj) = &args[1] {
            match obj.get(&args[0]) {
                Some(value) => Ok(value.clone()),
                None => Ok(Value::Unit),
            }
        } else {
            Err(RuntimeError::UnsupportedOperation(format!(
                "Object.get expected Object, found {}",
                args[1]
            )))
        }
    }),
    put => native!(3, move |_env, args| {
        if let Value::Object(mut obj) = args[2].clone() {
            obj.insert(args[0].clone(), args[1].clone());
            Ok(Value::Object(obj))
        } else {
            Err(RuntimeError::UnsupportedOperation(format!(
                "Object.put expected Object, found {}",
                args[2]
            )))
        }
    }),
    keys => native!(1, move |_env, args| {
        if let Value::Object(obj) = &args[0] {
            Ok(Value::List(obj.keys().cloned().collect()))
        } else {
            Err(RuntimeError::UnsupportedOperation(format!(
                "Object.keys expected Object, found {}",
                args[0]
            )))
        }
    }),
    values => native!(1, move |_env, args| {
        if let Value::Object(obj) = &args[0] {
            Ok(Value::List(obj.values().cloned().collect()))
        } else {
            Err(RuntimeError::UnsupportedOperation(format!(
                "Object.values expected Object, found {}",
                args[0]
            )))
        }
    }),
}
