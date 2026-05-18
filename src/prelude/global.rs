use anyhow::bail;
use im::Vector;
use rug::Rational;

use crate::interpreter::{Value, ctx_of, type_name};
use crate::native;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        ("::", cons()),
        ("print", print()),
        ("println", println()),
        ("panic", panic()),
        ("default", default()),
    ]
}

pub fn panic() -> Value {
    native!("panic", 1, |_env, args| { bail!("panic: {}", args[0]) })
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

            (v, Value::Tuple(mut xs)) => {
                xs.push_front(v);
                Ok(Value::Tuple(xs))
            }

            (v, Value::Set(mut xs)) => {
                if xs.iter().any(|x| &v == x) {
                    Ok(Value::Set(xs))
                } else {
                    xs.push_back(v);
                    Ok(Value::Set(xs))
                }
            }
            (Value::Tuple(kvs) | Value::List(kvs), Value::Dict(mut xs)) => {
                if kvs.len() == 2 {
                    let (key, value) = (kvs[0].clone(), kvs[1].clone());
                    xs.retain(|(k, _)| k != &key);
                    xs.push_back((key, value));
                    Ok(Value::Dict(xs))
                } else {
                    bail!(
                        "unsupported operation: can only cons a list or tuple of length 2 with a dict"
                    )
                }
            }

            (left, right) => bail!("unsupported operation: cannot cons {} with {}", left, right),
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
            Value::Tuple(_) => Value::Tuple(Vector::new()),
            Value::Dict(_) => Value::Dict(Vector::new()),
            Value::Set(_) => Value::Set(Vector::new()),
            Value::Range { .. } => Value::Range {
                start: Rational::new().into(),
                end: Some(Rational::new().into()),
                inclusive: false,
            },
            // A lazy cons cell behaves like a list view; the natural empty is
            // just the empty list.
            Value::Cons { .. } => Value::List(Vector::new()),
            v @ Value::Native { .. } | v @ Value::Function { .. } | v @ Value::Module { .. } => {
                bail!(
                    "unsupported operation: default is not supported for {},{},{}",
                    type_name(v),
                    type_name(v),
                    type_name(v)
                )
            }
        })
    })
}

fn print() -> Value {
    native!("io.print", 1, |env, args| {
        let s = &args[0].to_string();
        let ctx = ctx_of(env);
        if ctx.is_interactive {
            writeln!(ctx_of(env).out.borrow_mut(), "{}", s)
                .map_err(|e| anyhow::anyhow!("io error: {}", e))?;
        } else {
            write!(ctx_of(env).out.borrow_mut(), "{}", s)
                .map_err(|e| anyhow::anyhow!("io error: {}", e))?;
        }
        Ok(Value::Unit)
    })
}
fn println() -> Value {
    native!("io.println", 1, |env, args| {
        let s = &args[0].to_string();
        writeln!(ctx_of(env).out.borrow_mut(), "{}", s)
            .map_err(|e| anyhow::anyhow!("io error: {}", e))?;
        Ok(Value::Unit)
    })
}
