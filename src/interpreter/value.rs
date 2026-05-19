use im::{HashMap, HashSet, Vector};
use rug::Rational;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::ast::{Expr, Pattern};

use super::error::RuntimeResult;
use super::number::format_rational;
use super::scope::Env;

pub enum LazyState {
    Pending { body: Expr, env: Env },
    Forced(Value),
    // A native-built thunk. Used by stdlib streams (e.g. `file.lines`) that
    // can't be expressed as an AST expression because they carry Rust state
    // like an open `BufReader`.
    Native(Box<dyn FnOnce() -> RuntimeResult<Value>>),
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
    Object(HashMap<Value, Value>),
    Set(HashSet<Value>),
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
        params: Vec<Pattern>,
        body: Expr,
        env: Env,
    },
    Native {
        name: &'static str,
        arity: usize,
        applied: Vec<Value>,
        f: NativeFn,
    },
}

pub type NativeFunction = Rc<dyn Fn(&Env, &[Value]) -> RuntimeResult<Value>>;

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
        Value::Object(_) => "object",
        Value::Set(_) => "set",
        Value::Range { .. } => "range",
        Value::Cons { .. } => "cons",
        Value::Function { .. } => "function",
        Value::Native { .. } => "native",
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
        (Value::Object(x), Value::Object(y)) => x == y,
        (Value::Set(x), Value::Set(y)) => x == y,
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

impl core::cmp::Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Unit => {}
            Value::Number(n) => n.to_string().hash(state),
            Value::String(s) => s.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Atom(name) => name.hash(state),
            Value::List(xs) => {
                xs.len().hash(state);
                for x in xs {
                    x.hash(state);
                }
            }
            // Set/Object iteration order isn't fixed by im, so XOR per-element
            // hashes to get an order-independent (still equality-compatible)
            // hash.
            Value::Set(xs) => {
                let mut h: u64 = 0;
                for x in xs {
                    let mut hasher = DefaultHasher::new();
                    x.hash(&mut hasher);
                    h ^= hasher.finish();
                }
                state.write_u64(h);
            }
            Value::Object(es) => {
                let mut h: u64 = 0;
                for (k, v) in es {
                    let mut hasher = DefaultHasher::new();
                    k.hash(&mut hasher);
                    v.hash(&mut hasher);
                    h ^= hasher.finish();
                }
                state.write_u64(h);
            }
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                start.to_string().hash(state);
                match end {
                    None => 0u8.hash(state),
                    Some(e) => {
                        1u8.hash(state);
                        e.to_string().hash(state);
                    }
                }
                inclusive.hash(state);
            }
            // Tail is lazy — only fold the eager head in. Cons cells don't
            // typically live inside hashed collections; if they do, two cells
            // with the same head will collide but PartialEq will still keep
            // them distinct.
            Value::Cons { head, .. } => head.hash(state),
            // Functions/Natives aren't reflexive under PartialEq (function ==
            // function is always false). Hashing by name/discriminant alone is
            // a best-effort placeholder; using them as keys is unsupported.
            Value::Function { .. } => {}
            Value::Native { name, .. } => name.hash(state),
        }
    }
}

/// Force a Cons spine into a flat Vec of its elements. Returns None if a thunk
/// fails to evaluate, or if the terminator type has no natural sequence
/// (number, bool, function, etc.). Strings flatten to one-char string values,
/// maps to `[k, v]` pairs, ranges to numbers, so cross-type equality like
/// `take(3, "hel") == "hel"` and `cons-built-object == literal-object` works.
fn flatten_cons(v: &Value) -> Option<Vec<Value>> {
    let (mut items, mut cur_tail) = match v {
        Value::Cons { head, tail } => (vec![(**head).clone()], tail.clone()),
        Value::List(xs) => return Some(xs.iter().cloned().collect()),
        Value::Set(xs) => return Some(xs.iter().cloned().collect()),
        Value::String(s) => return Some(string_chars(s)),
        Value::Object(es) => return Some(es.iter().map(|(k, v)| pair(k, v)).collect()),
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
            Value::List(xs) => {
                items.extend(xs.iter().cloned());
                return Some(items);
            }
            Value::Set(xs) => {
                items.extend(xs.iter().cloned());
                return Some(items);
            }
            Value::String(s) => {
                items.extend(string_chars(&s));
                return Some(items);
            }
            Value::Object(es) => {
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
            Value::Object(es) => {
                let mut entries: Vec<(String, String)> = es
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                write!(f, "{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Set(xs) => {
                let mut items: Vec<String> = xs.iter().map(|x| x.to_string()).collect();
                items.sort();
                write!(f, "{{")?;
                for (i, x) in items.iter().enumerate() {
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
            Value::Function { params, .. } => write!(
                f,
                "<fn ({})>",
                params
                    .iter()
                    .map(format_pattern)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Native { name, .. } => write!(f, "<native {}>", name),
        }
    }
}

/// Render a Pattern roughly back into its source form. Used by the
/// `<fn (...)>` display so destructuring params show their shape.
pub fn format_pattern(p: &Pattern) -> String {
    match p {
        Pattern::Wildcard => "_".to_string(),
        Pattern::Unit => "()".to_string(),
        Pattern::Ident(name) => name.clone(),
        Pattern::Number(n) => format_rational(n),
        Pattern::String(s) => format!("{:?}", s),
        Pattern::Bool(b) => b.to_string(),
        Pattern::Atom(name) => format!(":{}", name),
        Pattern::List(items) => {
            let parts: Vec<String> = items
                .iter()
                .map(|i| match i {
                    crate::ast::PatternItem::Pattern(p) => format_pattern(p),
                    crate::ast::PatternItem::Rest(None) => "..".to_string(),
                    crate::ast::PatternItem::Rest(Some(n)) => format!("..{}", n),
                })
                .collect();
            format!("[{}]", parts.join(", "))
        }
        Pattern::Object(entries) => {
            let parts: Vec<String> = entries
                .iter()
                .map(|(_, v)| format_pattern(v))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        Pattern::Set(items) => {
            let parts: Vec<String> = items.iter().map(format_pattern).collect();
            format!("{{{}}}", parts.join(", "))
        }
        Pattern::Cons { head, tail } => {
            format!("{} :: {}", format_pattern(head), format_pattern(tail))
        }
    }
}

/// Walk a Cons spine, forcing thunks as we go, then print in the shape of the
/// terminator. A spine ending in a set prints with `{...}`, in a object with
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
            let mut strs: Vec<String> = items
                .iter()
                .chain(xs.iter())
                .map(|x| x.to_string())
                .collect();
            strs.sort();
            write!(f, "{{")?;
            for (i, s) in strs.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", s)?;
            }
            write!(f, "}}")
        }
        Value::Object(es) => {
            let mut entries: Vec<(String, String)> = Vec::new();
            for x in &items {
                let Some((k, v)) = pair_of(x) else {
                    return write!(f, "<bad object cons cell: {}>", x);
                };
                entries.push((k.to_string(), v.to_string()));
            }
            for (k, v) in es {
                entries.push((k.to_string(), v.to_string()));
            }
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            write!(f, "{{")?;
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}: {}", k, v)?;
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
