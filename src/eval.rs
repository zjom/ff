use std::path::Path;

use crate::interpreter::{self, Ctx, Scope, Value, eval_program};
use crate::parser::parse;

pub fn eval_file(path: impl AsRef<Path>) -> anyhow::Result<Value> {
    let path = path.as_ref();
    let source = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {}", path.display(), e))?;
    let program = parse(&source)?;
    let env = Scope::with_ctx(Ctx::stdio_with_file(path.to_path_buf()));
    crate::prelude::install(&env);
    eval_program(&program, &env)
}

pub fn eval_source(source: &str) -> anyhow::Result<Value> {
    let program = parse(source)?;
    interpreter::run(&program)
}
