use im::Vector;

use crate::interpreter::{LazyState, RuntimeResult};
use crate::native;
use crate::prelude::{err_str_tuple, object, ok_tuple};
use std::io::{BufReader, Stdin};
use std::io::{Bytes, Read};
use std::sync::{Arc, Mutex};

use crate::prelude::Value;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![("stdin", stdin())]
}

fn stdin() -> Value {
    native!("Io.stdin", 0, move |_, _| Ok(stdin_object()))
}

fn stdin_object() -> Value {
    let entries = vec![("bytes", stdin_bytes_fn())];

    object(entries)
}

fn stdin_bytes_fn() -> Value {
    native!("stdin.bytes", 0, move |_env, _args| {
        let reader = BufReader::new(std::io::stdin());
        Ok(ok_tuple(bytes_stream(reader.bytes())?))
    })
}

fn bytes_stream(mut bytes: Bytes<BufReader<Stdin>>) -> RuntimeResult<Value> {
    match bytes.next() {
        Some(res) => match res {
            Ok(p) => {
                let tail = Arc::new(Mutex::new(LazyState::Native(Box::new(move || {
                    bytes_stream(bytes)
                }))));
                Ok(Value::Cons {
                    head: Arc::new(ok_tuple(Value::Number(Arc::new(p.into())))),
                    tail,
                })
            }
            Err(e) => Ok(err_str_tuple(e.to_string())),
        },
        None => Ok(Value::List(Vector::new())),
    }
}
