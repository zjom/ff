# ff

A small expression-based language with pattern matching, curried functions, and
destructuring. Implemented in Rust on top of [pest](https://pest.rs).

## Running

```sh
cargo run        # start the REPL
```

REPL conventions: `>>` prompts for a new input, `..` for a continuation when the
buffer doesn't yet parse. A blank line while continuing shows the parse error
and clears the buffer. Ctrl-D exits.

## Tour

```
# comments start with `#` and run to end of line
x = 1 + 2 * 3            # => 7
greeting = "hello"
ok = true
```

### Numbers, strings, booleans

```
1        2.5        0.0      2**200
"hi"     "\" not yet supported"
true     false
```

numbers are arbitrary precision

Strings are plain text between `"`; there are currently no escape sequences.

### Arithmetic and comparison

| Operators                     | Notes                              |
|-------------------------------|------------------------------------|
| `+ - * / %`                   | numbers; `+` also concatenates strings |
| `** ^`                        | power, right-associative           |
| `== !=`                       | structural equality on any value   |
| `< <= > >=`                   | numbers and strings                |
| `&& \|\|`                     | short-circuiting; require bool operands |
| `! -`                         | unary not / negate                 |
| `~  !~`                       | substring containment on strings   |

### Collections

```
list  = [1, 2, 3]
tuple = (1, 2)            # () is the empty tuple; (x,) is a 1-tuple
dict  = {"name": "ada", "age": 36}
set   = {1, 2, 2, 3}      # deduplicates to {1, 2, 3}
```

### Dot access

```
list.0                    # list/tuple index — 1
dict.name                 # dict string-key lookup — "ada"
matrix = [[1, 2], [3, 4]]
matrix.0.1                # chained — 2
```

`.<digits>` only matches integers, so `xs.0.1` always reads as `(xs.0).1` rather
than as a single decimal index.

### Functions

Functions are introduced with `(params) => body`. Multi-parameter functions are
sugar for curried single-parameter functions, and multi-argument calls are sugar
for chained calls:

```
add = (x, y) => x + y     # same as (x) => (y) => x + y
add(3, 4)                 # => 7
add(3)(4)                 # => 7  — same call
inc = add(1)              # partial application
inc(10)                   # => 11

noargs = () => 42         # zero-arg functions are preserved
noargs()                  # => 42
```

For one-argument functions, the parens are optional on both sides:

```
inc = x => x + 1          # same as (x) => x + 1
inc 5                     # => 6  — same as inc(5)
add = x => y => x + y     # right-associative arrow; same as (x, y) => x + y
add 3 4                   # => 7  — same as add(3)(4)
```

Juxtaposition application binds tighter than any operator: `inc 5 + 2` is
`(inc 5) + 2`. To pass a negative literal, use parens: `f (-1)` — bare `f -1`
parses as `f - 1`.

Recursion works because the closure captures a shared handle to the scope it
was defined in:

```
fact = (n) => if n == 0 then 1 else n * fact(n - 1)
fact(6)                   # => 720
```

### Control flow

```
if x > 0 then x else -x   # if-then-else is always an expression
```

`match` evaluates a value against arms separated by commas. First match wins;
no match raises a runtime error. A trailing comma is allowed.

```
match value
  0          -> "zero",
  1          -> "one",
  n          -> n * 100   # bare ident binds
```

An arm can carry an `if` guard that runs after the pattern binds; a false
guard falls through to the next arm:

```
match n
  n if n < 0  -> "neg",
  0           -> "zero",
  n if n < 10 -> "small",
  _           -> "big"
```

If the scrutinee is omitted, the `match` evaluates to a one-argument function
whose argument becomes the scrutinee — useful for assigning a matcher to a
name:

```
describe = match
  0          -> "zero",
  1          -> "one",
  n          -> "many"
describe 0                # => "zero"
describe(42)              # => "many"

fact = match              # recursive match-as-function
  0 -> 1,
  n -> n * fact(n - 1)
fact 6                    # => 720
```

### Patterns

Patterns appear on the left of `=` (destructuring assignment) and in `match`
arms. The same forms are accepted in both:

```
_                         # wildcard, matches anything
x                         # ident, binds
42  3.14  "lit"  true     # literal — matches by equality

[a, b, c]                 # list of exactly three
[a, b, ..rest]            # rest captures remaining as a list
[head, .., last]          # rest can be in the middle
[a, b, ..]                # rest without a name discards

(x, y)                    # tuple of two
{"key": v}                # dict — required keys (extras allowed in value)
{1, 2}                    # set — each sub-pattern must match some element
```

Destructuring on assignment:

```
[first, ..rest] = [10, 20, 30, 40]
first                     # => 10
rest                      # => [20, 30, 40]

(x, y) = (1, 2)
{"name": who} = {"name": "ff", "age": 36}
who                       # => "ff"
```

Pattern matching on values:

```
describe = (xs) -> match xs
  []          -> "empty",
  [x]         -> "one element",
  [x, ..rest] -> "many"
```

## Project layout

```
src/
  ff.pest         # grammar (pest)
  ast.rs          # AST types
  parser.rs       # pest → AST, Pratt-parsed expressions, currying desugar
  interpreter.rs  # tree-walking evaluator
  repl.rs         # REPL loop
  main.rs         # binary entry — calls repl::run()
  lib.rs          # module index
```
