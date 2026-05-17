use crate::interpreter::{Env, Value, apply, define};
use crate::native;
use std::collections::HashMap;

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
}

fn define_module(env: &Env, name: &'static str, members: Vec<(&'static str, Value)>) {
    let members = members
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect::<HashMap<_, _>>();
    define(env, name, Value::Module { name, members });
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
