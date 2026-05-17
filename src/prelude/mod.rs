use crate::interpreter::{
    Env, Scope, Value, apply, ctx_of, define, eval_program, local_vars, type_name,
};
use crate::native;
use crate::parser::parse;
use anyhow::bail;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod io;

pub fn install(env: &Env) {
    define_module(env, "io", io::members());

    // `x |> f` desugars (via custom_op_or in the grammar) to `((|>)(x))(f)`,
    // so this is an arity-2 native; the interpreter handles the partial
    // application after the first argument.
    define(
        env,
        "|>",
        native!("|>", 2, |env, args| {
            apply(env, args[1].clone(), vec![args[0].clone()])
        }),
    );

    define(
        env,
        "import",
        native!("import", 1, |env, args| {
            let path_str = match &args[0] {
                Value::String(s) => s.clone(),
                v => bail!("import expects string, got {}", type_name(v)),
            };
            let ctx = ctx_of(env);
            let resolved = match ctx.current_file.borrow().as_ref() {
                Some(p) => p
                    .parent()
                    .map(|d| d.join(&path_str))
                    .unwrap_or_else(|| PathBuf::from(&path_str)),
                None => PathBuf::from(&path_str),
            };
            let source = std::fs::read_to_string(&resolved).map_err(|e| {
                anyhow::anyhow!("failed to read {}: {}", resolved.display(), e)
            })?;
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
        }),
    );
}

fn define_module(env: &Env, name: &'static str, members: Vec<(&'static str, Value)>) {
    let members = members
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect::<HashMap<_, _>>();
    define(
        env,
        name,
        Value::Module {
            name: name.to_string(),
            members,
        },
    );
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
