use crate::interpreter::{Scope, Value, apply, ctx_of, eval_program, local_vars, type_name};
use crate::native;
use crate::parser::parse;
use crate::prelude::native_module;
use anyhow::bail;
use std::path::PathBuf;

pub fn members() -> Vec<(&'static str, Value)> {
    vec![("|>", pipe()), ("import", import())]
}

fn pipe() -> Value {
    native!("|>", 2, |env, args| {
        apply(env, args[1].clone(), vec![args[0].clone()])
    })
}

fn import() -> Value {
    native!("import", 1, |env, args| {
        let path_str = match &args[0] {
            Value::String(s) => s.clone(),
            v => bail!("import expects string, got {}", type_name(v)),
        };
        if let Some(m) = native_module(&path_str) {
            return Ok(m);
        }
        let ctx = ctx_of(env);
        let resolved = match ctx.current_file.borrow().as_ref() {
            Some(p) => p
                .parent()
                .map(|d| d.join(&path_str))
                .unwrap_or_else(|| PathBuf::from(&path_str)),
            None => PathBuf::from(&path_str),
        };
        let source = std::fs::read_to_string(&resolved)
            .map_err(|e| anyhow::anyhow!("failed to read {}: {}", resolved.display(), e))?;
        let program = parse(&source)?;
        let name = resolved
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("module")
            .to_string();
        let module_env = Scope::child(env.clone());
        let prev = ctx.current_file.replace(Some(resolved));
        let result = eval_program(&program, &module_env);
        *ctx.current_file.borrow_mut() = prev;
        result?;
        Ok(Value::Module {
            name,
            members: local_vars(&module_env),
        })
    })
}
