use anyhow::{Result, bail};
use im::Vector;
use rug::Rational;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{Pattern, PatternItem};

use super::expr::{eval_expr, force_tail};
use super::number::{range_has_elem, rat_succ};
use super::scope::Env;
use super::value::Value;

pub(super) fn match_pattern(
    pat: &Pattern,
    val: &Value,
    env: &Env,
) -> Result<Option<HashMap<String, Value>>> {
    let mut bindings = HashMap::new();
    if match_into(pat, val, env, &mut bindings)? {
        Ok(Some(bindings))
    } else {
        Ok(None)
    }
}

fn match_into(
    pat: &Pattern,
    val: &Value,
    env: &Env,
    bindings: &mut HashMap<String, Value>,
) -> Result<bool> {
    match pat {
        Pattern::Wildcard => Ok(true),
        Pattern::Unit => Ok(matches!(val, Value::Unit)),
        Pattern::Ident(name) => {
            bindings.insert(name.clone(), val.clone());
            Ok(true)
        }
        Pattern::Number(n) => Ok(matches!(val, Value::Number(m) if **m == *n)),
        Pattern::String(s) => Ok(matches!(val, Value::String(t) if **t == **s)),
        Pattern::Bool(p) => Ok(matches!(val, Value::Bool(b) if b == p)),
        Pattern::Atom(name) => Ok(matches!(val, Value::Atom(n) if n.as_ref() == name.as_str())),
        Pattern::List(items) => match val {
            Value::Range {
                start,
                end,
                inclusive,
            } => match_seq_range(items, start, end.as_deref(), *inclusive, env, bindings),
            _ => match_seq(items, val, env, bindings),
        },
        Pattern::Dict(entries) => match val {
            Value::Dict(d) => {
                for (key_expr, sub_pat) in entries {
                    let key = eval_expr(key_expr, env)?;
                    let Some((_, found)) = d.iter().find(|(k, _)| k == &key) else {
                        return Ok(false);
                    };
                    if !match_into(sub_pat, found, env, bindings)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            // Dict patterns also destructure modules, keyed by member name:
            // `{Left, Right} = import "lib.ff"`.
            Value::Module { members, .. } => {
                for (key_expr, sub_pat) in entries {
                    let key = eval_expr(key_expr, env)?;
                    let Value::String(name) = key else {
                        return Ok(false);
                    };
                    let Some(found) = members.get(&*name).cloned() else {
                        return Ok(false);
                    };
                    if !match_into(sub_pat, &found, env, bindings)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        },
        Pattern::Set(items) => {
            let Value::Set(s) = val else {
                return Ok(false);
            };
            for p in items {
                let mut tmp = HashMap::new();
                let any = s
                    .iter()
                    .any(|el| match_into(p, el, env, &mut tmp).unwrap_or(false));
                if !any {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        // `head :: tail` is the inverse of the `::` operator: peel off the
        // first element (and rebuild the tail in the same shape) for any value
        // type the operator can construct.
        Pattern::Cons { head, tail } => match val {
            Value::List(xs) => match_cons_seq(head, tail, xs, env, bindings, Value::List),
            Value::Set(xs) => match_cons_seq(head, tail, xs, env, bindings, Value::Set),
            Value::String(s) => {
                let Some(first) = s.chars().next() else {
                    return Ok(false);
                };
                let h = Value::String(first.to_string().into());
                let t = Value::String(Rc::from(&s[first.len_utf8()..]));
                if !match_into(head, &h, env, bindings)? {
                    return Ok(false);
                }
                match_into(tail, &t, env, bindings)
            }
            Value::Dict(es) => {
                let Some((k, v)) = es.front() else {
                    return Ok(false);
                };
                let h = Value::List(im::vector![k.clone(), v.clone()]);
                if !match_into(head, &h, env, bindings)? {
                    return Ok(false);
                }
                let mut rest = es.clone();
                rest.pop_front();
                match_into(tail, &Value::Dict(rest), env, bindings)
            }
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                if !range_has_elem(start, end.as_deref(), *inclusive) {
                    return Ok(false);
                }
                let h = Value::Number(start.clone());
                if !match_into(head, &h, env, bindings)? {
                    return Ok(false);
                }
                let t = Value::Range {
                    start: Rc::new(rat_succ(start)),
                    end: end.clone(),
                    inclusive: *inclusive,
                };
                match_into(tail, &t, env, bindings)
            }
            // Force one step of a lazy cons cell. The forced tail becomes the
            // `rest` binding, so chained `a :: b :: rest` patterns peel
            // elements off the stream one thunk at a time. A wildcard tail
            // skips the force — otherwise `h :: _` would diverge on streams
            // whose tail thunk doesn't terminate.
            Value::Cons { head: h, tail: t } => {
                if !match_into(head, h, env, bindings)? {
                    return Ok(false);
                }
                if matches!(**tail, Pattern::Wildcard) {
                    return Ok(true);
                }
                let rest = force_tail(t)?;
                match_into(tail, &rest, env, bindings)
            }
            _ => Ok(false),
        },
    }
}

fn match_seq_range(
    items: &[PatternItem],
    start: &Rc<Rational>,
    end: Option<&Rational>,
    inclusive: bool,
    env: &Env,
    bindings: &mut HashMap<String, Value>,
) -> Result<bool> {
    let rest_idx = items.iter().position(|i| matches!(i, PatternItem::Rest(_)));
    let mut cur: Rational = (**start).clone();
    match rest_idx {
        None => {
            // Exact-length list pattern: peel one element per item, then the
            // range must be exhausted. Infinite ranges never match exact length.
            for p in items {
                let PatternItem::Pattern(p) = p else {
                    unreachable!()
                };
                if !range_has_elem(&cur, end, inclusive) {
                    return Ok(false);
                }
                let elem = Value::Number(Rc::new(cur.clone()));
                if !match_into(p, &elem, env, bindings)? {
                    return Ok(false);
                }
                cur += 1;
            }
            if range_has_elem(&cur, end, inclusive) {
                return Ok(false);
            }
            Ok(true)
        }
        Some(idx) => {
            let before = &items[..idx];
            let after = &items[idx + 1..];
            if after.iter().any(|i| matches!(i, PatternItem::Rest(_))) {
                bail!("multiple `..` patterns in one sequence");
            }
            // `[a, .., b]` against a range would need to seek from the end —
            // only finite ranges have one, and even then the user can convert
            // to a list explicitly. Refuse for now.
            if !after.is_empty() {
                return Ok(false);
            }
            for p in before {
                let PatternItem::Pattern(p) = p else {
                    unreachable!()
                };
                if !range_has_elem(&cur, end, inclusive) {
                    return Ok(false);
                }
                let elem = Value::Number(Rc::new(cur.clone()));
                if !match_into(p, &elem, env, bindings)? {
                    return Ok(false);
                }
                cur += 1;
            }
            if let PatternItem::Rest(Some(name)) = &items[idx] {
                bindings.insert(
                    name.clone(),
                    Value::Range {
                        start: Rc::new(cur),
                        end: end.map(|e| Rc::new(e.clone())),
                        inclusive,
                    },
                );
            }
            Ok(true)
        }
    }
}

fn match_cons_seq(
    head: &Pattern,
    tail: &Pattern,
    xs: &Vector<Value>,
    env: &Env,
    bindings: &mut HashMap<String, Value>,
    rewrap: impl FnOnce(Vector<Value>) -> Value,
) -> Result<bool> {
    let Some(h) = xs.front() else {
        return Ok(false);
    };
    if !match_into(head, h, env, bindings)? {
        return Ok(false);
    }
    let mut rest = xs.clone();
    rest.pop_front();
    match_into(tail, &rewrap(rest), env, bindings)
}

fn match_seq(
    items: &[PatternItem],
    val: &Value,
    env: &Env,
    bindings: &mut HashMap<String, Value>,
) -> Result<bool> {
    let Value::List(elems) = val else {
        return Ok(false);
    };
    let rest_idx = items.iter().position(|i| matches!(i, PatternItem::Rest(_)));
    match rest_idx {
        None => {
            if elems.len() != items.len() {
                return Ok(false);
            }
            for (p, v) in items.iter().zip(elems.iter()) {
                let PatternItem::Pattern(p) = p else {
                    unreachable!()
                };
                if !match_into(p, v, env, bindings)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Some(idx) => {
            if items[idx + 1..]
                .iter()
                .any(|i| matches!(i, PatternItem::Rest(_)))
            {
                bail!("multiple `..` patterns in one sequence");
            }
            let before = &items[..idx];
            let after = &items[idx + 1..];
            if elems.len() < before.len() + after.len() {
                return Ok(false);
            }
            for (p, v) in before.iter().zip(elems.iter()) {
                let PatternItem::Pattern(p) = p else {
                    unreachable!()
                };
                if !match_into(p, v, env, bindings)? {
                    return Ok(false);
                }
            }
            let after_start = elems.len() - after.len();
            for (p, v) in after.iter().zip(elems.iter().skip(after_start)) {
                let PatternItem::Pattern(p) = p else {
                    unreachable!()
                };
                if !match_into(p, v, env, bindings)? {
                    return Ok(false);
                }
            }
            if let PatternItem::Rest(Some(name)) = &items[idx] {
                let middle = elems.clone().slice(before.len()..after_start);
                bindings.insert(name.clone(), Value::List(middle));
            }
            Ok(true)
        }
    }
}
