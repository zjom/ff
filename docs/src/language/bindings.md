# Bindings and Scope

A binding introduces a name and assigns it a value. The left-hand side
is a [pattern](./patterns.md), not just a bare identifier:

```ff
x = 1
[a, b]            = [10, 20]
[head, ..tail]    = [1, 2, 3, 4]
{:lang: lang}     = {:lang: "ff"}
```

Bindings are **immutable within a scope**. Reassigning the same name in
the same scope is a parse-time error. To replace a value, introduce a
new scope (a block) or shadow it in a nested scope.

## Blocks

A parenthesized group of statements is a *block*. The value of the
block is the value of its last expression; inner bindings are local:

```ff
area = (
  w = 4
  h = 5
  w * h
)                       # area = 20, w and h are not in scope here
```

To distinguish a block from a parenthesized expression, a block must
contain either:

- a single **assignment** (`(x = 1)`), or
- two or more statements separated by `;` or newlines.

So `(x)` is just a parenthesized `x`, while `(x = 1; x + 2)` is a block
that evaluates to `3`.

A block whose last statement is an assignment evaluates to `()`.

## Lexical scope

Functions capture their defining scope. Inner bindings shadow outer ones
but don't mutate them:

```ff
x = 1
f = () => (
  x = 2
  x
)
f()             # 2
x               # 1
```

## Top-level

The top level of a file (or REPL session) is a scope of its own. All
top-level bindings are visible to subsequent statements, and any value
marked with `export` becomes part of the module's public API.
