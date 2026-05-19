mod common;
use common::eval;

#[test]
fn spawn_returns_pid() {
    let src = r#"
Actor = import "Actor"
counter = {:init: () => 0}
[:ok, pid] = Actor.spawn(counter)
typeof(pid)
"#;
    assert_eq!(eval(src), ":pid");
}

#[test]
fn counter_sanity() {
    let src = r#"
Actor = import "Actor"

handle_call = (msg, n) => match msg
  :get -> [n, n]
handle_cast = (msg, n) => match msg
  [:add, x] -> n + x,
  :reset -> 0

counter = {
  :init: () => 10
  :handle_call: handle_call
  :handle_cast: handle_cast
}

[:ok, pid] = Actor.spawn(counter)
Actor.cast(pid, [:add, 5])
Actor.cast(pid, [:add, 7])
Actor.call(pid, :get)
"#;
    assert_eq!(eval(src), "[:ok, 22]");
}

#[test]
fn cast_then_call_in_order() {
    // Each cast appends its arg to a list; final call returns the list.
    // Verifies strict per-actor FIFO message processing.
    let src = r#"
Actor = import "Actor"

handle_call = (msg, state) => match msg
  :get -> [state, state]
handle_cast = (msg, state) => match msg
  [:push, x] -> x :: state

logger = {
  :init: () => []
  :handle_call: handle_call
  :handle_cast: handle_cast
}

[:ok, pid] = Actor.spawn(logger)
Actor.cast(pid, [:push, 1])
Actor.cast(pid, [:push, 2])
Actor.cast(pid, [:push, 3])
[:ok, result] = Actor.call(pid, :get)
result
"#;
    assert_eq!(eval(src), "[3, 2, 1]");
}

#[test]
fn nested_call_across_actors() {
    // A.relay forwards a ping to B and returns B's reply unchanged.
    // Verifies that a call from inside a handler nests properly.
    let src = r#"
Actor = import "Actor"

b_call = (msg, s) => match msg
  :ping -> [:pong, s]
b = {:init: () => (), :handle_call: b_call}
[:ok, b_pid] = Actor.spawn(b)

a_call = (msg, s) => match msg
  [:relay, target] -> (
    [:ok, reply] = Actor.call(target, :ping)
    [reply, s]
  )
a = {:init: () => (), :handle_call: a_call}
[:ok, a_pid] = Actor.spawn(a)

Actor.call(a_pid, [:relay, b_pid])
"#;
    assert_eq!(eval(src), "[:ok, :pong]");
}

#[test]
fn self_call_is_deadlock() {
    let src = r#"
Actor = import "Actor"

call_handler = (msg, s) => match msg
  :try_self -> (
    [:ok, me] = Actor.self()
    result = Actor.call(me, :ping)
    [result, s]
  ),
  :ping -> [:pong, s]

actor = {:init: () => (), :handle_call: call_handler}
[:ok, pid] = Actor.spawn(actor)
Actor.call(pid, :try_self)
"#;
    assert_eq!(eval(src), "[:ok, [:error, :self_deadlock]]");
}

#[test]
fn call_dead_pid_returns_no_proc() {
    let src = r#"
Actor = import "Actor"
actor = {:init: () => (), :handle_call: (msg, s) => [:ok, s]}
[:ok, pid] = Actor.spawn(actor)
Actor.stop(pid)
Actor.call(pid, :hi)
"#;
    assert_eq!(eval(src), "[:error, :no_proc]");
}

#[test]
fn cast_to_dead_pid_drops_silently() {
    let src = r#"
Actor = import "Actor"
actor = {:init: () => (), :handle_call: (msg, s) => [:ok, s]}
[:ok, pid] = Actor.spawn(actor)
Actor.stop(pid)
Actor.cast(pid, :anything)
"#;
    assert_eq!(eval(src), "()");
}

#[test]
fn handler_crash_returns_error_and_kills_actor() {
    let src = r#"
Actor = import "Actor"

call_handler = (msg, s) => match msg
  :crash -> [1 / 0, s],
  :ping -> [:pong, s]

actor = {:init: () => (), :handle_call: call_handler}
[:ok, pid] = Actor.spawn(actor)
[:error, _] = Actor.call(pid, :crash)
Actor.alive(pid)
"#;
    assert_eq!(eval(src), "false");
}

#[test]
fn self_at_top_level_is_not_in_actor() {
    let src = r#"
Actor = import "Actor"
Actor.self()
"#;
    assert_eq!(eval(src), "[:error, :not_in_actor]");
}

#[test]
fn pid_equality_by_id() {
    let src = r#"
Actor = import "Actor"
actor = {:init: () => ()}
[:ok, a] = Actor.spawn(actor)
[:ok, b] = Actor.spawn(actor)
[a == a, a == b]
"#;
    assert_eq!(eval(src), "[true, false]");
}

#[test]
fn missing_call_handler() {
    let src = r#"
Actor = import "Actor"
actor = {:init: () => ()}
[:ok, pid] = Actor.spawn(actor)
Actor.call(pid, :anything)
"#;
    // The handler-absent path surfaces as [:error, "..."] with a string
    // body (handler-crashed shape) rather than a clean :no_call_handler atom;
    // that's documented in the runtime and acceptable for v1.
    let result = eval(src);
    assert!(
        result.starts_with("[:error,"),
        "expected error tuple, got {}",
        result
    );
}
