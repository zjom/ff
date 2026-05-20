use crate::interpreter::runtime::{self, Pid, RequestReply};
use crate::interpreter::{RuntimeError, Value, type_name};
use crate::prelude::{err_atom_tuple, err_str_tuple, ok_tuple};
use crate::{members, native};

members! {
    "Actor",
    spawn => native!(1, |env, args| {
        match runtime::spawn(env, args[0].clone()) {
            Ok(pid) => Ok(ok_tuple(pid)),
            Err(e) => Ok(err_str_tuple(e.to_string())),
        }
    }),
    notify => native!(2, |env, args| {
        let pid = expect_pid("Actor.notify", &args[0])?;
        runtime::notify(env, pid, args[1].clone());
        Ok(Value::Unit)
    }),
    request => native!(2, |env, args| {
        let pid = expect_pid("Actor.request", &args[0])?;
        let reply = runtime::request(env, pid, args[1].clone());
        Ok(request_reply_to_value(reply))
    }),
    "self" => native!(0, |env, _args| {
        Ok(match runtime::current_pid(env) {
            Some(pid) => ok_tuple(Value::Pid(pid)),
            None => err_atom_tuple(":not_in_actor"),
        })
    }),
    alive => native!(1, |env, args| {
        let pid = expect_pid("Actor.alive", &args[0])?;
        Ok(Value::Bool(runtime::alive(env, pid)))
    }),
    stop => native!(1, |env, args| {
        let pid = expect_pid("Actor.stop", &args[0])?;
        runtime::stop(env, pid);
        Ok(Value::Unit)
    }),
    run_until_idle => native!(0, |env, _args| {
        runtime::drain_all(env);
        Ok(Value::Unit)
    }),
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
