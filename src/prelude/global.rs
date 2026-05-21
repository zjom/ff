use im::{HashMap, HashSet, Vector};
use rug::Rational;
use std::sync::Arc;

use crate::interpreter::{RuntimeError, Value, ctx_of, rat_mod, rat_pow, type_name};
use crate::{members, native};

members! {
    "",
    "::" => native!(2, |_env, args| {
        match (args[0].clone(), args[1].clone()) {
            (v, Value::List(mut xs)) => {
                xs.push_front(v);
                Ok(Value::List(xs))
            }

            (Value::String(left), Value::String(right)) => {
                Ok(Value::String(format!("{}{}", left, right).into()))
            }

            (v, Value::Set(mut xs)) => {
                xs.insert(v);
                Ok(Value::Set(xs))
            }
            (Value::List(kvs), Value::Object(mut xs)) => {
                if kvs.len() == 2 {
                    let (key, value) = (kvs[0].clone(), kvs[1].clone());
                    xs.insert(key, value);
                    Ok(Value::Object(xs))
                } else {
                    Err(RuntimeError::UnsupportedOperation(
                        "can only cons a 2-element list with an object".into(),
                    ))
                }
            }

            (left, right) => Err(RuntimeError::UnsupportedOperation(format!(
                "cannot cons {} with {}",
                left, right
            ))),
        }
    }),

    // ---- arithmetic --------------------------------------------------------
    // All arithmetic ops are number-only. String concatenation lives on `::`
    // (which already handles `string :: string`); `+` is reserved for numbers
    // so the operator stays type-clean.
    "+" => native!(2, |_env, args| {
        let (a, b) = num_pair("+", &args[0], &args[1])?;
        Ok(Value::Number(Arc::new(Rational::from(a.as_ref() + b.as_ref()))))
    }),
    "-" => native!(2, |_env, args| {
        let (a, b) = num_pair("-", &args[0], &args[1])?;
        Ok(Value::Number(Arc::new(Rational::from(a.as_ref() - b.as_ref()))))
    }),
    "*" => native!(2, |_env, args| {
        let (a, b) = num_pair("*", &args[0], &args[1])?;
        Ok(Value::Number(Arc::new(Rational::from(a.as_ref() * b.as_ref()))))
    }),
    "/" => native!(2, |_env, args| {
        let (a, b) = num_pair("/", &args[0], &args[1])?;
        if b.cmp0() == std::cmp::Ordering::Equal {
            return Err(RuntimeError::DivisionByZero);
        }
        Ok(Value::Number(Arc::new(Rational::from(a.as_ref() / b.as_ref()))))
    }),
    "%" => native!(2, |_env, args| {
        let (a, b) = num_pair("%", &args[0], &args[1])?;
        if b.cmp0() == std::cmp::Ordering::Equal {
            return Err(RuntimeError::ModuloByZero);
        }
        Ok(Value::Number(Arc::new(rat_mod(a, b))))
    }),
    "**" => native!(2, |_env, args| {
        let (a, b) = num_pair("**", &args[0], &args[1])?;
        Ok(Value::Number(Arc::new(rat_pow(a, b)?)))
    }),

    // ---- equality / comparison --------------------------------------------
    "==" => native!(2, |_env, args| Ok(Value::Bool(args[0] == args[1]))),
    "!=" => native!(2, |_env, args| Ok(Value::Bool(args[0] != args[1]))),
    "<" => native!(2, |_env, args| ord_cmp("<", &args[0], &args[1], |o| o == std::cmp::Ordering::Less)),
    "<=" => native!(2, |_env, args| ord_cmp("<=", &args[0], &args[1], |o| o != std::cmp::Ordering::Greater)),
    ">" => native!(2, |_env, args| ord_cmp(">", &args[0], &args[1], |o| o == std::cmp::Ordering::Greater)),
    ">=" => native!(2, |_env, args| ord_cmp(">=", &args[0], &args[1], |o| o != std::cmp::Ordering::Less)),

    // ---- substring match --------------------------------------------------
    "~" => native!(2, |_env, args| {
        let (a, b) = str_pair("~", &args[0], &args[1])?;
        Ok(Value::Bool(a.contains(b.as_ref())))
    }),
    "!~" => native!(2, |_env, args| {
        let (a, b) = str_pair("!~", &args[0], &args[1])?;
        Ok(Value::Bool(!a.contains(b.as_ref())))
    }),

    print => native!(1, |env, args| {
        let s = &args[0].to_string();
        let ctx = ctx_of(env);
        if ctx.is_interactive {
            writeln!(ctx_of(env).out.lock().unwrap(), "{}", s)?;
        } else {
            write!(ctx_of(env).out.lock().unwrap(), "{}", s)?;
        }
        Ok(Value::Unit)
    }),
    println => native!(1, |env, args| {
        let s = &args[0].to_string();
        writeln!(ctx_of(env).out.lock().unwrap(), "{}", s)?;
        Ok(Value::Unit)
    }),
    "typeof" => native!(1, |_env, args| {
        Ok(Value::Atom(type_name(&args[0]).into()))
    }),
    panic => native!(1, |_env, args| {
        Err(RuntimeError::Panic(args[0].to_string()))
    }),
    default => native!(1, |_env, args| {
        Ok(match &args[0] {
            Value::Unit => Value::Unit,
            Value::Number(_) => Value::Number(Rational::new().into()),
            Value::String(_) => Value::String("".into()),
            Value::Bool(_) => Value::Bool(false),
            Value::List(_) => Value::List(Vector::new()),
            Value::Object(_) => Value::Object(HashMap::new()),
            Value::Set(_) => Value::Set(HashSet::new()),
            Value::Range { .. } => Value::Range {
                start: Rational::new().into(),
                end: Some(Rational::new().into()),
                inclusive: false,
            },
            // A lazy cons cell behaves like a list view; the natural empty is
            // just the empty list.
            Value::Cons { .. } => Value::List(Vector::new()),
            v @ Value::Atom(_)
            | v @ Value::Native { .. }
            | v @ Value::Function { .. }
            | v @ Value::Pid(_) => {
                let t = type_name(v);
                return Err(RuntimeError::UnsupportedOperation(format!(
                    "default is not supported for {},{},{}",
                    t, t, t
                )));
            }
        })
    }),
}

