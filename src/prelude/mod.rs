use crate::interpreter::{Env, Scope, Value, ctx_of, define, eval_program};
use crate::parser::parse;
use anyhow::{Result, anyhow};
use im::vector;
use std::collections::HashMap;
use std::path::PathBuf;

mod fs;
mod global;

pub fn install(env: &Env) {
    for (name, f) in global::members() {
        define(env, name, f);
    }

    let program = parse(include_str!("stdlib.ff")).expect("failed to parse stdlib.ff");
    let ctx = ctx_of(env);
    let module_env = Scope::child(env.clone());
    let prev_exports = ctx.current_exports.replace(Some(HashMap::new()));
    let result = eval_program(&program, &module_env);
    let exports = ctx.current_exports.replace(prev_exports);
    result.expect("failed to evaluate stdlib.ff");
    for (name, value) in exports.unwrap_or_default() {
        define(env, &name, value);
    }
}

pub(crate) fn ok(v: Value) -> Value {
    Value::List(vector![Value::Atom("ok".into()), v])
}

pub(crate) fn err(msg: impl Into<String>) -> Value {
    Value::List(vector![
        Value::Atom("error".into()),
        Value::String(msg.into().into())
    ])
}

pub(crate) fn dict(entries: Vec<(&str, Value)>) -> Value {
    let entries = entries
        .into_iter()
        .map(|(name, value)| (Value::String(name.into()), value))
        .collect();
    Value::Dict(entries)
}

pub(crate) fn native_module(name: &str) -> Option<Value> {
    let members = match name {
        "fs" => fs::members(),
        _ => return None,
    };
    Some(Value::Module {
        name: name.to_string(),
        members: members
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<HashMap<_, _>>(),
    })
}

/// Resolve and load a module by path. Built-in modules (`io`, etc.) are checked
/// first; otherwise the path is resolved relative to the current file and the
/// source is parsed and evaluated in a fresh child scope. Only names registered
/// via `export` end up in the returned module's members.
pub fn import_module(env: &Env, path_str: &str) -> Result<Value> {
    if let Some(m) = native_module(path_str) {
        return Ok(m);
    }
    let ctx = ctx_of(env);
    let resolved = match ctx.current_file.borrow().as_ref() {
        Some(p) => p
            .parent()
            .map(|d| d.join(path_str))
            .unwrap_or_else(|| PathBuf::from(path_str)),
        None => PathBuf::from(path_str),
    };
    let source = std::fs::read_to_string(&resolved)
        .map_err(|e| anyhow!("failed to read {}: {}", resolved.display(), e))?;
    let program = parse(&source)?;
    let name = resolved
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("module")
        .to_string();
    let module_env = Scope::child(env.clone());
    let prev_file = ctx.current_file.replace(Some(resolved));
    let prev_exports = ctx.current_exports.replace(Some(HashMap::new()));
    let result = eval_program(&program, &module_env);
    let exports = ctx.current_exports.replace(prev_exports);
    *ctx.current_file.borrow_mut() = prev_file;
    result?;
    Ok(Value::Module {
        name,
        members: exports.unwrap_or_default(),
    })
}

// Build a `Value::Native` with the given name, arity, and body. The body is
// `Fn(&Ctx, &[Value]) -> Result<Value>`; arity-checking and partial application
// are handled by the interpreter's Call dispatch.
#[macro_export]
macro_rules! native {
    ($name:expr, $arity:expr, $body:expr) => {
        $crate::interpreter::Value::Native {
            name: $name,
            arity: $arity,
            applied: Vec::new(),
            f: $crate::interpreter::NativeFn(std::rc::Rc::new($body)),
        }
    };
}
