# Pattern Matching

Patterns appear in three places:

1. The left-hand side of an assignment.
2. Function parameters.
3. Each arm of a `match` expression.

The same vocabulary of patterns is shared by all three.

## Pattern vocabulary

| Pattern             | Matches                                              |
|---------------------|------------------------------------------------------|
| `x` (identifier)    | anything, binds it to `x`                            |
| `_`                 | anything, binds nothing                              |
| `42`, `"s"`, `true` | the exact literal                                    |
| `:ok`               | the matching atom                                    |
| `[p, q, ..rest]`    | a list/range/cons-spine; `..rest` binds the rest     |
| `h :: t`            | peel one element; `t` is the tail (any sequence)     |
| `{:k: p, ..}`       | object containing key `:k` matching `p`              |
| `{p, q}`            | a set containing `p` and `q`                         |
| `()`                | the unit value                                       |

`..` inside a list pattern matches zero or more elements; bare `..`
discards them, `..name` binds them as a list.

### Cons patterns

`h :: t` is type-polymorphic. It peels a single element off any
sequence — list, string (one character), set, or lazy range:

```ff
sum = match
  h :: t -> h + sum(t),
  _      -> 0

sum([1, 2, 3, 4])                  # 10

first_char = match
  c :: _ -> c
first_char("hello")                # "h"
```

Because cons doesn't force its tail, `match` on infinite ranges works
fine:

```ff
match [0..]
  a :: b :: c :: _ -> [a, b, c]    # [0, 1, 2]
```

### Object patterns

Object patterns match by key. Missing keys cause the pattern to fail.

```ff
{:name: n, :age: a} = {:name: "Ada", :age: 36}    # n = "Ada", a = 36

# Shorthand: `{x}` desugars to `{"x": x}` for destructuring modules.
{map, filter} = import "list_utils.ff"
```

The `..` rest pattern is allowed in object patterns and discards the
remaining keys.

## `match`

`match` dispatches on shape, returning the value of the first arm that
matches:

```ff
describe = match
  []          -> "empty",
  [x]         -> "one: " + x,
  [x, y]      -> "two",
  _           -> "many"
```

Two forms of `match`:

- **Multi-line**: `match SCRUTINEE\n  pat -> expr, …`. A newline
  separates the scrutinee from the first arm.
- **Single-line**: `match SCRUTINEE: pat -> expr, …`. A colon plays
  the same role.

Arms are separated by commas; newlines around the comma are fine.

### Bare `match` (one-arg function)

When you omit the scrutinee, `match` evaluates to a one-argument
function whose argument is matched against the arms. This is the
idiomatic way to define case-analyzing functions:

```ff
describe = match
  []     -> "empty",
  _ :: _ -> "non-empty"

describe([])              # "empty"
describe([1, 2])          # "non-empty"
```

### Guards

`if EXPR` after the pattern refines an arm. The guard runs in a scope
where the pattern's bindings are visible; if it returns false the arm is
skipped and the next is tried:

```ff
sign = match
  n if n < 0 -> "neg",
  0          -> "zero",
  _          -> "pos"
```

### No-match behaviour

If no arm matches, the `match` raises a runtime error. To make a match
total, end with `_ -> …`.

## Destructuring assignment

The same patterns work in plain `=`:

```ff
[head, ..tail] = [1, 2, 3, 4]    # head = 1, tail = [2, 3, 4]
[x, y]         = [10, 20]
{:lang: lang}  = {:lang: "ff"}
```

If the pattern doesn't match, the assignment raises a runtime error.

## Patterns as parameters

Function parameters are patterns, so destructuring composes naturally:

```ff
first = [a, ..] => a
add   = ([a, b], c) => a + b + c
greet = {:name: n} => "Hi, " + n
```

A parameter that fails to match raises a runtime error at the call
site. For partial patterns where the input might not have the expected
shape, prefer `match` inside the function body with a fallback arm.
