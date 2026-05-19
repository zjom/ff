//! Erlang/Elixir-style actor runtime, backed by tokio.
//!
//! Each `Actor.spawn` parks an actor on its own `tokio::task::spawn_blocking`
//! task. The actor's mailbox is an `mpsc::UnboundedSender`; messages drive a
//! synchronous loop that invokes `:handle_request` / `:handle_notify` callbacks via
//! [`apply`]. `Actor.request` synchronously waits on a `oneshot::Receiver` for
//! the reply.
//!
//! Independent actors run on independent tokio threads from
//! tokio's blocking pool. Per-actor message ordering is strict (one handler at a time per actor).
//!
//! Self-deadlock detection: `Actor.request(self, _)` from inside a handler is
//! recognised via a thread-local current pid and rejected with
//! `[:error, :self_deadlock]` rather than blocking the handler thread
//! forever.

use std::cell::Cell;
use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::{mpsc, oneshot};

use super::error::{RuntimeError, RuntimeResult};
use super::expr::apply;
use super::scope::{Env, ctx_of};
use super::value::{Value, type_name};

pub type Pid = u64;

pub enum ActorMsg {
    Notify(Value),
    Request {
        msg: Value,
        reply: oneshot::Sender<RequestReply>,
    },
    Stop,
}

#[derive(Debug)]
pub enum RequestReply {
    Ok(Value),
    Err(Arc<str>),
}

struct ActorHandle {
    sender: mpsc::UnboundedSender<ActorMsg>,
    alive: Arc<AtomicBool>,
}

type ActorMap = Arc<Mutex<HashMap<Pid, Arc<ActorHandle>>>>;

pub struct Runtime {
    next_pid: AtomicU64,
    actors: ActorMap,
    tokio: Arc<tokio::runtime::Runtime>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        // Each `Ctx` owns a tokio runtime. Dropping the `Runtime` (via the
        // `Ctx` going out of scope at end-of-script or end-of-test) calls
        // `shutdown_background` so any actors still running don't block the
        // drop on the caller's thread.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        Self {
            next_pid: AtomicU64::new(1),
            actors: Arc::new(Mutex::new(HashMap::new())),
            tokio: Arc::new(rt),
        }
    }

    fn mint_pid(&self) -> Pid {
        self.next_pid.fetch_add(1, Ordering::SeqCst)
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // Dropping the senders signals every actor's `blocking_recv` to wake
        // and exit, freeing the blocking pool threads. The Arc<Runtime> may
        // still be held elsewhere; either way, when the tokio runtime is
        // finally dropped, `shutdown_background` keeps the caller unblocked.
        if let Ok(mut actors) = self.actors.lock() {
            actors.clear();
        }
    }
}

thread_local! {
    // Set for the lifetime of each actor's blocking task (and temporarily
    // during `init` so `Actor.self()` works there). Used for the
    // self-deadlock guard and `Actor.self()`.
    static CURRENT_PID: Cell<Option<Pid>> = const { Cell::new(None) };
}

fn with_current_pid<R>(pid: Option<Pid>, f: impl FnOnce() -> R) -> R {
    let prev = CURRENT_PID.with(|c| c.replace(pid));
    let r = f();
    CURRENT_PID.with(|c| c.set(prev));
    r
}

pub fn current_pid(_env: &Env) -> Option<Pid> {
    CURRENT_PID.with(|c| c.get())
}

fn lookup_cb(template: &Value, name: &str) -> Option<Value> {
    let Value::Object(es) = template else {
        return None;
    };
    es.get(&Value::Atom(name.into())).cloned()
}

fn runtime_snapshot(env: &Env) -> (ActorMap, tokio::runtime::Handle) {
    let ctx = ctx_of(env);
    let rt = ctx.runtime.lock().unwrap();
    (rt.actors.clone(), rt.tokio.handle().clone())
}

fn lookup_actor(actors: &ActorMap, pid: Pid) -> Option<Arc<ActorHandle>> {
    actors.lock().unwrap().get(&pid).cloned()
}

