use anyhow::bail;

use crate::interpreter::{Value, apply, ctx_of};
use crate::native;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        ("|>", pipe()),
        ("::", cons()),
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
            (v, Value::List(xs)) => {
                let mut ys = xs;
                ys.insert(0, v);
                Ok(Value::List(ys))
            }

            (Value::String(mut left), Value::String(right)) => {
                left.push_str(right.as_str());
                Ok(Value::String(left))
            }

            (v, Value::Tuple(xs)) => {
                let mut ys = xs;
                ys.insert(0, v);
                Ok(Value::Tuple(ys))
            }

            (v, Value::Set(xs)) => {
                if xs.iter().any(|x| &v == x) {
                    Ok(Value::Set(xs))
                } else {
                    let mut ys = xs;
                    ys.push(v);
                    Ok(Value::Set(ys))
                }
            }
            (Value::Tuple(kvs), Value::Dict(mut xs)) => {
                let (key, value) = (kvs[0].clone(), kvs[1].clone());
                if kvs.len() == 2 {
                    xs.retain(|(k, _)| k != &key);
                    xs.push((key, value));
                    Ok(Value::Dict(xs))
                } else {
                    bail!("unsupported operation: can only cons a length 2 tuple with a dict")
                }
            }
            (left, right) => bail!("unsupported operation: cannot cons {} with {}", left, right),
        }
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
