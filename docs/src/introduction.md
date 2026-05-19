# Introduction

**ff** — *functional ff* — is a small, highly extensible, functional language
with strong Rust interoperability and native actor-based concurrency.

It is designed around a few ideas:

- **Everything is an expression.** Blocks, `if`, `match`, function literals,
  and `import` all return values.
- **Functions are values, and everything is curried.** `add(1, 2)` and
  `add(1)(2)` are the same thing.
- **Numbers are exact rationals.** `1 / 3 + 1 / 3 == 2 / 3` — no
  floating-point drift.
- **Immutable, persistent data structures.** Lists, objects, and sets
  share structure on update.
- **Pattern matching on shape.** The same patterns work in `match`, in
  destructuring assignments, and in function parameters.
- **Actors for concurrency.** Each actor processes its mailbox in order;
  spawn as many as you want and let the runtime schedule them.
- **Plain Rust closures become ff functions.** Anything with `serde`
  derives crosses the boundary without glue.

## A small example

```ff
# arithmetic is exact-rational
1 / 3 + 1 / 3                     # 2/3

# functions are curried; pipes thread data left-to-right
[1, 2, 3]
  |> map(x => x * x)
  |> reduce((a, b) => a + b, 0)   # 14

# pattern matching on shape, with cons (`::`) peeling one element
describe = match
  []         -> "empty",
  [_]        -> "one",
  _ :: _     -> "many",
  _          -> "other"

# actors are objects with atom-keyed callbacks
counter = {
  :init: () => 0,
  :handle_notify: (msg, n) => match msg
    [:add, x] -> n + x,
    :reset    -> 0
  :handle_request: (msg, n) => match msg
    :get -> [n, n]
}

[:ok, pid] = Actor.spawn(counter)
Actor.notify(pid, [:add, 5])
Actor.request(pid, :get)            # [:ok, 5]
```

## How to read these docs

The [Language](./language/values.md) section is a tour: read it
front-to-back the first time. Each chapter is short and self-contained.

The [Standard Library](./stdlib/overview.md) section is a reference —
skim the overview, then dip in for specific modules.

[Rust Interop](./interop.md) is for embedding ff inside a Rust program,
or extending ff with Rust-implemented natives.
