//! Erlang/Elixir-style actor runtime.
//!
//! An *actor* is an ff `Object` whose atom-keyed fields are callbacks:
//! `:init`, `:handle_call`, `:handle_cast`. The runtime spawns one per
//! `Actor.spawn` call, hands it a mailbox, and drains messages by invoking
//! the relevant callback via [`apply`].
//!
//! The scheduler is **cooperative and single-threaded** — there is no
//! parallelism. "Concurrent" here means many actors interleaved on one OS
//! thread: each actor processes its own mailbox strictly in order, but the
//! global order between actors is determined by which `cast`/`call` site
//! drives the scheduler. This matches BEAM's per-actor semantics (no mid-
//! handler preemption) while sacrificing BEAM's parallel scheduling.
//!
//! Reentrancy: while an actor's handler is on the Rust stack its
//! `processing` flag is set; any attempt to `call` into a processing actor
//! returns `:deadlock` rather than blocking forever.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::rc::Rc;

use super::error::{RuntimeError, RuntimeResult};
use super::expr::apply;
use super::scope::{Env, ctx_of};
use super::value::{Value, type_name};

pub type Pid = u64;

pub enum ActorMsg {
    Cast(Value),
    Call {
        msg: Value,
        reply: Rc<RefCell<Option<CallReply>>>,
    },
    Stop,
}

#[derive(Clone)]
pub enum CallReply {
    Ok(Value),
    Err(Rc<str>),
}

pub struct Actor {
    pub pid: Pid,
    pub template: Value,
    pub state: Value,
    pub mailbox: VecDeque<ActorMsg>,
    pub alive: bool,
    // True while a handler frame for this actor is on the Rust stack. Used
    // both to gate re-enqueue (an actor handling a message must not be in
    // `ready` simultaneously) and to detect synchronous `call`-cycles.
    pub processing: bool,
}

pub struct Runtime {
    next_pid: Pid,
    actors: HashMap<Pid, Rc<RefCell<Actor>>>,
    ready: VecDeque<Pid>,
    current: Option<Pid>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            next_pid: 1,
            actors: HashMap::new(),
            ready: VecDeque::new(),
            current: None,
        }
    }

    fn mint_pid(&mut self) -> Pid {
        let p = self.next_pid;
        self.next_pid += 1;
        p
    }

    fn get(&self, pid: Pid) -> Option<Rc<RefCell<Actor>>> {
        self.actors.get(&pid).cloned()
    }
}

fn lookup_cb(template: &Value, name: &str) -> Option<Value> {
    let Value::Object(es) = template else {
        return None;
    };
    es.get(&Value::Atom(name.into())).cloned()
}

/// Spawn a new actor from an Object template. Runs `:init` (if present) on
/// the spawner's stack so init errors propagate as a spawn failure rather
/// than crashing the new process silently.
pub fn spawn(env: &Env, template: Value) -> RuntimeResult<Value> {
    if !matches!(template, Value::Object(_)) {
        return Err(RuntimeError::NativeTypeError {
            native: "Actor.spawn",
            expected: "object template",
            got: type_name(&template),
        });
    }

    let ctx = ctx_of(env);
    let pid = ctx.runtime.borrow_mut().mint_pid();

    let init_cb = lookup_cb(&template, "init");
    let state = if let Some(init_fn) = init_cb {
        let prev = {
            let mut rt = ctx.runtime.borrow_mut();
            let p = rt.current;
            rt.current = Some(pid);
            p
        };
        let result = apply(env, init_fn, vec![]);
        ctx.runtime.borrow_mut().current = prev;
        result?
    } else {
        Value::Unit
    };

    let actor = Actor {
        pid,
        template,
        state,
        mailbox: VecDeque::new(),
        alive: true,
        processing: false,
    };
    ctx.runtime
        .borrow_mut()
        .actors
        .insert(pid, Rc::new(RefCell::new(actor)));
    Ok(Value::Pid(pid))
}

/// Fire-and-forget message. Dead/unknown pids drop silently (Erlang semantics).
/// Drains the scheduler before returning so casts produce visible effects in
/// scripts that don't otherwise drive the runtime.
pub fn cast(env: &Env, pid: Pid, msg: Value) {
    let ctx = ctx_of(env);
    let actor_rc = ctx.runtime.borrow().get(pid);
    let Some(actor_rc) = actor_rc else { return };
    if !actor_rc.borrow().alive {
        return;
    }
    actor_rc.borrow_mut().mailbox.push_back(ActorMsg::Cast(msg));
    enqueue_if_idle(&ctx.runtime, &actor_rc, pid);
    drain_all(env);
}

/// Synchronous request. On a single thread we can't truly block; instead we
/// loop the scheduler until the reply slot fills or the queue empties.
/// Returns `:no_proc` for unknown/dead targets, `:deadlock`/`:self_deadlock`
/// when the target is already on the Rust call stack.
pub fn call(env: &Env, pid: Pid, msg: Value) -> CallReply {
    let ctx = ctx_of(env);
    let actor_rc = ctx.runtime.borrow().get(pid);
    let Some(actor_rc) = actor_rc else {
        return CallReply::Err(":no_proc".into());
    };
    if !actor_rc.borrow().alive {
        return CallReply::Err(":no_proc".into());
    }
    if actor_rc.borrow().processing {
        let kind = if ctx.runtime.borrow().current == Some(pid) {
            ":self_deadlock"
        } else {
            ":deadlock"
        };
        return CallReply::Err(kind.into());
    }

    let slot = Rc::new(RefCell::new(None::<CallReply>));
    actor_rc.borrow_mut().mailbox.push_back(ActorMsg::Call {
        msg,
        reply: slot.clone(),
    });
    enqueue_if_idle(&ctx.runtime, &actor_rc, pid);
    drain_until(env, &slot);
    let reply = slot.borrow().clone();
    reply.unwrap_or(CallReply::Err(":no_proc".into()))
}

