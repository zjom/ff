use crate::interpreter::runtime::{self, Pid, RequestReply};
use crate::interpreter::{RuntimeError, Value, type_name};
use crate::native;
use crate::prelude::{err_atom_tuple, err_str_tuple, ok_tuple};

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        ("spawn", spawn()),
        ("notify", notify()),
        ("request", request()),
        ("self", self_()),
        ("alive", alive()),
        ("stop", stop()),
        ("run_until_idle", run_until_idle()),
    ]
}

fn request_reply_to_value(reply: RequestReply) -> Value {
    match reply {
        RequestReply::Ok(v) => ok_tuple(v),
        RequestReply::Err(s) => {
            if s.starts_with(':') {
                err_atom_tuple(&s)
            } else {
                err_str_tuple(s)
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
            Ok(pid) => Ok(ok_tuple(pid)),
            Err(e) => Ok(err_str_tuple(e.to_string())),
        }
    })
}

fn notify() -> Value {
    native!("Actor.notify", 2, |env, args| {
        let pid = expect_pid("Actor.notify", &args[0])?;
        runtime::notify(env, pid, args[1].clone());
        Ok(Value::Unit)
    })
}

fn request() -> Value {
    native!("Actor.request", 2, |env, args| {
        let pid = expect_pid("Actor.request", &args[0])?;
        let reply = runtime::request(env, pid, args[1].clone());
        Ok(request_reply_to_value(reply))
    })
}

fn self_() -> Value {
    native!("Actor.self", 0, |env, _args| {
        Ok(match runtime::current_pid(env) {
            Some(pid) => ok_tuple(Value::Pid(pid)),
            None => err_atom_tuple(":not_in_actor"),
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
