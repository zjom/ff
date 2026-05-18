use im::Vector;
use rug::Rational;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Expr;

use super::number::format_rational;
use super::scope::Env;

pub enum LazyState {
    Pending { body: Expr, env: Env },
    Forced(Value),
    // A native-built thunk. Used by stdlib streams (e.g. `file.lines`) that
    // can't be expressed as an AST expression because they carry Rust state
    // like an open `BufReader`.
    Native(Box<dyn FnOnce() -> anyhow::Result<Value>>),
}

impl std::fmt::Debug for LazyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LazyState::Pending { body, env } => f
                .debug_struct("Pending")
                .field("body", body)
                .field("env", env)
                .finish(),
            LazyState::Forced(v) => f.debug_tuple("Forced").field(v).finish(),
            LazyState::Native(_) => f.write_str("Native(<thunk>)"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Unit,
    Number(Rc<Rational>),
    String(Rc<str>),
    Bool(bool),
    // `:name` — Elixir-style atom. Equal iff names match; prints as `:name`.
    Atom(Rc<str>),
    List(Vector<Value>),
    Dict(Vector<(Value, Value)>),
    Set(Vector<Value>),
    // Lazy integer-step range. `end == None` is infinite (`[start..]`);
    // `inclusive` distinguishes `[a..b]` from `[a..=b]`. Step is always +1.
    Range {
        start: Rc<Rational>,
        end: Option<Rc<Rational>>,
        inclusive: bool,
    },
    // Lazy cons cell. Built by the `a :: b` binary expression: `head` is
    // forced (the lhs of `::`), `tail` defers `b` until something needs it
    // (pattern-match, display, equality). This is what lets recursive
    // stdlib builders like `f(x) :: map(f, rest)` terminate.
    Cons {
        head: Rc<Value>,
        tail: Rc<RefCell<LazyState>>,
    },
    Function {
        params: Vec<String>,
        body: Expr,
        env: Env,
    },
    Native {
        name: &'static str,
        arity: usize,
        applied: Vec<Value>,
        f: NativeFn,
    },
    Module {
        name: String,
        members: HashMap<String, Value>,
    },
}

pub type NativeFunction = Rc<dyn Fn(&Env, &[Value]) -> anyhow::Result<Value>>;

#[derive(Clone)]
pub struct NativeFn(pub NativeFunction);

impl std::fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<native fn>")
    }
}

pub fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Unit => "unit",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Bool(_) => "bool",
        Value::Atom(_) => "atom",
        Value::List(_) => "list",
        Value::Dict(_) => "dict",
        Value::Set(_) => "set",
        Value::Range { .. } => "range",
        Value::Cons { .. } => "cons",
        Value::Function { .. } => "function",
        Value::Native { .. } => "native",
        Value::Module { .. } => "module",
    }
}

fn value_eq(a: &Value, b: &Value) -> bool {
    // A Cons spine and a fully-evaluated container are equal iff they hold the
    // same sequence — flatten on either side so `1 :: [2, 3] == [1, 2, 3]` and
    // `map(f, [0..3]) == [f(0), f(1), f(2)]` work as expected.
    if matches!(a, Value::Cons { .. }) || matches!(b, Value::Cons { .. }) {
        return match (flatten_cons(a), flatten_cons(b)) {
            (Some(xs), Some(ys)) => {
                xs.len() == ys.len() && xs.iter().zip(ys.iter()).all(|(x, y)| value_eq(x, y))
            }
            _ => false,
        };
    }
    match (a, b) {
        (Value::Unit, Value::Unit) => true,
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Atom(x), Value::Atom(y)) => x == y,
        (Value::List(x), Value::List(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| value_eq(a, b))
        }
        (Value::Dict(x), Value::Dict(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.iter().any(|(k2, v2)| value_eq(k, k2) && value_eq(v, v2)))
        }
        (Value::Set(x), Value::Set(y)) => {
            x.len() == y.len() && x.iter().all(|a| y.iter().any(|b| value_eq(a, b)))
        }
        (
            Value::Range {
                start: s1,
                end: e1,
                inclusive: i1,
            },
            Value::Range {
                start: s2,
                end: e2,
                inclusive: i2,
            },
        ) => {
            s1 == s2
                && i1 == i2
                && match (e1, e2) {
                    (None, None) => true,
                    (Some(a), Some(b)) => a == b,
                    _ => false,
                }
        }
        _ => false,
    }
}

