# Operators

ff has a small set of built-in operators and lets you define your own.

## Built-in operators

### Arithmetic

| Op       | Meaning                | Precedence (tightest first) |
|----------|------------------------|-----------------------------|
| `**`, `^`| exponent (right-assoc) | 1                           |
| `*`, `/`, `%` | mul, div, mod     | 2                           |
| `+`, `-` | add, sub               | 3                           |

All arithmetic is on exact rationals. `/` never truncates.

### Comparison

`==`, `!=`, `<`, `<=`, `>`, `>=` work on numbers, strings, atoms, and
structurally on collections (`[1, 2] == [1, 2]`, `{1, 2, 3} == {3, 2, 1}`).

`~` tests substring containment for strings; `!~` is its negation.

### Logical

`&&`, `||`, `!`. `&&` and `||` short-circuit.

### Cons

`::` prepends an element to a sequence:

| Left           | Right          | Result                                   |
|----------------|----------------|------------------------------------------|
| any value `v`  | list `xs`      | list with `v` at the head                |
| string `a`     | string `b`     | concatenation (`a + b`)                  |
| any value `v`  | set `xs`       | set with `v` inserted                    |
| `[k, v]`       | object `o`     | object with `k → v` inserted             |

Cons is the canonical way to build sequences incrementally; combined
with [pattern matching](./patterns.md), it forms the basic shape of
list-processing recursion.

### Prefix

`-` (numeric negation) and `!` (logical not). Both are single-character
so they stack: `--5`, `!!true`.

## Operator precedence at a glance

From tightest to loosest:

1. function application (`f x`, `f(x)`), `.access`
2. prefix `-`, `!`
3. `**` / `^`
4. `*`, `/`, `%`
5. `+`, `-`
6. `^…`, `@…`  (custom "concat" ops)
7. `==`, `!=`, `<`, `<=`, `>`, `>=`, `~`, `!~` (and custom `<…`/`>…`/`=…`/`!…`/`$…`)
8. `&&` (and custom `&…`)
9. `||` (and custom `|…`)
10. `::` and the prelude pipe `|>`

Inside each level, associativity follows the leading character — `**`
right-associates; the rest left-associate.

## Custom operators

Any sequence of operator characters can be a user-defined infix.
Precedence is **OCaml-style**, picked from the first character: ops
starting with `*`/`/`/`%` bind tighter than ones starting with `+`/`-`,
which bind tighter than ones starting with `=`/`<`/`>`, and so on.

```ff
(<|>) = (x, y) => if x != default(x) then x else y
"" <|> "fallback"             # "fallback"
0  <|> 42                     # 42
```

Prefix operators start with `?` or `~` and must be at least two characters:

```ff
(~?) = x => default(x)
~?[1, 2, 3]                   # []
```

Operator characters: `! $ % & * + - / < = > ? @ ^ | ~`. The `:` character
is reserved (atoms, `::`), and `=>` / `->` are reserved (function and
match arrows).

### Using operators as values

Wrap any operator name in parens to use it like an ordinary identifier:

```ff
plus = (+)
[1, 2, 3] |> reduce(plus, 0)         # 6

cons = (::)
cons(0, [1, 2, 3])                   # [0, 1, 2, 3]
```

This works in both expressions and patterns, including for custom
operators. To export a custom operator from a module, use the
parenthesized form: `export (<|>)`.

## Pipes and composition (prelude)

Defined in the prelude as ordinary functions, not built-ins:

```ff
(|>) = (x, f) => f(x)
(<<) = (f, g) => x => f(g(x))
(>>) = (f, g) => x => g(f(x))
```

`|>` is the workhorse for chained transformations:

```ff
[1..=10]
  |> filter(x => x % 2 == 0)
  |> map(x => x * x)
  |> reduce((a, b) => a + b, 0)
```
