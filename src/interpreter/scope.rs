use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::runtime::Runtime;
use super::value::Value;

pub struct Ctx {
    // `out` and the other mutable fields are wrapped in `Mutex` rather than
    // `RefCell` so a `Ctx` can be shared across the threads spawned by the
    // actor runtime. Locks are always short-lived (a single write/read/swap)
    // so contention is negligible in practice.
    pub out: Mutex<Box<dyn Write + Send>>,
    pub current_file: Mutex<Option<PathBuf>>,
    pub current_exports: Mutex<Option<Vec<(String, Value)>>>,
    pub is_interactive: bool,
    pub runtime: Mutex<Runtime>,
}

impl Ctx {
    pub fn stdio() -> Arc<Self> {
        Arc::new(Ctx {
            out: Mutex::new(Box::new(std::io::stdout())),
            current_file: Mutex::new(None),
            current_exports: Mutex::new(None),
            is_interactive: true,
            runtime: Mutex::new(Runtime::new()),
        })
    }

    pub fn stdio_with_file(path: PathBuf) -> Arc<Self> {
        Arc::new(Ctx {
            out: Mutex::new(Box::new(std::io::stdout())),
            current_file: Mutex::new(Some(path)),
            current_exports: Mutex::new(None),
            is_interactive: false,
            runtime: Mutex::new(Runtime::new()),
        })
    }
}

impl std::fmt::Debug for Ctx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ctx")
    }
}

pub type Env = Arc<Mutex<Scope>>;

#[derive(Debug)]
pub struct Scope {
    pub(super) vars: HashMap<String, Value>,
    parent: Option<Env>,
    ctx: Arc<Ctx>,
}

impl Scope {
    pub fn new() -> Env {
        Self::with_ctx(Ctx::stdio())
    }

    pub fn with_ctx(ctx: Arc<Ctx>) -> Env {
        Arc::new(Mutex::new(Scope {
            vars: HashMap::new(),
            parent: None,
            ctx,
        }))
    }

    pub fn child(parent: Env) -> Env {
        let ctx = parent.lock().unwrap().ctx.clone();
        Arc::new(Mutex::new(Scope {
            vars: HashMap::new(),
            parent: Some(parent),
            ctx,
        }))
    }
}

pub fn ctx_of(env: &Env) -> Arc<Ctx> {
    env.lock().unwrap().ctx.clone()
}

pub(super) fn lookup(env: &Env, name: &str) -> Option<Value> {
    if let Some(v) = env.lock().unwrap().vars.get(name).cloned() {
        return Some(v);
    }
    let parent = env.lock().unwrap().parent.clone();
    parent.and_then(|p| lookup(&p, name))
}

pub fn define(env: &Env, name: &str, val: Value) {
    env.lock().unwrap().vars.insert(name.to_string(), val);
}
