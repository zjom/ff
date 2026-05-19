mod common;
use common::eval;

#[test]
fn spawn_returns_pid() {
    let src = r#"
counter = {:init: () => 0}
[:ok, pid] = Actor.spawn(counter)
typeof(pid)
"#;
    assert_eq!(eval(src), ":pid");
}

#[test]
fn counter_sanity() {
    let src = r#"
handle_request = (msg, n) => match msg
  :get -> [n, n]
handle_notify = (msg, n) => match msg
  [:add, x] -> n + x,
  :reset -> 0

counter = {
  :init: () => 10
  :handle_request: handle_request
  :handle_notify: handle_notify
}

[:ok, pid] = Actor.spawn(counter)
Actor.notify(pid, [:add, 5])
Actor.notify(pid, [:add, 7])
Actor.request(pid, :get)
"#;
    assert_eq!(eval(src), "[:ok, 22]");
}

#[test]
fn notify_then_request_in_order() {
    // Each notify appends its arg to a list; final request returns the list.
    // Verifies strict per-actor FIFO message processing.
    let src = r#"
handle_request = (msg, state) => match msg
  :get -> [state, state]
handle_notify = (msg, state) => match msg
  [:push, x] -> x :: state

logger = {
  :init: () => []
  :handle_request: handle_request
  :handle_notify: handle_notify
}

[:ok, pid] = Actor.spawn(logger)
Actor.notify(pid, [:push, 1])
Actor.notify(pid, [:push, 2])
Actor.notify(pid, [:push, 3])
[:ok, result] = Actor.request(pid, :get)
result
"#;
    assert_eq!(eval(src), "[3, 2, 1]");
}

#[test]
fn nested_request_across_actors() {
    // A.relay forwards a ping to B and returns B's reply unchanged.
    // Verifies that a request from inside a handler nests properly.
    let src = r#"
b_request = (msg, s) => match msg
  :ping -> [:pong, s]
b = {:init: () => (), :handle_request: b_request}
[:ok, b_pid] = Actor.spawn(b)

a_request = (msg, s) => match msg
  [:relay, target] -> (
    [:ok, reply] = Actor.request(target, :ping)
    [reply, s]
  )
a = {:init: () => (), :handle_request: a_request}
[:ok, a_pid] = Actor.spawn(a)

Actor.request(a_pid, [:relay, b_pid])
"#;
    assert_eq!(eval(src), "[:ok, :pong]");
}

#[test]
fn self_request_is_deadlock() {
    let src = r#"
request_handler = (msg, s) => match msg
  :try_self -> (
    [:ok, me] = Actor.self()
    result = Actor.request(me, :ping)
    [result, s]
  ),
  :ping -> [:pong, s]

actor = {:init: () => (), :handle_request: request_handler}
[:ok, pid] = Actor.spawn(actor)
Actor.request(pid, :try_self)
"#;
    assert_eq!(eval(src), "[:ok, [:error, :self_deadlock]]");
}

#[test]
fn request_dead_pid_returns_no_proc() {
    let src = r#"
actor = {:init: () => (), :handle_request: (msg, s) => [:ok, s]}
[:ok, pid] = Actor.spawn(actor)
Actor.stop(pid)
Actor.request(pid, :hi)
"#;
    assert_eq!(eval(src), "[:error, :no_proc]");
}

#[test]
fn notify_to_dead_pid_drops_silently() {
    let src = r#"
actor = {:init: () => (), :handle_request: (msg, s) => [:ok, s]}
[:ok, pid] = Actor.spawn(actor)
Actor.stop(pid)
Actor.notify(pid, :anything)
"#;
    assert_eq!(eval(src), "()");
}

#[test]
fn handler_crash_returns_error_and_kills_actor() {
    let src = r#"
request_handler = (msg, s) => match msg
  :crash -> [1 / 0, s],
  :ping -> [:pong, s]

actor = {:init: () => (), :handle_request: request_handler}
[:ok, pid] = Actor.spawn(actor)
[:error, _] = Actor.request(pid, :crash)
Actor.alive(pid)
"#;
    assert_eq!(eval(src), "false");
}

#[test]
fn self_at_top_level_is_not_in_actor() {
    let src = r#"
Actor.self()
"#;
    assert_eq!(eval(src), "[:error, :not_in_actor]");
}

#[test]
fn pid_equality_by_id() {
    let src = r#"
actor = {:init: () => ()}
[:ok, a] = Actor.spawn(actor)
[:ok, b] = Actor.spawn(actor)
[a == a, a == b]
"#;
    assert_eq!(eval(src), "[true, false]");
}

#[test]
fn missing_request_handler() {
    let src = r#"
actor = {:init: () => ()}
[:ok, pid] = Actor.spawn(actor)
Actor.request(pid, :anything)
"#;
    // The handler-absent path surfaces as [:error, "..."] with a string
    // body (handler-crashed shape) rather than a clean :no_request_handler atom;
    // that's documented in the runtime and acceptable for v1.
    let result = eval(src);
    assert!(
        result.starts_with("[:error,"),
        "expected error tuple, got {}",
        result
    );
}
