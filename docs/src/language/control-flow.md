# Control Flow

ff has just two control-flow forms — `if`/`else` and `match`. Everything
else is built from function calls and blocks. There are no loops; use
recursion or lazy sequences.

## `if`/`then`/`else`

`if` is an expression — it always has both branches:

```ff
abs = n => if n < 0 then -n else n
```

`then` and `else` are required keywords. Newlines are allowed between
any of the tokens, so the same expression can be laid out across
multiple lines:

```ff
result = if cond
  then expensive_thing()
  else fallback()
```

Both arms must evaluate to *some* value; if you want one branch to do
nothing useful, return `()`.

## `match`

`match` dispatches on the shape of a value. See
[Pattern Matching](./patterns.md) for the full story. In brief:

```ff
classify = match
  []     -> "empty",
  [_]    -> "one",
  _ :: _ -> "many"
```

## Blocks

A parenthesized group of statements lets you sequence work and bind
intermediate values:

```ff
result = (
  a = compute()
  b = a + 1
  b * 2
)
```

See [Bindings and Scope](./bindings.md#blocks) for the rules.

## Looping

There is no `while` or `for`. Iteration is expressed with recursion or
with lazy combinators on sequences:

```ff
# explicit recursion
count_down = n =>
  if n <= 0
    then println("done")
    else (
      println(n)
      count_down(n - 1)
    )

# lazy pipeline
[1..=100]
  |> filter(x => x % 3 == 0)
  |> map(x => x * x)
  |> take 5
```

## Early returns

There is no `return` keyword. A function's value is the value of its
body. To short-circuit, wrap the body in `match` or `if` and choose the
result explicitly.

## Panicking

`panic msg` raises a runtime error with the given message. It does not
return — useful for unreachable cases or invariant violations.

```ff
checked = match
  [:ok, v]      -> v,
  [:error, msg] -> panic msg
```
