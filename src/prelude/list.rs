use im::Vector;
use rug::Rational;
use std::sync::Arc;

use crate::interpreter::{RuntimeError, RuntimeResult, Value, force_tail, type_name};
use crate::{members, native};

members! {
    "List",
    len => native!(1, move |_env, args| {
        let xs = as_vec("List.len", &args[0])?;
        Ok(num(xs.len()))
    }),
    head => native!(1, move |_env, args| {
        let xs = as_vec("List.head", &args[0])?;
        Ok(xs.into_iter().next().unwrap_or(Value::Unit))
    }),
    tail => native!(1, move |_env, args| {
        let xs = as_vec("List.tail", &args[0])?;
        if xs.is_empty() {
            Ok(Value::Unit)
        } else {
            Ok(list_value(xs.into_iter().skip(1).collect()))
        }
    }),
    last => native!(1, move |_env, args| {
        let xs = as_vec("List.last", &args[0])?;
        Ok(xs.into_iter().next_back().unwrap_or(Value::Unit))
    }),
    nth => native!(2, move |_env, args| {
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
    }),
    is_empty => native!(1, move |_env, args| {
        let xs = as_vec("List.is_empty", &args[0])?;
        Ok(Value::Bool(xs.is_empty()))
    }),
    contains => native!(2, move |_env, args| {
        let xs = as_vec("List.contains", &args[1])?;
        Ok(Value::Bool(xs.iter().any(|x| x == &args[0])))
    }),
    index_of => native!(2, move |_env, args| {
        let xs = as_vec("List.index_of", &args[1])?;
        Ok(xs
            .iter()
            .position(|x| x == &args[0])
            .map(num)
            .unwrap_or(Value::Unit))
    }),
    reverse => native!(1, move |_env, args| {
        let mut xs = as_vec("List.reverse", &args[0])?;
        xs.reverse();
        Ok(list_value(xs))
    }),
    concat => native!(2, move |_env, args| {
        let mut a = as_vec("List.concat", &args[0])?;
        let b = as_vec("List.concat", &args[1])?;
        a.extend(b);
        Ok(list_value(a))
    }),
    flatten => native!(1, move |_env, args| {
        let outer = as_vec("List.flatten", &args[0])?;
        let mut out: Vector<Value> = Vector::new();
        for x in outer {
            for item in as_vec("List.flatten", &x)? {
                out.push_back(item);
            }
        }
        Ok(Value::List(out))
    }),
    zip => native!(2, move |_env, args| {
        let a = as_vec("List.zip", &args[0])?;
        let b = as_vec("List.zip", &args[1])?;
        let pairs: Vector<Value> = a
            .into_iter()
            .zip(b)
            .map(|(x, y)| Value::List(im::vector![x, y]))
            .collect();
        Ok(Value::List(pairs))
    }),
    drop => native!(2, move |_env, args| {
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
    }),
    take => native!(2, move |_env, args| {
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
    }),
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