pub fn alive(env: &Env, pid: Pid) -> bool {
    let ctx = ctx_of(env);
    ctx.runtime
        .borrow()
        .get(pid)
        .map(|a| a.borrow().alive)
        .unwrap_or(false)
}

pub fn stop(env: &Env, pid: Pid) {
    let ctx = ctx_of(env);
    if let Some(a) = ctx.runtime.borrow().get(pid) {
        let mut a = a.borrow_mut();
        a.alive = false;
        a.mailbox.clear();
    }
}

pub fn current_pid(env: &Env) -> Option<Pid> {
    ctx_of(env).runtime.borrow().current
}

pub fn drain_all(env: &Env) {
    while step_one(env) {}
}

fn drain_until(env: &Env, slot: &Rc<RefCell<Option<CallReply>>>) {
    while slot.borrow().is_none() {
        if !step_one(env) {
            break;
        }
    }
}

fn enqueue_if_idle(runtime: &RefCell<Runtime>, actor_rc: &Rc<RefCell<Actor>>, pid: Pid) {
    if actor_rc.borrow().processing {
        return;
    }
    let mut rt = runtime.borrow_mut();
    if !rt.ready.contains(&pid) {
        rt.ready.push_back(pid);
    }
}

/// Single scheduler tick: pop one ready actor, deliver one message, return.
/// Returns false if the ready queue was empty (caller should stop looping).
fn step_one(env: &Env) -> bool {
    let ctx = ctx_of(env);

    let pid = match ctx.runtime.borrow_mut().ready.pop_front() {
        Some(p) => p,
        None => return false,
    };

    let actor_rc = match ctx.runtime.borrow().get(pid) {
        Some(a) => a,
        None => return true,
    };

    if !actor_rc.borrow().alive {
        return true;
    }

    let msg = actor_rc.borrow_mut().mailbox.pop_front();
    let Some(msg) = msg else { return true };

    actor_rc.borrow_mut().processing = true;
    let prev_current = {
        let mut rt = ctx.runtime.borrow_mut();
        let p = rt.current;
        rt.current = Some(pid);
        p
    };

    let (template, state) = {
        let a = actor_rc.borrow();
        (a.template.clone(), a.state.clone())
    };

    match msg {
        ActorMsg::Cast(m) => match invoke_cast(env, &template, m, state) {
            Ok(new_state) => actor_rc.borrow_mut().state = new_state,
            Err(e) => {
                let _ = writeln!(
                    ctx.out.borrow_mut(),
                    "[actor #PID<{}> crashed: {}]",
                    pid,
                    e
                );
                actor_rc.borrow_mut().alive = false;
            }
        },
        ActorMsg::Call { msg: m, reply } => match invoke_call(env, &template, m, state) {
            Ok((reply_val, new_state)) => {
                actor_rc.borrow_mut().state = new_state;
                *reply.borrow_mut() = Some(CallReply::Ok(reply_val));
            }
            Err(e) => {
                actor_rc.borrow_mut().alive = false;
                *reply.borrow_mut() =
                    Some(CallReply::Err(format!("handler crashed: {}", e).into()));
            }
        },
        ActorMsg::Stop => actor_rc.borrow_mut().alive = false,
    }

    ctx.runtime.borrow_mut().current = prev_current;
    actor_rc.borrow_mut().processing = false;

    let still_busy = {
        let a = actor_rc.borrow();
        a.alive && !a.mailbox.is_empty()
    };
    if still_busy {
        ctx.runtime.borrow_mut().ready.push_back(pid);
    }
    true
}

fn invoke_cast(env: &Env, template: &Value, msg: Value, state: Value) -> RuntimeResult<Value> {
    let Some(cb) = lookup_cb(template, "handle_cast") else {
        return Ok(state);
    };
    // ff multi-param functions are curried at parse time, so dispatch is two
    // unary `apply`s rather than one with `vec![msg, state]`.
    let intermediate = apply(env, cb, vec![msg])?;
    apply(env, intermediate, vec![state])
}

fn invoke_call(
    env: &Env,
    template: &Value,
    msg: Value,
    state: Value,
) -> RuntimeResult<(Value, Value)> {
    let Some(cb) = lookup_cb(template, "handle_call") else {
        return Err(RuntimeError::UnsupportedOperation(
            ":no_call_handler".into(),
        ));
    };
    let intermediate = apply(env, cb, vec![msg])?;
    let result = apply(env, intermediate, vec![state])?;
    if let Value::List(xs) = &result
        && xs.len() == 2
    {
        return Ok((xs.get(0).unwrap().clone(), xs.get(1).unwrap().clone()));
    }
    Err(RuntimeError::UnsupportedOperation(format!(
        "handle_call must return [reply, new_state], got {}",
        result
    )))
}
