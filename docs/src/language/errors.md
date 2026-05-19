# Error Handling

ff has no exceptions in user code. Functions that may fail return a
tagged two-element list — `[:ok, value]` on success or
`[:error, info]` on failure — and the caller pattern-matches to handle
both.

## The convention

```ff
safe_div = (a, b) =>
  if b == 0
    then [:error, "div0"]
    else [:ok, a / b]

handle = match
  [:ok, v]      -> v,
  [:error, msg] -> -1

handle(safe_div(10, 2))           # 5
handle(safe_div(10, 0))           # -1
```

This pattern is followed by every native function that can fail
(`Json.parse`, `Http.get`, `Fs.open`, `String.parse_int`, `Actor.request`,
…). When you're embedding Rust code, use [`FfResult`](../interop.md) and
the conversion is automatic.

## Panicking

For genuinely unrecoverable conditions — invariant violations,
assertion failures — use `panic`:

```ff
panic "this should never happen"
```

`panic` aborts the current evaluation. In a script, it exits with a
non-zero status; in the REPL, it returns to the prompt; inside an
actor, it crashes only that actor and is reported to callers as
`[:error, "handler crashed: …"]` (see [Actors](../concurrency.md)).

The stdlib `assert` builds on `panic`:

```ff
assert = args => (
  [val, msg] = match args
    [val, msg] -> [val, msg],
    val        -> [val, "assert failed"]

  if val != default(val)
    then val
    else panic msg
)

assert(1 == 1)                         # 1
assert([1 == 2, "nope"])               # panic: nope
```

## Why two-element lists?

A tagged list is just data. It can be passed around, stored, logged,
forwarded across an actor boundary, and matched against the same way
any other list is matched. There's no special-case exception machinery
to learn, and "errors" interoperate cleanly with normal control flow.

## Combining results

There's no built-in `and_then` / monadic plumbing — just nested
`match`:

```ff
parse_and_double = s => match String.parse_int(s)
  [:ok, n]      -> [:ok, n * 2],
  [:error, msg] -> [:error, msg]
```

If you find yourself doing this a lot, define a `try` operator:

```ff
(>>=) = (result, f) => match result
  [:ok, v]      -> f(v),
  [:error, msg] -> [:error, msg]

String.parse_int("21")
  >>= (n => [:ok, n * 2])              # [:ok, 42]
```
