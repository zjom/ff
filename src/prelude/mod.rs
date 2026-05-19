use crate::interpreter::{
    Env, RuntimeError, RuntimeResult, Scope, Value, ctx_of, define, eval_program,
};
use crate::parser::parse;
use im::{HashMap, vector};
use std::path::PathBuf;

mod fs;
mod global;
mod object;

pub fn install(env: &Env) {
    for (name, f) in global::members() {
        define(env, name, f);
    }

    let program = parse(include_str!("stdlib.ff")).expect("failed to parse stdlib.ff");
    let ctx = ctx_of(env);
    let module_env = Scope::child(env.clone());
    let prev_exports = ctx.current_exports.replace(Some(Vec::new()));
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

pub(crate) fn object(entries: Vec<(&str, Value)>) -> Value {
    let entries = entries
        .into_iter()
        .map(|(name, value)| (Value::String(name.into()), value))
        .collect();
    Value::Object(entries)
}

pub(crate) fn native_module(name: &str) -> Option<Value> {
    let members = match name {
        "Fs" => fs::members(),
        "Object" => object::members(),
        _ => return None,
    };
    Some(object_with_atom_keys(
        members.into_iter().map(|(k, v)| (k.to_string(), v)),
    ))
}

fn object_with_atom_keys(entries: impl IntoIterator<Item = (String, Value)>) -> Value {
    let mut out: HashMap<Value, Value> = HashMap::new();
    for (k, v) in entries {
        out.insert(Value::Atom(k.into()), v);
    }
    Value::Object(out)
}

/// Resolve and load a module by path. Built-in modules (`io`, etc.) are checked
/// first; otherwise the path is resolved relative to the current file and the
/// source is parsed and evaluated in a fresh child scope. Only names registered
/// via `export` end up in the returned module's members.
pub fn import_module(env: &Env, path_str: &str) -> RuntimeResult<Value> {
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
    let source = std::fs::read_to_string(&resolved).map_err(|e| RuntimeError::ReadFile {
        path: resolved.display().to_string(),
        source: e,
    })?;
    let program = parse(&source).map_err(|e| RuntimeError::Parse(e.to_string()))?;
    let module_env = Scope::child(env.clone());
    let prev_file = ctx.current_file.replace(Some(resolved));
    let prev_exports = ctx.current_exports.replace(Some(Vec::new()));
    let result = eval_program(&program, &module_env);
    let exports = ctx.current_exports.replace(prev_exports);
    *ctx.current_file.borrow_mut() = prev_file;
    result?;
    Ok(object_with_atom_keys(exports.unwrap_or_default()))
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
