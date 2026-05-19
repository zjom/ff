use im::Vector;
use std::io::BufRead;

use crate::interop::{FfResult, native_fn};
use crate::interpreter::{LazyState, RuntimeResult};
use crate::native;
use crate::prelude::{err_str_tuple, object, ok_tuple};
use std::io::BufReader;
use std::io::{Bytes, Read, Write};
use std::sync::{Arc, Mutex};

use crate::prelude::Value;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        ("stdin", stdin()),
        ("stdout", stdout()),
        ("stderr", stderr()),
    ]
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

fn stdout() -> Value {
    native!("Io.stdout", 0, move |_, _| Ok(Stdout::object()))
}

struct Stdout {}
impl Stdout {
    fn object() -> Value {
        let entries = vec![
            ("write", Self::write_fn()),
            ("writeln", Self::writeln_fn()),
            ("flush", Self::flush_fn()),
        ];
        object(entries)
    }

    fn write_fn() -> Value {
        native_fn("stdout.write", move |s: String| -> FfResult<()> {
            std::io::stdout().write_all(s.as_bytes()).into()
        })
    }

    fn writeln_fn() -> Value {
        native_fn("stdout.writeln", move |s: String| -> FfResult<()> {
            writeln!(std::io::stdout(), "{}", s).into()
        })
    }

    fn flush_fn() -> Value {
        native_fn("stdout.flush", move || -> FfResult<()> {
            std::io::stdout().flush().into()
        })
    }
}

fn stderr() -> Value {
    native!("Io.stderr", 0, move |_, _| Ok(Stderr::object()))
}

struct Stderr {}
impl Stderr {
    fn object() -> Value {
        let entries = vec![
            ("write", Self::write_fn()),
            ("writeln", Self::writeln_fn()),
            ("flush", Self::flush_fn()),
        ];
        object(entries)
    }

    fn write_fn() -> Value {
        native_fn("stderr.write", move |s: String| -> FfResult<()> {
            std::io::stderr().write_all(s.as_bytes()).into()
        })
    }

    fn writeln_fn() -> Value {
        native_fn("stderr.writeln", move |s: String| -> FfResult<()> {
            writeln!(std::io::stderr(), "{}", s).into()
        })
    }

    fn flush_fn() -> Value {
        native_fn("stderr.flush", move || -> FfResult<()> {
            std::io::stderr().flush().into()
        })
    }
}
