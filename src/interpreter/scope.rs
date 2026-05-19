use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

use super::runtime::Runtime;
use super::value::Value;

pub struct Ctx {
    pub out: RefCell<Box<dyn Write>>,
    pub current_file: RefCell<Option<PathBuf>>,
    pub current_exports: RefCell<Option<Vec<(String, Value)>>>,
    pub is_interactive: bool,
    pub runtime: RefCell<Runtime>,
}

impl Ctx {
    pub fn stdio() -> Rc<Self> {
        Rc::new(Ctx {
            out: RefCell::new(Box::new(std::io::stdout())),
            current_file: RefCell::new(None),
            current_exports: RefCell::new(None),
            is_interactive: true,
            runtime: RefCell::new(Runtime::new()),
        })
    }

    pub fn stdio_with_file(path: PathBuf) -> Rc<Self> {
        Rc::new(Ctx {
            out: RefCell::new(Box::new(std::io::stdout())),
            current_file: RefCell::new(Some(path)),
            current_exports: RefCell::new(None),
            is_interactive: false,
            runtime: RefCell::new(Runtime::new()),
        })
    }
}

impl std::fmt::Debug for Ctx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ctx")
    }
}

pub type Env = Rc<RefCell<Scope>>;

#[derive(Debug)]
pub struct Scope {
    pub(super) vars: HashMap<String, Value>,
    parent: Option<Env>,
    ctx: Rc<Ctx>,
}

impl Scope {
    pub fn new() -> Env {
        Self::with_ctx(Ctx::stdio())
    }

    pub fn with_ctx(ctx: Rc<Ctx>) -> Env {
        Rc::new(RefCell::new(Scope {
            vars: HashMap::new(),
            parent: None,
            ctx,
        }))
    }

    pub fn child(parent: Env) -> Env {
        let ctx = parent.borrow().ctx.clone();
        Rc::new(RefCell::new(Scope {
            vars: HashMap::new(),
            parent: Some(parent),
            ctx,
        }))
    }
}

pub fn ctx_of(env: &Env) -> Rc<Ctx> {
    env.borrow().ctx.clone()
}

pub(super) fn lookup(env: &Env, name: &str) -> Option<Value> {
    if let Some(v) = env.borrow().vars.get(name).cloned() {
        return Some(v);
    }
    let parent = env.borrow().parent.clone();
    parent.and_then(|p| lookup(&p, name))
}

pub fn define(env: &Env, name: &str, val: Value) {
    env.borrow_mut().vars.insert(name.to_string(), val);
}
