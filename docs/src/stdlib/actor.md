# Actor

Actor primitives. Pre-imported by the prelude as `Actor`.

See [Concurrency](../concurrency.md) for the full model and worked
examples. This page is the function reference.

## Functions

| Function                       | Description                                  |
|--------------------------------|----------------------------------------------|
| `Actor.spawn(obj)`             | Start a new actor; `[:ok, pid]` / `[:error, msg]` |
| `Actor.notify(pid, msg)`       | Fire-and-forget; returns `()` immediately    |
| `Actor.request(pid, msg)`      | Synchronous call; `[:ok, reply]` / `[:error, …]` |
| `Actor.self()`                 | `[:ok, pid]` inside a handler, `[:error, :not_in_actor]` otherwise |
| `Actor.alive(pid)`             | `true` if the actor is running               |
| `Actor.stop(pid)`              | Request a graceful stop; returns `()`        |
| `Actor.run_until_idle()`       | Drain all mailboxes (test helper)            |

## Actor object shape

`Actor.spawn` expects an object with atom-keyed callbacks:

```ff
{
  :init:           () => state,
  :handle_notify:  (msg, state) => new_state,           # optional
  :handle_request: (msg, state) => [reply, new_state]   # optional
}
```

## Failure modes

| Condition                              | Result                            |
|----------------------------------------|-----------------------------------|
| pid not found / actor stopped          | `[:error, :no_proc]`              |
| `Actor.request(self_pid)` in handler   | `[:error, :self_deadlock]`        |
| Handler raises an exception            | actor dies; caller sees `[:error, "handler crashed: …"]` |

Crashes are contained — they never propagate into the caller's stack.
