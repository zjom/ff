use im::vector;
use std::rc::Rc;

use crate::interpreter::runtime::{self, CallReply, Pid};
use crate::interpreter::{RuntimeError, Value, type_name};
use crate::native;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        ("spawn", spawn()),
        ("cast", cast()),
        ("call", call()),
        ("self", self_()),
        ("alive", alive()),
        ("stop", stop()),
        ("run_until_idle", run_until_idle()),
    ]
}

fn ok_atom_or_value(v: Value) -> Value {
    Value::List(vector![Value::Atom("ok".into()), v])
}

// `[:error, :tag]` — distinct from the string-bearing `err` helper so
// reasons like `:no_proc` stay matchable as atoms.
fn err_atom(tag: &str) -> Value {
    let trimmed = tag.strip_prefix(':').unwrap_or(tag);
    Value::List(vector![
        Value::Atom("error".into()),
        Value::Atom(trimmed.into()),
    ])
}

fn err_msg(msg: impl Into<Rc<str>>) -> Value {
    Value::List(vector![
        Value::Atom("error".into()),
        Value::String(msg.into()),
    ])
}

fn call_reply_to_value(reply: CallReply) -> Value {
    match reply {
        CallReply::Ok(v) => ok_atom_or_value(v),
        CallReply::Err(s) => {
            if s.starts_with(':') {
                err_atom(&s)
            } else {
                err_msg(s)
            }
        }
    }
}

fn expect_pid(native: &'static str, v: &Value) -> Result<Pid, RuntimeError> {
    if let Value::Pid(id) = v {
        Ok(*id)
    } else {
        Err(RuntimeError::NativeTypeError {
            native,
            expected: "pid",
            got: type_name(v),
        })
    }
}

fn spawn() -> Value {
    native!("Actor.spawn", 1, |env, args| {
        match runtime::spawn(env, args[0].clone()) {
            Ok(pid) => Ok(ok_atom_or_value(pid)),
            Err(e) => Ok(err_msg(e.to_string())),
        }
    })
}

fn cast() -> Value {
    native!("Actor.cast", 2, |env, args| {
        let pid = expect_pid("Actor.cast", &args[0])?;
        runtime::cast(env, pid, args[1].clone());
        Ok(Value::Unit)
    })
}

fn call() -> Value {
    native!("Actor.call", 2, |env, args| {
        let pid = expect_pid("Actor.call", &args[0])?;
        let reply = runtime::call(env, pid, args[1].clone());
        Ok(call_reply_to_value(reply))
    })
}

fn self_() -> Value {
    native!("Actor.self", 0, |env, _args| {
        Ok(match runtime::current_pid(env) {
            Some(pid) => ok_atom_or_value(Value::Pid(pid)),
            None => err_atom(":not_in_actor"),
        })
    })
}

fn alive() -> Value {
    native!("Actor.alive", 1, |env, args| {
        let pid = expect_pid("Actor.alive", &args[0])?;
        Ok(Value::Bool(runtime::alive(env, pid)))
    })
}

fn stop() -> Value {
    native!("Actor.stop", 1, |env, args| {
        let pid = expect_pid("Actor.stop", &args[0])?;
        runtime::stop(env, pid);
        Ok(Value::Unit)
    })
}

fn run_until_idle() -> Value {
    native!("Actor.run_until_idle", 0, |env, _args| {
        runtime::drain_all(env);
        Ok(Value::Unit)
    })
}
