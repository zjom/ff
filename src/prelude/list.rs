use im::Vector;
use rug::Rational;
use std::sync::Arc;

use crate::interpreter::{RuntimeError, RuntimeResult, Value, force_tail, type_name};
use crate::native;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![
        ("len", len()),
        ("head", head()),
        ("tail", tail()),
        ("last", last()),
        ("nth", nth()),
        ("is_empty", is_empty()),
        ("contains", contains()),
        ("index_of", index_of()),
        ("reverse", reverse()),
        ("concat", concat()),
        ("flatten", flatten()),
        ("zip", zip()),
        ("drop", drop_n()),
        ("take", take_n()),
    ]
}

// Materialize a ff sequence (List or Cons-spine) into a Vec. Refuses other types.
fn as_vec(native: &'static str, v: &Value) -> RuntimeResult<Vec<Value>> {
    match v {
        Value::List(xs) => Ok(xs.iter().cloned().collect()),
        Value::Cons { head, tail } => {
            let mut items = vec![(**head).clone()];
            let mut cur = tail.clone();
            loop {
                let forced = force_tail(&cur)?;
                match forced {
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

fn num(n: usize) -> Value {
    Value::Number(Arc::new(Rational::from(n)))
}

fn list_value(items: Vec<Value>) -> Value {
    Value::List(items.into_iter().collect())
}

fn values_eq(a: &Value, b: &Value) -> bool {
    a == b
}

fn len() -> Value {
    native!("List.len", 1, move |_env, args| {
        let xs = as_vec("List.len", &args[0])?;
        Ok(num(xs.len()))
    })
}

fn head() -> Value {
    native!("List.head", 1, move |_env, args| {
        let xs = as_vec("List.head", &args[0])?;
        Ok(xs.into_iter().next().unwrap_or(Value::Unit))
    })
}

fn tail() -> Value {
    native!("List.tail", 1, move |_env, args| {
        let xs = as_vec("List.tail", &args[0])?;
        if xs.is_empty() {
            Ok(Value::Unit)
        } else {
            Ok(list_value(xs.into_iter().skip(1).collect()))
        }
    })
}

fn last() -> Value {
    native!("List.last", 1, move |_env, args| {
        let xs = as_vec("List.last", &args[0])?;
        Ok(xs.into_iter().next_back().unwrap_or(Value::Unit))
    })
}

fn nth() -> Value {
    native!("List.nth", 2, move |_env, args| {
        let Value::Number(n) = &args[0] else {
            return Err(RuntimeError::NativeTypeError {
                native: "List.nth",
                expected: "number",
                got: type_name(&args[0]),
            });
        };
        let Some(i) = n.numer().to_i64() else {
            return Ok(Value::Unit);
        };
        let xs = as_vec("List.nth", &args[1])?;
        if i < 0 {
            return Ok(Value::Unit);
        }
        Ok(xs.into_iter().nth(i as usize).unwrap_or(Value::Unit))
    })
}

fn is_empty() -> Value {
    native!("List.is_empty", 1, move |_env, args| {
        let xs = as_vec("List.is_empty", &args[0])?;
        Ok(Value::Bool(xs.is_empty()))
    })
}

fn contains() -> Value {
    native!("List.contains", 2, move |_env, args| {
        let xs = as_vec("List.contains", &args[1])?;
        Ok(Value::Bool(xs.iter().any(|x| values_eq(x, &args[0]))))
    })
}

fn index_of() -> Value {
    native!("List.index_of", 2, move |_env, args| {
        let xs = as_vec("List.index_of", &args[1])?;
        Ok(xs
            .iter()
            .position(|x| values_eq(x, &args[0]))
            .map(num)
            .unwrap_or(Value::Unit))
    })
}

fn reverse() -> Value {
    native!("List.reverse", 1, move |_env, args| {
        let mut xs = as_vec("List.reverse", &args[0])?;
        xs.reverse();
        Ok(list_value(xs))
    })
}

fn concat() -> Value {
    native!("List.concat", 2, move |_env, args| {
        let mut a = as_vec("List.concat", &args[0])?;
        let b = as_vec("List.concat", &args[1])?;
        a.extend(b);
        Ok(list_value(a))
    })
}

fn flatten() -> Value {
    native!("List.flatten", 1, move |_env, args| {
        let outer = as_vec("List.flatten", &args[0])?;
        let mut out: Vector<Value> = Vector::new();
        for x in outer {
            for item in as_vec("List.flatten", &x)? {
                out.push_back(item);
            }
        }
        Ok(Value::List(out))
    })
}

fn zip() -> Value {
    native!("List.zip", 2, move |_env, args| {
        let a = as_vec("List.zip", &args[0])?;
        let b = as_vec("List.zip", &args[1])?;
        let pairs: Vector<Value> = a
            .into_iter()
            .zip(b)
            .map(|(x, y)| Value::List(im::vector![x, y]))
            .collect();
        Ok(Value::List(pairs))
    })
}

fn drop_n() -> Value {
    native!("List.drop", 2, move |_env, args| {
        let Value::Number(n) = &args[0] else {
            return Err(RuntimeError::NativeTypeError {
                native: "List.drop",
                expected: "number",
                got: type_name(&args[0]),
            });
        };
        let n = n.numer().to_i64().unwrap_or(0).max(0) as usize;
        let xs = as_vec("List.drop", &args[1])?;
        Ok(list_value(xs.into_iter().skip(n).collect()))
    })
}

fn take_n() -> Value {
    native!("List.take", 2, move |_env, args| {
        let Value::Number(n) = &args[0] else {
            return Err(RuntimeError::NativeTypeError {
                native: "List.take",
                expected: "number",
                got: type_name(&args[0]),
            });
        };
        let n = n.numer().to_i64().unwrap_or(0).max(0) as usize;
        let xs = as_vec("List.take", &args[1])?;
        Ok(list_value(xs.into_iter().take(n).collect()))
    })
}
