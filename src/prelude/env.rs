use crate::members;
use std::collections::HashMap;

use crate::{interop::native_fn, interpreter::Value};

members! {
    "Env",
    args => || -> Vec<String> { std::env::args().collect() },
    vars => || std::env::vars().collect::<HashMap<String, String>>(),
    // Accept either a string key (`"PATH"`) or an atom (`:PATH`, which arrives
    // as the serialized form `":PATH"`); strip the leading `:` to recover the
    // env-var name in both cases.
    var => |key: String| {
        let name = key.strip_prefix(':').unwrap_or(&key);
        match std::env::var(name) {
            Ok(value) => (":ok", value),
            Err(e) => (":error", e.to_string()),
        }
    },
}
