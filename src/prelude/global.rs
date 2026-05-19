use im::{HashMap, HashSet, Vector};
use rug::Rational;

use crate::interpreter::{RuntimeError, Value, ctx_of, type_name};
use crate::native;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        ("::", cons()),
        ("print", print()),
        ("typeof", type_of()),
        ("println", println()),
        ("panic", panic()),
        ("default", default()),
    ]
}

fn type_of() -> Value {
    native!("typeof", 1, |_env, args| {
        Ok(Value::Atom(type_name(&args[0]).into()))
    })
}

fn panic() -> Value {
    native!("panic", 1, |_env, args| {
        Err(RuntimeError::Panic(args[0].to_string()))
    })
}

fn cons() -> Value {
    native!("::", 2, |_env, args| {
        match (args[0].clone(), args[1].clone()) {
            (v, Value::List(mut xs)) => {
                xs.push_front(v);
                Ok(Value::List(xs))
            }

            (Value::String(left), Value::String(right)) => {
                Ok(Value::String(format!("{}{}", left, right).into()))
            }

            (v, Value::Set(mut xs)) => {
                xs.insert(v);
                Ok(Value::Set(xs))
            }
            (Value::List(kvs), Value::Object(mut xs)) => {
                if kvs.len() == 2 {
                    let (key, value) = (kvs[0].clone(), kvs[1].clone());
                    xs.insert(key, value);
                    Ok(Value::Object(xs))
                } else {
                    Err(RuntimeError::UnsupportedOperation(
                        "can only cons a 2-element list with an object".into(),
                    ))
                }
            }

            (left, right) => Err(RuntimeError::UnsupportedOperation(format!(
                "cannot cons {} with {}",
                left, right
            ))),
        }
    })
}

fn default() -> Value {
    native!("default", 1, |_env, args| {
        Ok(match &args[0] {
            Value::Unit => Value::Unit,
            Value::Number(_) => Value::Number(Rational::new().into()),
            Value::String(_) => Value::String("".into()),
            Value::Bool(_) => Value::Bool(false),
            Value::List(_) => Value::List(Vector::new()),
            Value::Object(_) => Value::Object(HashMap::new()),
            Value::Set(_) => Value::Set(HashSet::new()),
            Value::Range { .. } => Value::Range {
                start: Rational::new().into(),
                end: Some(Rational::new().into()),
                inclusive: false,
            },
            // A lazy cons cell behaves like a list view; the natural empty is
            // just the empty list.
            Value::Cons { .. } => Value::List(Vector::new()),
            v @ Value::Atom(_)
            | v @ Value::Native { .. }
            | v @ Value::Function { .. }
            | v @ Value::Pid(_) => {
                let t = type_name(v);
                return Err(RuntimeError::UnsupportedOperation(format!(
                    "default is not supported for {},{},{}",
                    t, t, t
                )));
            }
        })
    })
}

fn print() -> Value {
    native!("io.print", 1, |env, args| {
        let s = &args[0].to_string();
        let ctx = ctx_of(env);
        if ctx.is_interactive {
            writeln!(ctx_of(env).out.lock().unwrap(), "{}", s)?;
        } else {
            write!(ctx_of(env).out.lock().unwrap(), "{}", s)?;
        }
        Ok(Value::Unit)
    })
}
fn println() -> Value {
    native!("io.println", 1, |env, args| {
        let s = &args[0].to_string();
        writeln!(ctx_of(env).out.lock().unwrap(), "{}", s)?;
        Ok(Value::Unit)
    })
}
