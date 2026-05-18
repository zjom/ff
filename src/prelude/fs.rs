use crate::prelude::{dict, err, ok};
use std::cell::RefCell;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::rc::Rc;

use anyhow::bail;
use im::{Vector, vector};
use rug::Rational;

use crate::interpreter::{LazyState, Value, type_name};
use crate::native;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![("open", open_file())]
}

fn open_file() -> Value {
    native!("open", 1, |_env, args| {
        let Value::String(path) = &args[0] else {
            bail!("open_file expected a string, got {}", type_name(&args[0]));
        };
        if !Path::new(&**path).exists() {
            err("file not exists");
        }
        Ok(file_dict(path.clone()))
    })
}

fn file_dict(path: Rc<str>) -> Value {
    let entries = vec![
        ("path", Value::String(path.clone())),
        ("write", write_fn(path.clone())),
        ("append", append_fn(path.clone())),
        ("read", read_fn(path.clone())),
        ("lines", lines_fn(path.clone())),
        ("metadata", metadata_fn(path)),
    ];
    dict(entries)
}

fn write_fn(path: Rc<str>) -> Value {
    native!("file.write", 1, move |_env, args| {
        match args[0] {
            Value::String(ref data) => Ok(match fs::write(&*path, &**data) {
                Ok(()) => ok(Value::Unit),
                Err(e) => err(e.to_string()),
            }),
            _ => {
                bail!("file.write expected a string, got {}", type_name(&args[0]));
            }
        }
    })
}

fn append_fn(path: Rc<str>) -> Value {
    native!("file.append", 1, move |_env, args| {
        match args[0] {
            Value::String(ref data) => Ok(match fs::OpenOptions::new().append(true).open(&*path) {
                Ok(mut f) => match f.write_all(data.as_bytes()) {
                    Ok(()) => ok(Value::Unit),
                    Err(e) => err(e.to_string()),
                },
                Err(e) => err(e.to_string()),
            }),
            _ => {
                bail!("file.write expected a string, got {}", type_name(&args[0]));
            }
        }
    })
}

fn read_fn(path: Rc<str>) -> Value {
    native!("file.read", 0, move |_env, _args| {
        Ok(match fs::read_to_string(&*path) {
            Ok(s) => ok(Value::String(s.into())),
            Err(e) => err(e.to_string()),
        })
    })
}

fn lines_fn(path: Rc<str>) -> Value {
    native!("file.lines", 0, move |_env, _args| {
        Ok(match File::open(&*path) {
            Ok(f) => ok(lines_stream(BufReader::new(f))?),
            Err(e) => err(e.to_string()),
        })
    })
}

// Walk a `BufReader` one line at a time, producing a `Cons` spine whose tail
// is a native thunk capturing the (advanced) reader. EOF terminates the spine
// with `Value::List(empty)` so consumers that flatten or pattern-match see a
// proper sequence terminator.
fn lines_stream(mut reader: BufReader<File>) -> anyhow::Result<Value> {
    let mut buf = String::new();
    match reader.read_line(&mut buf) {
        Ok(0) => Ok(Value::List(Vector::new())),
        Ok(_) => {
            strip_line_ending(&mut buf);
            let tail = Rc::new(RefCell::new(LazyState::Native(Box::new(move || {
                lines_stream(reader)
            }))));
            Ok(Value::Cons {
                head: Rc::new(Value::String(buf.into())),
                tail,
            })
        }
        Err(e) => bail!("file.lines: {}", e),
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

fn metadata_fn(path: Rc<str>) -> Value {
    native!("file.metadata", 0, move |_env, _args| {
        Ok(match fs::metadata(&*path) {
            Ok(m) => {
                let entries = vector![
                    (
                        Value::String("size".into()),
                        Value::Number(Rc::new(Rational::from(m.len()))),
                    ),
                    (Value::String("is_file".into()), Value::Bool(m.is_file())),
                    (Value::String("is_dir".into()), Value::Bool(m.is_dir())),
                ];
                ok(Value::Dict(entries))
            }
            Err(e) => err(e.to_string()),
        })
    })
}