impl core::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        value_eq(self, other)
    }
}

/// Force a Cons spine into a flat Vec of its elements. Returns None if a thunk
/// fails to evaluate, or if the terminator type has no natural sequence
/// (number, bool, function, etc.). Strings flatten to one-char string values,
/// dicts to `[k, v]` pairs, ranges to numbers, so cross-type equality like
/// `take(3, "hel") == "hel"` and `cons-built-dict == literal-dict` works.
fn flatten_cons(v: &Value) -> Option<Vec<Value>> {
    let (mut items, mut cur_tail) = match v {
        Value::Cons { head, tail } => (vec![(**head).clone()], tail.clone()),
        Value::List(xs) | Value::Set(xs) => {
            return Some(xs.iter().cloned().collect());
        }
        Value::String(s) => return Some(string_chars(s)),
        Value::Dict(es) => return Some(es.iter().map(|(k, v)| pair(k, v)).collect()),
        Value::Range {
            start,
            end,
            inclusive,
        } => {
            let e = end.as_deref()?;
            return Some(range_elems(start, e, *inclusive));
        }
        _ => return None,
    };
    loop {
        let forced = super::expr::force_tail(&cur_tail).ok()?;
        match forced {
            Value::Cons { head, tail } => {
                items.push((*head).clone());
                cur_tail = tail;
            }
            Value::List(xs) | Value::Set(xs) => {
                items.extend(xs.iter().cloned());
                return Some(items);
            }
            Value::String(s) => {
                items.extend(string_chars(&s));
                return Some(items);
            }
            Value::Dict(es) => {
                items.extend(es.iter().map(|(k, v)| pair(k, v)));
                return Some(items);
            }
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                let end = end?; // refuse to flatten infinite tails
                items.extend(range_elems(&start, &end, inclusive));
                return Some(items);
            }
            Value::Unit => return Some(items),
            _ => return None,
        }
    }
}

fn string_chars(s: &str) -> Vec<Value> {
    s.chars()
        .map(|c| Value::String(c.to_string().into()))
        .collect()
}

fn pair(k: &Value, v: &Value) -> Value {
    Value::List(im::vector![k.clone(), v.clone()])
}

fn range_elems(start: &Rational, end: &Rational, inclusive: bool) -> Vec<Value> {
    let mut out = Vec::new();
    let mut cur: Rational = start.clone();
    while super::number::range_has_elem(&cur, Some(end), inclusive) {
        out.push(Value::Number(Rc::new(cur.clone())));
        cur += 1;
    }
    out
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Number(n) => write!(f, "{}", format_rational(n)),
            Value::String(s) => write!(f, "{:?}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Atom(name) => write!(f, ":{}", name),
            Value::List(xs) => {
                write!(f, "[")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", x)?;
                }
                write!(f, "]")
            }
            Value::Dict(es) => {
                write!(f, "{{")?;
                for (i, (k, v)) in es.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Set(xs) => {
                write!(f, "{{")?;
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", x)?;
                }
                write!(f, "}}")
            }
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                write!(f, "[{}..", format_rational(start))?;
                if *inclusive {
                    write!(f, "=")?;
                }
                if let Some(e) = end {
                    write!(f, "{}", format_rational(e))?;
                }
                write!(f, "]")
            }
            Value::Cons { head, tail } => fmt_cons(f, head, tail),
            Value::Function { params, .. } => write!(f, "<fn ({})>", params.join(", ")),
            Value::Native { name, .. } => write!(f, "<native {}>", name),
            Value::Module { name, .. } => write!(f, "<module {}>", name),
        }
    }
}

