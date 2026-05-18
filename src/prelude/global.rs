use anyhow::{Result, bail};
use im::{Vector, vector};

use crate::interpreter::{Env, Value, apply, ctx_of, type_name};
use crate::native;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        ("|>", pipe()),
        ("::", cons()),
        ("filter", filter()),
        ("print", print()),
        ("println", println()),
    ]
}

fn pipe() -> Value {
    native!("|>", 2, |env, args| {
        apply(env, args[1].clone(), vec![args[0].clone()])
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

fn filter() -> Value {
    native!("filter", 2, |env, args| {
        let f = args[0].clone();
        match args[1].clone() {
            Value::List(xs) => Ok(Value::List(keep_each(env, &f, xs)?)),
            Value::Tuple(xs) => Ok(Value::Tuple(keep_each(env, &f, xs)?)),
            Value::Set(xs) => Ok(Value::Set(keep_each(env, &f, xs)?)),
            Value::Dict(es) => {
                let mut out = Vector::new();
                for (k, v) in es {
                    let pair = Value::Tuple(vector![k.clone(), v.clone()]);
                    if keep(env, &f, pair)? {
                        out.push_back((k, v));
                    }
                }
                Ok(Value::Dict(out))
            }
            Value::String(s) => {
                let mut out = String::new();
                for ch in s.chars() {
                    if keep(env, &f, Value::String(ch.to_string().into()))? {
                        out.push(ch);
                    }
                }
                Ok(Value::String(out.into()))
            }
            other => bail!("filter: unsupported collection {}", type_name(&other)),
        }
    })
}

fn keep_each(env: &Env, f: &Value, xs: Vector<Value>) -> Result<Vector<Value>> {
    let mut out = Vector::new();
    for x in xs {
        if keep(env, f, x.clone())? {
            out.push_back(x);
        }
    }
    Ok(out)
}

fn keep(env: &Env, f: &Value, x: Value) -> Result<bool> {
    match apply(env, f.clone(), vec![x])? {
        Value::Bool(b) => Ok(b),
        v => bail!("filter: predicate must return bool, got {}", type_name(&v)),
    }
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