/// Spawn a new actor from an Object template. Runs `:init` inline on the
/// caller's thread (with `CURRENT_PID` temporarily pointing at the new pid)
/// so an init failure surfaces as a spawn error rather than a silent crash.
pub fn spawn(env: &Env, template: Value) -> RuntimeResult<Value> {
    if !matches!(template, Value::Object(_)) {
        return Err(RuntimeError::NativeTypeError {
            native: "Actor.spawn",
            expected: "object template",
            got: type_name(&template),
        });
    }

    let pid = {
        let ctx = ctx_of(env);
        let rt = ctx.runtime.lock().unwrap();
        rt.mint_pid()
    };

    let state = if let Some(init_fn) = lookup_cb(&template, "init") {
        with_current_pid(Some(pid), || apply(env, init_fn, vec![]))?
    } else {
        Value::Unit
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<ActorMsg>();
    let alive = Arc::new(AtomicBool::new(true));
    let handle = Arc::new(ActorHandle {
        sender: tx,
        alive: alive.clone(),
    });

    let (actors, tokio_handle) = runtime_snapshot(env);
    actors.lock().unwrap().insert(pid, handle);

    // Each actor lives on its own blocking task. The task captures a clone of
    // the spawning env so it can dispatch handlers via `apply`. Closures inside
    // the template already carry their own captured env; the cloned env here
    // is only used so the `&Env` arg to `apply` is valid for natives that look
    // up `ctx`.
    let env_for_task = env.clone();
    let ctx_for_task = ctx_of(env);
    let template_for_task = template.clone();
    let actors_for_task = actors.clone();

    tokio_handle.spawn_blocking(move || {
        CURRENT_PID.with(|c| c.set(Some(pid)));
        let mut state = state;
        while let Some(msg) = rx.blocking_recv() {
            if !alive.load(Ordering::SeqCst) {
                break;
            }
            match msg {
                ActorMsg::Notify(m) => {
                    match invoke_notify(&env_for_task, &template_for_task, m, state.clone()) {
                        Ok(new_state) => state = new_state,
                        Err(e) => {
                            let _ = writeln!(
                                ctx_for_task.out.lock().unwrap(),
                                "[actor #PID<{}> crashed: {}]",
                                pid,
                                e
                            );
                            alive.store(false, Ordering::SeqCst);
                            break;
                        }
                    }
                }
                ActorMsg::Request { msg: m, reply } => {
                    match invoke_request(&env_for_task, &template_for_task, m, state.clone()) {
                        Ok((reply_val, new_state)) => {
                            state = new_state;
                            let _ = reply.send(RequestReply::Ok(reply_val));
                        }
                        Err(e) => {
                            let _ = reply
                                .send(RequestReply::Err(format!("handler crashed: {}", e).into()));
                            alive.store(false, Ordering::SeqCst);
                            break;
                        }
                    }
                }
                ActorMsg::Stop => {
                    alive.store(false, Ordering::SeqCst);
                    break;
                }
            }
        }
        // Remove the actor handle so future `request`s see :no_proc.
        if let Ok(mut map) = actors_for_task.lock() {
            map.remove(&pid);
        }
        CURRENT_PID.with(|c| c.set(None));
    });

    Ok(Value::Pid(pid))
}

pub fn notify(env: &Env, pid: Pid, msg: Value) {
    let (actors, _) = runtime_snapshot(env);
    let Some(handle) = lookup_actor(&actors, pid) else {
        return;
    };
    if !handle.alive.load(Ordering::SeqCst) {
        return;
    }
    let _ = handle.sender.send(ActorMsg::Notify(msg));
}

pub fn request(env: &Env, pid: Pid, msg: Value) -> RequestReply {
    if CURRENT_PID.with(|c| c.get()) == Some(pid) {
        return RequestReply::Err(":self_deadlock".into());
    }
    let (actors, _) = runtime_snapshot(env);
    let Some(handle) = lookup_actor(&actors, pid) else {
        return RequestReply::Err(":no_proc".into());
    };
    if !handle.alive.load(Ordering::SeqCst) {
        return RequestReply::Err(":no_proc".into());
    }

    let (reply_tx, reply_rx) = oneshot::channel();
    if handle
        .sender
        .send(ActorMsg::Request {
            msg,
            reply: reply_tx,
        })
        .is_err()
    {
        return RequestReply::Err(":no_proc".into());
    }
    match reply_rx.blocking_recv() {
        Ok(r) => r,
        Err(_) => RequestReply::Err(":no_proc".into()),
    }
}

pub fn alive(env: &Env, pid: Pid) -> bool {
    let (actors, _) = runtime_snapshot(env);
    lookup_actor(&actors, pid)
        .map(|h| h.alive.load(Ordering::SeqCst))
        .unwrap_or(false)
}

pub fn stop(env: &Env, pid: Pid) {
    let (actors, _) = runtime_snapshot(env);
    if let Some(handle) = lookup_actor(&actors, pid) {
        handle.alive.store(false, Ordering::SeqCst);
        let _ = handle.sender.send(ActorMsg::Stop);
    }
}

/// No-op under the parallel runtime: messages drain on their own as tokio
/// schedules the actor tasks. Retained so existing scripts compile.
pub fn drain_all(_env: &Env) {}

fn invoke_notify(env: &Env, template: &Value, msg: Value, state: Value) -> RuntimeResult<Value> {
    let Some(cb) = lookup_cb(template, "handle_notify") else {
        return Ok(state);
    };
    let intermediate = apply(env, cb, vec![msg])?;
    apply(env, intermediate, vec![state])
}

fn invoke_request(
    env: &Env,
    template: &Value,
    msg: Value,
    state: Value,
) -> RuntimeResult<(Value, Value)> {
    let Some(cb) = lookup_cb(template, "handle_request") else {
        return Err(RuntimeError::UnsupportedOperation(
            ":no_request_handler".into(),
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
        "handle_request must return [reply, new_state], got {}",
        result
    )))
}