fn num_pair<'a>(
    native: &'static str,
    a: &'a Value,
    b: &'a Value,
) -> crate::interpreter::RuntimeResult<(&'a Arc<Rational>, &'a Arc<Rational>)> {
    let Value::Number(av) = a else {
        return Err(RuntimeError::NativeTypeError {
            native,
            expected: "number",
            got: type_name(a),
        });
    };
    let Value::Number(bv) = b else {
        return Err(RuntimeError::NativeTypeError {
            native,
            expected: "number",
            got: type_name(b),
        });
    };
    Ok((av, bv))
}

fn str_pair<'a>(
    native: &'static str,
    a: &'a Value,
    b: &'a Value,
) -> crate::interpreter::RuntimeResult<(&'a Arc<str>, &'a Arc<str>)> {
    let Value::String(av) = a else {
        return Err(RuntimeError::NativeTypeError {
            native,
            expected: "string",
            got: type_name(a),
        });
    };
    let Value::String(bv) = b else {
        return Err(RuntimeError::NativeTypeError {
            native,
            expected: "string",
            got: type_name(b),
        });
    };
    Ok((av, bv))
}

// `<`, `<=`, `>`, `>=` work for numbers and strings; reject everything else
// with a type error mentioning the operator.
fn ord_cmp(
    native: &'static str,
    a: &Value,
    b: &Value,
    pick: impl Fn(std::cmp::Ordering) -> bool,
) -> crate::interpreter::RuntimeResult<Value> {
    let ordering = match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.cmp(y),
        (Value::String(x), Value::String(y)) => x.cmp(y),
        _ => {
            return Err(RuntimeError::NativeTypeError {
                native,
                expected: "two numbers or two strings",
                got: type_name(a),
            });
        }
    };
    Ok(Value::Bool(pick(ordering)))
}
