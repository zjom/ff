# Functions

Functions are first-class values. `=>` builds a lambda.

```ff
inc   = x => x + 1
add   = (x, y) => x + y
add3  = a b c => a + b + c
zero  = () => 0
```

Parameters can be space-separated or comma-separated, with or without
parens. Parens are *required* for zero parameters (`() => …`).

## Application

Call with parentheses, or by juxtaposition:

```ff
inc(5)         # 6
inc 5          # 6   — juxtaposition is application
add 3 4        # 7
```

Juxtaposition is left-associative, so `f g h` is `(f g) h`.

## Currying

**Every function is curried.** A multi-arg function applied to fewer
arguments returns a function expecting the rest:

```ff
add  = (x, y) => x + y
add5 = add(5)
add5(10)       # 15

# you can also call it like this — currying makes both equivalent:
add(5)(10)     # 15
```

This is true for native functions too. `map(f)` is a function awaiting
its list; `map(f, xs)` calls it immediately.

## Destructuring parameters

Any [pattern](./patterns.md) that works on the left of `=` works as a
parameter. The function fails at the call site if the argument doesn't
match.

```ff
first = [a, ..] => a
key   = [k, _] :: _ => k        # first key of an atom-keyed object
name  = {:name: n} => n
sum   = ([a, b], c) => a + b + c
```

For partial patterns (e.g. cons on a possibly-empty list), prefer
[`match`](./patterns.md#match) with a fallback arm so the failure is
surfaced cleanly rather than at the call site.

## Pipes and composition

The prelude defines three operators for threading values through
functions:

```ff
(|>) = (x, f) => f(x)            # pipe: x |> f == f(x)
(<<) = (f, g) => x => f(g(x))    # left composition
(>>) = (f, g) => x => g(f(x))    # right composition
```

```ff
[1, 2, 3]
  |> map(x => x * x)
  |> reduce((a, b) => a + b, 0)        # 14

double_then_inc = inc << double         # apply double, then inc
inc_then_double = inc >> double         # apply inc, then double
```

By convention, multi-argument prelude functions take the **data argument
last** so they compose under `|>`. `map(f, xs)`, `filter(p, xs)`,
`reduce(f, init, xs)`, `Object.get(k, obj)` all follow this pattern.

## Operators as values

Wrap any operator in parens to use it as a regular function:

```ff
plus = (+)
[1, 2, 3] |> reduce(plus, 0)            # 6
```

This works for built-in operators (`(+)`, `(*)`, `(::)`, …) and for any
[custom operator](./operators.md) you define.

## Optional and variadic arguments

There's no language-level support for either. Both fall out of pattern
matching on a list of arguments — see the
[`assert`](../stdlib/prelude.md#assert) source in the stdlib for a
worked example.
