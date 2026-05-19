# Actors

ff models concurrency on Erlang-style **actors**: independent sequential
processes that communicate only by message passing. Each actor owns
some private state and processes its mailbox strictly in order;
concurrency comes from interleaving many actors. There is no shared
mutable memory.

## Defining an actor

An actor is just an object with atom-keyed callbacks:

| Key                | Signature                                      | Required |
|--------------------|------------------------------------------------|----------|
| `:init`            | `() => initial_state`                          | yes      |
| `:handle_notify`   | `(msg, state) => new_state`                    | optional |
| `:handle_request`  | `(msg, state) => [reply, new_state]`           | optional |

```ff
counter = {
  :init: () => 0,

  :handle_notify: (msg, n) => match msg
    [:add, x] -> n + x,
    :reset    -> 0,

  :handle_request: (msg, n) => match msg
    :get -> [n, n]
}
```

`:init` returns the starting state. `:handle_notify` handles fire-and-forget
messages and returns the new state. `:handle_request` handles synchronous
calls and returns a two-element list `[reply, new_state]`.

## Spawning

`Actor.spawn(actor)` starts a new actor and returns its pid as
`[:ok, pid]`:

```ff
[:ok, pid] = Actor.spawn(counter)
```

The spawn happens on a tokio multi-thread runtime; each actor parks on
its own blocking task, so independent actors progress in parallel on
different OS threads.

## Talking to an actor

```ff
Actor.notify(pid, [:add, 5])     # fire-and-forget, returns ()
Actor.notify(pid, [:add, 7])
Actor.request(pid, :get)         # synchronous, returns [:ok, 12]
```

- `notify` returns `()` immediately. The message is enqueued; the
  actor will get to it in mailbox order.
- `request` blocks the caller until the target's handler returns,
  then yields `[:ok, reply]` (or `[:error, …]` if something went
  wrong).

Per-actor ordering is strict — one handler runs at a time, and the
mailbox is FIFO.

## Error handling

| Situation                            | Result                                 |
|--------------------------------------|----------------------------------------|
| Target pid is dead or unknown        | `[:error, :no_proc]`                   |
| `Actor.self(self_pid)` self-request  | `[:error, :self_deadlock]`             |
| Handler raises (panic, type error)   | `[:error, "handler crashed: …"]` ; the actor dies but the script continues |

The caller always sees a result — a crashing actor never propagates an
exception into the caller's stack.

## Other primitives

| Function                     | Description                                            |
|------------------------------|--------------------------------------------------------|
| `Actor.self()`               | `[:ok, pid]` inside a handler; `[:error, :not_in_actor]` at top level |
| `Actor.alive(pid)`           | `true` if the actor is still running                   |
| `Actor.stop(pid)`            | request that the actor stop; returns `()`              |
| `Actor.run_until_idle()`     | drain all pending mailboxes (mostly for tests)         |

## A worked example

```ff
# A pub/sub registry.
registry = {
  :init: () => {:subs: []},

  :handle_notify: (msg, st) => match msg
    [:subscribe, pid] -> {:subs: pid :: st.subs},

  :handle_request: (msg, st) => match msg
    [:broadcast, m] -> (
      # notify each subscriber, return the count
      n = match st.subs
        _ :: _ -> (
          notify_all = (pids) => match pids
            p :: rest -> (
              Actor.notify(p, m)
              1 + notify_all(rest)
            ),
            _ -> 0
          notify_all(st.subs)
        ),
        _ -> 0
      [n, st]
    )
}

[:ok, reg] = Actor.spawn(registry)
```

## When to reach for actors

- **Long-lived state** that should be encapsulated and accessed
  serially (counters, caches, in-memory stores).
- **Producer/consumer** pipelines where work and back-pressure should
  be handled by separate processes.
- **Fan-out** to independent units of work that can run on different OS
  threads.

For pure transformations of values, just use plain functions and pipes.
Actors are for managing state and concurrency, not for structuring
computation.
