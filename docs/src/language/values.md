# Values and Types

ff has a small set of built-in value kinds:

| Kind     | Examples                              |
|----------|---------------------------------------|
| Number   | `42`, `2.5`, `1 / 3`                  |
| String   | `"hello"`                             |
| Bool     | `true`, `false`                       |
| Unit     | `()`                                  |
| List     | `[1, "two", true]`                    |
| Object   | `{:lang: "ff"}`, `{"k": v}`           |
| Set      | `{1, 2, 3}`                           |
| Atom     | `:ok`, `:error`                       |
| Range    | `[0..]`, `[1..10]`, `[1..=10]`        |
| Function | `x => x + 1` (and curried natives)    |
| Pid      | returned from `Actor.spawn`           |

`typeof v` returns the type as an atom (`:number`, `:string`, `:list`, …).
`default(v)` returns the empty/zero value of the same type — useful for
generic seeds:

```ff
default(0)         # 0
default("hi")      # ""
default([1, 2])    # []
default({:k: 1})   # {}
default(true)      # false
```

## Numbers

Numbers are **exact rationals**. There's no separate `int` and `float` —
literals with a decimal point parse to a rational, not an IEEE-754
double. This means no precision drift:

```ff
0.1 + 0.2          # 3/10
1 / 3 + 1 / 3      # 2/3
2 ** 100           # 1267650600228229401496703205376
```

`**` and `^` both mean exponentiation (right-associative). `%` is
modulo. Division `/` produces a rational, not a truncated integer.

## Strings

Strings are double-quoted, immutable, and concatenated with `+`:

```ff
"abc" + "def"      # "abcdef"
```

The `~` operator tests substring containment; `!~` is its negation:

```ff
"hello world" ~ "world"     # true
"hello"       !~ "xyz"      # true
```

For more, see [String](../stdlib/string.md).

## Booleans and unit

`&&` and `||` short-circuit. `!` is logical not. The unit value `()` is
the result of statements that don't produce one (e.g. an assignment as
the last statement of a block).

```ff
true && !false     # true
true || undef      # true  — RHS not evaluated
```

## Lists

Lists are ordered, heterogeneous, and persistent:

```ff
[1, "two", true]
[10, 20, 30].0           # 10  — index by position
```

See [Collections](./collections.md) for the full story, and
[List](../stdlib/list.md) for the module of native operations.

## Objects

Objects are unordered maps from any value to any value. Atom-keyed entries
can be read with `.name`:

```ff
m = {:lang: "ff", :version: 0.1}
m.lang                              # "ff"

# arbitrary keys go through Object.get / Object.put
squares = {1: 1, 2: 4, 4: 16}
Object.get(2, squares)              # 4
```

## Sets

Sets are unordered, deduplicated collections:

```ff
{1, 2, 2, 3}        # {1, 2, 3}
```

`::` adds an element. Element membership and equality are structural.

## Atoms

`:name` is a self-evaluating constant — a tiny named value that compares
equal only to itself. Atoms are the idiomatic way to tag union variants,
return success/failure, or key common object fields. See
[Atoms](./atoms.md).

## Ranges

`[a..]`, `[a..b]`, `[a..=b]` are lazy ranges. They don't materialize
until consumed, so `[0..]` is a perfectly fine infinite sequence as long
as a downstream combinator stops pulling. See [Ranges and
Laziness](./ranges.md).
