use std::path::Path;

use crate::interpreter::{self, Value};
use crate::parser::parse;

pub fn eval_file(path: impl AsRef<Path>) -> anyhow::Result<Value> {
    let path = path.as_ref();
    let source = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read {}: {}", path.display(), e))?;
    eval_source(&source)
}

pub fn eval_source(source: &str) -> anyhow::Result<Value> {
    let program = parse(source)?;
    interpreter::run(&program)
}
