# Rust Interop

ff is implemented as a Rust library; you can embed it in any Rust
program and exchange values both ways. All conversion goes through
`serde` — anything with `Serialize` / `DeserializeOwned` works without
hand-written glue.

## Setting up an environment

```rust
use f2::interpreter::Scope;
use f2::prelude;

let env = Scope::new();
prelude::install(&env);
```

`Scope::new()` returns a fresh top-level scope. `prelude::install`
wires up everything documented in the
[Prelude](./stdlib/prelude.md) page.

## Pushing Rust values into ff

```rust
use f2::interop::define_value;
use serde::Serialize;

#[derive(Serialize)]
struct User { name: String, age: u32 }

define_value(&env, "me", &User { name: "Ada".into(), age: 36 }).unwrap();
```

After this, ff scripts evaluated in `env` see `me` as an object with
`:name` and `:age` atom-keys.

## Registering Rust functions

`register` lifts a Rust closure into the ff environment as a curried
native function. Arguments deserialize from ff values; the return type
serializes back.

```rust
use f2::interop::register;

register(&env, "greet", |u: User| format!("Hello, {}!", u.name));
```

Scripts can now call `greet(me)` and either `greet(me)` or `greet(me)`
(via partial application — `greet` is curried like everything else).

### Fallible natives

Return `FfResult<T>` to surface a tagged pair to the caller:

```rust
use f2::interop::FfResult;

register(&env, "checked_div", |a: i64, b: i64| -> FfResult<i64> {
    if b == 0 {
        FfResult::Err("divide by zero".into())
    } else {
        FfResult::Ok(a / b)
    }
});
```

In ff:

```ff
match checked_div(10, 0)
  [:ok, n]      -> n,
  [:error, msg] -> println("oops: " + msg)
```

Any `Result<T, E: Display>` converts in with `.into()`:

```rust
register(&env, "read", |path: String| -> FfResult<String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string()).into()
});
```

## Evaluating scripts

```rust
use f2::parser::parse;
use f2::interpreter::eval_program;

let prog = parse("greet(me)").unwrap();
let result = eval_program(&prog, &env).unwrap();
assert_eq!(result.to_string(), "\"Hello, Ada!\"");
```

`eval_program` returns a `Value` — the same enum the interpreter uses
internally. Use `from_value` to pull it back into a typed Rust value:

```rust
use f2::interop::from_value;

let n: i64 = from_value(eval_program(&parse("21 * 2").unwrap(), &env).unwrap()).unwrap();
assert_eq!(n, 42);
```

## Lower-level natives

`register` is the right tool when arguments and return values are
serde-friendly. For natives that need to inspect raw `Value`s — to
build a lazy stream, accept polymorphic arguments, or call back into
the actor runtime — use the `native!` macro from `f2::interop`. It
hands you the execution `Env` and an argument slice directly.

See `src/prelude/fs.rs` for a worked example that mixes both styles:
straightforward operations go through `register` / `native_fn`, while
`file.lines` uses `native!` because it builds a lazy cons spine.

## Argument convention

When you register multi-argument natives, **put the data argument
last** so calls compose under `|>`:

```rust
register(&env, "Greet.format", |greeting: String, name: String| -> String {
    format!("{}, {}!", greeting, name)
});
```

```ff
"Ada" |> Greet.format("Hello")        # "Hello, Ada!"
```

This is the convention every prelude and built-in module function
follows; user-supplied natives should match it.
