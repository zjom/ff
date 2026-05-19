# Prelude

The prelude is bound automatically before any script runs. It's small
and entirely written in ff on top of the native built-ins — you can
read the source in `src/prelude/stdlib.ff`.

## Pipe and composition

```ff
(|>) = (x, f) => f(x)               # x |> f         ==  f(x)
(<<) = (f, g) => x => f(g(x))       # (f << g)(x)    ==  f(g(x))
(>>) = (f, g) => x => g(f(x))       # (f >> g)(x)    ==  g(f(x))
```

```ff
[1, 2, 3] |> map(x => x * x) |> reduce((a, b) => a + b, 0)   # 14

double_then_inc = inc << double
inc_then_double = inc >> double
```

## Collection combinators

All operate on lists, cons-spines, ranges, and strings (where it makes
sense — `map` over a string maps per-character cons).

### `map(f, xs)`

```ff
map(x => x * 2, [1, 2, 3])         # [2, 4, 6]
```

### `filter(p, xs)`

```ff
filter(x => x % 2 == 0, [1..=10])  # [2, 4, 6, 8, 10]
```

### `reduce(f, init, xs)`

Left fold:

```ff
reduce((a, b) => a + b, 0, [1, 2, 3])    # 6
```

Pipe-friendly: `xs |> reduce(f, 0)`.

### `fold(f, init, xs)`

Right fold (not tail-recursive — use `reduce` for large inputs unless
right-associativity matters):

```ff
fold((x, acc) => x :: acc, [], [1, 2, 3])  # [1, 2, 3]
```

### `take(n, xs)`

Returns the first `n` elements as a list. Works on lazy ranges and
infinite sequences:

```ff
[0..] |> filter(x => x % 2 == 0) |> take 5      # [0, 2, 4, 6, 8]
```

### `rev(xs)`

Reverses a sequence into a list:

```ff
rev([1, 2, 3])                     # [3, 2, 1]
```

## `assert`

```ff
assert(1 == 1)                     # 1
assert([1 == 2, "msg"])            # panic: msg
```

Takes either a bare value or a `[val, msg]` pair. Panics with `msg` (or
`"assert failed"`) when `val` equals its `default`.

## `panic`

```ff
panic "boom"
```

Aborts the current evaluation with the given message. There's no
recovery — see [Error Handling](../language/errors.md) for the
convention you should normally use instead.

## `typeof` and `default`

```ff
typeof 42                          # :number
typeof "hi"                        # :string
typeof [1, 2]                      # :list
typeof :ok                         # :atom

default(42)                        # 0
default("hi")                      # ""
default([1, 2])                    # []
default({:k: 1})                   # {}
```

`default` is unsupported for atoms, functions, and pids — those don't
have a meaningful zero.

## `print` and `println`

```ff
print("no newline")
println("with newline")
```

Both write to stdout. In the REPL, `print` also adds a trailing newline
for readability; in script mode it doesn't.

## `::` (cons)

The cons operator is also bound as a function under the name `::`, so
you can `(::)` it like any other operator:

```ff
[(::), 1, [2, 3]]                  # contains the cons function
((::))(1, [2, 3])                  # [1, 2, 3]
```

## Pre-imported modules

`Object` and `Actor` are bound by the prelude — you can use them
without an explicit `import`.
