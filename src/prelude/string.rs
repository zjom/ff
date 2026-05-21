use crate::interop::FfResult;
use crate::interpreter::{RuntimeError, RuntimeResult, Value, force_tail, type_name};
use crate::{members, native};

members! {
    "String",
    of => native!(1, |_env, args| {
        Ok(Value::String(format!("{}", args[0]).into()))
    }),
    // Rust `format!`-style: `{}` interpolates the next arg, `{N}` the Nth,
    // `{:?}` / `{N:?}` use the debug-style display (strings are quoted, atoms
    // keep their `:`). `{{` and `}}` are literal braces.
    format => native!(2, |_env, args| {
        let Value::String(template) = &args[0] else {
            return Err(RuntimeError::NativeTypeError {
                native: "String.format",
                expected: "string",
                got: type_name(&args[0]),
            });
        };
        let template = template.to_string();
        let fargs = as_seq("String.format", &args[1])?;
        format_template(&template, &fargs).map(|s| Value::String(s.into()))
    }),
    len => |s: String| -> usize { s.chars().count() },
    upper => |s: String| -> String { s.to_uppercase() },
    lower => |s: String| -> String { s.to_lowercase() },
    trim => |s: String| -> String { s.trim().to_string() },
    trim_start => |s: String| -> String { s.trim_start().to_string() },
    trim_end => |s: String| -> String { s.trim_end().to_string() },
    contains => |needle: String, s: String| -> bool { s.contains(&needle) },
    starts_with => |prefix: String, s: String| -> bool { s.starts_with(&prefix) },
    ends_with => |suffix: String, s: String| -> bool { s.ends_with(&suffix) },
    replace => |from: String, to: String, s: String| -> String { s.replace(&from, &to) },
    split => |sep: String, s: String| -> Vec<String> {
        s.split(&sep).map(|p| p.to_string()).collect()
    },
    split_at => |i: usize, s: String| -> Vec<String> {
        s.split_at_checked(i).map(|(a,b)| vec![a.into(),b.into()]).unwrap_or(vec![s, String::new()])
    },
    lines => |s: String| -> Vec<String> {
        s.lines().map(|l| l.to_string()).collect()
    },
    join => |sep: String, parts: Vec<String>| -> String { parts.join(&sep) },
    chars => |s: String| -> Vec<String> {
        s.chars().map(|c| c.to_string()).collect()
    },
    repeat => |n: usize, s: String| -> String { s.repeat(n) },
    reverse => |s: String| -> String { s.chars().rev().collect() },
    // Char-position slice; bounds are clamped to [0, len] and a reversed range
    // yields the empty string.
    slice => |start: i64, end: i64, s: String| -> String {
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len() as i64;
        let start = start.clamp(0, n) as usize;
        let end = end.clamp(0, n) as usize;
        if start >= end { String::new() } else { chars[start..end].iter().collect() }
    },
    // Returns the char-index of the first occurrence, or `()` if not found.
    index_of => |needle: String, s: String| -> Option<usize> {
        s.find(&needle).map(|byte_pos| s[..byte_pos].chars().count())
    },
    parse_int => |s: String| -> FfResult<i64> {
        s.trim().parse::<i64>().map_err(|e| e.to_string()).into()
    },
    parse_float => |s: String| -> FfResult<f64> {
        s.trim().parse::<f64>().map_err(|e| e.to_string()).into()
    },
}

fn as_seq(native: &'static str, v: &Value) -> RuntimeResult<Vec<Value>> {
    match v {
        Value::List(xs) => Ok(xs.iter().cloned().collect()),
        Value::Cons { head, tail } => {
            let mut items = vec![(**head).clone()];
            let mut cur = tail.clone();
            loop {
                match force_tail(&cur)? {
                    Value::Cons { head, tail } => {
                        items.push((*head).clone());
                        cur = tail;
                    }
                    Value::List(xs) => {
                        items.extend(xs.iter().cloned());
                        return Ok(items);
                    }
                    Value::Unit => return Ok(items),
                    other => {
                        return Err(RuntimeError::UnsupportedOperation(format!(
                            "cons tail is not a list: {}",
                            other
                        )));
                    }
                }
            }
        }
        other => Err(RuntimeError::NativeTypeError {
            native,
            expected: "list",
            got: type_name(other),
        }),
    }
}

fn format_template(template: &str, args: &[Value]) -> RuntimeResult<String> {
    let mut out = String::new();
    let mut chars = template.chars().peekable();
    let mut next_idx: usize = 0;
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    out.push('{');
                    continue;
                }
                let mut spec = String::new();
                let mut closed = false;
                for nc in chars.by_ref() {
                    if nc == '}' {
                        closed = true;
                        break;
                    }
                    spec.push(nc);
                }
                if !closed {
                    return Err(RuntimeError::UnsupportedOperation(
                        "String.format: unclosed `{` in template".into(),
                    ));
                }
                let (idx, debug) = parse_spec(&spec, &mut next_idx)?;
                let v = args.get(idx).ok_or_else(|| {
                    RuntimeError::UnsupportedOperation(format!(
                        "String.format: missing argument {} (got {} arg(s))",
                        idx,
                        args.len()
                    ))
                })?;
                if debug {
                    out.push_str(&v.to_string());
                } else {
                    push_display(&mut out, v);
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    out.push('}');
                } else {
                    return Err(RuntimeError::UnsupportedOperation(
                        "String.format: stray `}` in template (use `}}` for a literal)".into(),
                    ));
                }
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

fn parse_spec(spec: &str, next_idx: &mut usize) -> RuntimeResult<(usize, bool)> {
    let (idx_part, fmt_part) = match spec.find(':') {
        Some(i) => (&spec[..i], &spec[i + 1..]),
        None => (spec, ""),
    };
    let idx = if idx_part.is_empty() {
        let i = *next_idx;
        *next_idx += 1;
        i
    } else {
        idx_part.parse::<usize>().map_err(|_| {
            RuntimeError::UnsupportedOperation(format!(
                "String.format: invalid argument index `{}`",
                idx_part
            ))
        })?
    };
    let debug = match fmt_part {
        "" => false,
        "?" => true,
        other => {
            return Err(RuntimeError::UnsupportedOperation(format!(
                "String.format: unsupported spec `:{}`",
                other
            )));
        }
    };
    Ok((idx, debug))
}

// Rust-`Display`-flavored rendering: strings drop their quotes so they splice
// cleanly into the output. Everything else uses the value's default display.
fn push_display(out: &mut String, v: &Value) {
    match v {
        Value::String(s) => out.push_str(s),
        other => out.push_str(&other.to_string()),
    }
}
