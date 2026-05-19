use im::Vector;
use std::io::BufRead;

use crate::interpreter::{LazyState, RuntimeResult};
use crate::native;
use crate::prelude::{err_str_tuple, object, ok_tuple};
use std::io::BufReader;
use std::io::{Bytes, Read};
use std::sync::{Arc, Mutex};

use crate::prelude::Value;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![("stdin", stdin())]
}

fn stdin() -> Value {
    native!("Io.stdin", 0, move |_, _| Ok(Stdin::object()))
}

struct Stdin {}
impl Stdin {
    fn object() -> Value {
        let entries = vec![("bytes", Self::bytes_fn()), ("lines", Self::lines_fn())];

        object(entries)
    }

    fn bytes_fn() -> Value {
        native!("stdin.bytes", 0, move |_env, _args| {
            let reader = BufReader::new(std::io::stdin());
            Ok(ok_tuple(Self::bytes_stream(reader.bytes())?))
        })
    }
    fn bytes_stream(mut bytes: Bytes<BufReader<std::io::Stdin>>) -> RuntimeResult<Value> {
        match bytes.next() {
            Some(res) => match res {
                Ok(p) => {
                    let tail = Arc::new(Mutex::new(LazyState::Native(Box::new(move || {
                        Self::bytes_stream(bytes)
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

    fn lines_fn() -> Value {
        native!("stdin.lines", 0, move |_env, _args| {
            Ok(ok_tuple(Self::lines_stream(BufReader::new(
                std::io::stdin(),
            ))?))
        })
    }

    // Walk a `BufReader` one line at a time, producing a `Cons` spine whose tail
    // is a native thunk capturing the (advanced) reader. EOF terminates the spine
    // with `Value::List(empty)` so consumers that flatten or pattern-match see a
    // proper sequence terminator.
    fn lines_stream(mut reader: BufReader<std::io::Stdin>) -> RuntimeResult<Value> {
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Ok(0) => Ok(Value::List(Vector::new())),
            Ok(_) => {
                strip_line_ending(&mut buf);
                let tail = Arc::new(Mutex::new(LazyState::Native(Box::new(move || {
                    Self::lines_stream(reader)
                }))));
                Ok(Value::Cons {
                    head: Arc::new(ok_tuple(Value::String(buf.into()))),
                    tail,
                })
            }
            Err(e) => Ok(err_str_tuple(e.to_string())),
        }
    }
}

fn strip_line_ending(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
