# ff

> functional ff

A small, expression-oriented functional language. Everything is an expression,
collections are immutable, functions curry, and pattern matching is the main
control-flow tool. Cons (`::`) is lazy in its tail, so the same `map`/`filter`
work on infinite streams as on finite lists.

## at a glance

```ff
# arithmetic is exact-rational; `**` and `^` are power
1 + 2 * 3          # 7
2 ** 10            # 1024
1 / 3 + 1 / 3      # 2/3 — no floating-point drift

# bindings are immutable within a scope; rebinding shadows
greeting = "hello"
println(greeting + ", world")
```

## values

```ff
# numbers, strings, bools
42
"abc" + "def"      # "abcdef"
true && !false     # true

# `()` is the unit value
()

# three collections — lists, dicts, sets. Lists are heterogeneous.
[1, "two", true]
{"name": "ada", "age": 36}
{1, 2, 2, 3}             # {1, 2, 3} — dedup

# `.` indexes lists by position and dicts by key
[10, 20, 30].0           # 10
{"x": {"y": 7}}.x.y      # 7
```

Layouts are forgiving: commas and newlines are interchangeable inside `[]`,
`{}`, and call args.

```ff
xs = [
  1
  2, 3
  4,
]
```

## functions

Functions are values. `=>` builds a lambda; parameters can be comma- or
space-separated, with or without parens.

```ff
inc   = x => x + 1
add   = (x, y) => x + y
add3  = a b c => a + b + c

inc(5)         # 6
inc 5          # 6  — juxtaposition is application
add(3)(4)      # 7  — everything is curried
add3 1 2 3     # 6

# partial application falls out of currying
add5 = add(5)
add5(10)       # 15
```

The pipe operator `|>` lives in the prelude:

```ff
[1, 2, 3] |> map(x => x * x) |> reduce((a, b) => a + b, 0)   # 14
```

## pattern matching

`match` dispatches on shape. The scrutinee is optional — a bare `match` is a
one-argument function, which is the idiomatic way to define case-analyzing
functions.

```ff
describe = match
  []          -> "empty",
  [x]         -> "one",
  [x, y]      -> "two",
  _           -> "many"

describe([1, 2, 3])      # "many"
```

Cons-patterns peel one element off any sequence — list, string, or set:

```ff
sum = match
  h :: t -> h + sum(t),
  _      -> 0

sum([1, 2, 3, 4])        # 10
```

Destructuring also works in plain assignments, with `..rest` for the middle or
the tail:

```ff
[head, ..tail] = [1, 2, 3, 4]    # head = 1, tail = [2, 3, 4]
[x, y]         = [10, 20]
{"name": who}  = {"name": "ada", "age": 36}
```

Guards refine an arm:

```ff
sign = match
  n if n < 0 -> "neg",
  0          -> "zero",
  _          -> "pos"
```

## blocks and control flow

Parentheses with multiple statements form a block. The value of the last
expression is the value of the block; inner bindings don't leak.

```ff
area = (
  w = 4
  h = 5
  w * h
)                        # area = 20

abs = n => if n < 0 then -n else n
```

## laziness and ranges

`[a..b]`, `[a..=b]`, and `[a..]` are lazy ranges. Because `::` doesn't force
its tail, list combinators stream:

```ff
match map(x => x * 2, [0..])         # infinite range
  a :: b :: c :: _ -> [a, b, c]       # [0, 2, 4]

[0..] |> filter(x => x % 2 == 0) |> take 5    # [0, 2, 4, 6, 8]
```

## atoms

`:name` is an atom — a self-evaluating constant that compares equal only to
itself. Pairs well with lists and pattern matching for tagged-union style.

```ff
:ok                          # :ok
:ok == :ok                   # true
:ok == :error                # false

safe_div = (a, b) =>
  if b == 0 then [:error, "div0"] else [:ok, a / b]

handle = match
  [:ok, v]      -> v,
  [:error, msg] -> -1

handle(safe_div(10, 2))      # 5
handle(safe_div(10, 0))      # -1
```

Atoms work anywhere a value does — list/set elements, dict keys, and patterns.
Dicts keyed by atoms read back with `.:name`:

```ff
m = {:name: "ada", :age: 36}
m.:name                                    # "ada"

{:name: who} = m                           # who = "ada"
```

## custom operators

Any sequence of operator characters can be a user-defined infix; precedence is
OCaml-style, picked from the first character (`*`/`/`/`%` bind tighter than
`+`/`-`, which bind tighter than `=`/`<`/`>`, etc.). Prefix ops start with `?`
or `~`.

```ff
(<|>) = (x, y) => if x != default(x) then x else y
"" <|> "fallback"        # "fallback"

(~?) = x => default(x)
~?[1, 2, 3]              # []
```

Wrapping any operator in parens turns it into a normal value:

```ff
plus = (+)
[1, 2, 3] |> reduce(plus, 0)    # 6
```

## modules

`import "path.ff"` returns a module value containing whatever the file marked
`export`. As a bare statement (not the RHS of `=`), an import also splats those
names into the current scope.

```ff
# math.ff
square = x => x * x
cube   = x => x * x * x
export square, cube

# main.ff
import "math.ff"
square(7)                # 49

# or keep it namespaced
M = import "math.ff"
M.cube(3)                # 27
```

## running

```sh
cargo run                # REPL
cargo run -- path.ff     # run a file
cargo test               # run the test suite
```
