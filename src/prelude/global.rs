use anyhow::bail;

use crate::interpreter::{Value, apply, ctx_of};
use crate::native;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        ("|>", pipe()),
        ("++", cons()),
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
    native!("++", 2, |_env, args| {
        match (args[0].clone(), args[1].clone()) {
            (v, Value::List(xs)) => {
                let mut ys = xs;
                ys.insert(0, v);
                Ok(Value::List(ys))
            }
            _ => bail!("can only cons to a list"),
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
