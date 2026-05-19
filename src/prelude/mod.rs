use crate::interpreter::{
    Env, RuntimeError, RuntimeResult, Scope, Value, ctx_of, define, eval_program,
};
use crate::parser::parse;
use std::path::PathBuf;

mod actor;
mod env;
mod fs;
mod global;
mod http;
mod io;
mod json;
mod list;
mod object;
mod string;
mod utils;
use utils::*;

pub fn install(env: &Env) {
    for (name, f) in global::members() {
        define(env, name, f);
    }

    let program = parse(include_str!("stdlib.ff")).expect("failed to parse stdlib.ff");
    let ctx = ctx_of(env);
    let module_env = Scope::child(env.clone());
    let prev_exports = (*ctx.current_exports.lock().unwrap()).replace(Vec::new());
    let result = eval_program(&program, &module_env);
    let exports = std::mem::replace(&mut *ctx.current_exports.lock().unwrap(), prev_exports);
    result.expect("failed to evaluate stdlib.ff");
    for (name, value) in exports.unwrap_or_default() {
        define(env, &name, value);
    }
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
    let resolved = match ctx.current_file.lock().unwrap().as_ref() {
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
    let prev_file = (*ctx.current_file.lock().unwrap()).replace(resolved);
    let prev_exports = (*ctx.current_exports.lock().unwrap()).replace(Vec::new());
    let result = eval_program(&program, &module_env);
    let exports = std::mem::replace(&mut *ctx.current_exports.lock().unwrap(), prev_exports);
    *ctx.current_file.lock().unwrap() = prev_file;
    result?;
    Ok(object(exports.unwrap_or_default()))
}

fn native_module(name: &str) -> Option<Value> {
    let members = match name {
        "Fs" => fs::members(),
        "Object" => object::members(),
        "Env" => env::members(),
        "Actor" => actor::members(),
        "Io" => io::members(),
        "Http" => http::members(),
        "Json" => json::members(),
        "String" => string::members(),
        "List" => list::members(),
        _ => return None,
    };
    Some(object(members.into_iter().map(|(k, v)| (k.to_string(), v))))
}