/// Walk a Cons spine, forcing thunks as we go, then print in the shape of the
/// terminator. A spine ending in a set prints with `{...}`, in a dict with
/// `{k: v, ...}`, in a string with `"..."`, etc. — this is what makes
/// `take(3, "hello")` display as `"hel"` and `take(2, {"a":1,"b":2})` as
/// `{"a": 1, "b": 2}`. Infinite ranges show their open-end marker without
/// forcing further; forcing errors surface as `<error: ...>`.
fn fmt_cons(
    f: &mut std::fmt::Formatter<'_>,
    head: &Rc<Value>,
    tail: &Rc<RefCell<LazyState>>,
) -> std::fmt::Result {
    let mut items: Vec<Value> = vec![(**head).clone()];
    let mut cur_tail = tail.clone();
    let terminator = loop {
        let forced = match super::expr::force_tail(&cur_tail) {
            Ok(v) => v,
            Err(e) => return write!(f, "<error: {}>", e),
        };
        match forced {
            Value::Cons { head, tail } => {
                items.push((*head).clone());
                cur_tail = tail;
            }
            other => break other,
        }
    };

    match &terminator {
        Value::Set(xs) => {
            write!(f, "{{")?;
            let mut first = true;
            for x in items.iter().chain(xs.iter()) {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}", x)?;
                first = false;
            }
            write!(f, "}}")
        }
        Value::Dict(es) => {
            write!(f, "{{")?;
            let mut first = true;
            for x in &items {
                let Some((k, v)) = pair_of(x) else {
                    return write!(f, "<bad dict cons cell: {}>", x);
                };
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}: {}", k, v)?;
                first = false;
            }
            for (k, v) in es {
                if !first {
                    write!(f, ", ")?;
                }
                write!(f, "{}: {}", k, v)?;
                first = false;
            }
            write!(f, "}}")
        }
        Value::String(s) => {
            // String cons builds chars/substrings: write the concatenation
            // back out as a single string literal.
            let mut buf = String::new();
            for x in &items {
                let Value::String(t) = x else {
                    // Non-string head with a string terminator: fall back to
                    // the list-style display rather than producing nonsense.
                    return fmt_cons_listish(f, &items, &terminator);
                };
                buf.push_str(t);
            }
            buf.push_str(s);
            write!(f, "{:?}", buf)
        }
        _ => fmt_cons_listish(f, &items, &terminator),
    }
}

fn pair_of(v: &Value) -> Option<(&Value, &Value)> {
    let Value::List(xs) = v else {
        return None;
    };
    if xs.len() != 2 {
        return None;
    }
    Some((xs.get(0).unwrap(), xs.get(1).unwrap()))
}

fn fmt_cons_listish(
    f: &mut std::fmt::Formatter<'_>,
    items: &[Value],
    terminator: &Value,
) -> std::fmt::Result {
    write!(f, "[")?;
    let mut wrote_any = false;
    for x in items {
        if wrote_any {
            write!(f, ", ")?;
        }
        write!(f, "{}", x)?;
        wrote_any = true;
    }
    match terminator {
        Value::List(xs) => {
            for x in xs {
                if wrote_any {
                    write!(f, ", ")?;
                }
                write!(f, "{}", x)?;
                wrote_any = true;
            }
        }
        Value::Range {
            start,
            end,
            inclusive,
        } => match end {
            None => {
                if wrote_any {
                    write!(f, ", ")?;
                }
                write!(f, "{}..", format_rational(start))?;
            }
            Some(e) => {
                let mut cur: Rational = (**start).clone();
                while super::number::range_has_elem(&cur, Some(e), *inclusive) {
                    if wrote_any {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", format_rational(&cur))?;
                    wrote_any = true;
                    cur += 1;
                }
            }
        },
        Value::Unit => {}
        other => {
            if wrote_any {
                write!(f, ", ")?;
            }
            write!(f, "{}", other)?;
        }
    }
    write!(f, "]")
}
