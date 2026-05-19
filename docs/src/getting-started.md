# Getting Started

## Running ff

ff ships as a single Rust binary. From the repo root:

```sh
cargo run                # REPL
cargo run -- path.ff     # run a file
cargo test               # run the test suite
```

The REPL evaluates one expression per line. Multi-line input is supported
inside parentheses, brackets, or braces — the REPL keeps reading until the
delimiters balance.

## Your first script

Create `hello.ff`:

```ff
greet = name => "Hello, " + name + "!"
println(greet("world"))
```

Then:

```sh
cargo run -- hello.ff
# Hello, world!
```

## File extension and layout

Source files use the `.ff` extension. Each file is its own module — see
[Modules](./language/modules.md) for how to split code across files and
expose values with `export`.

A file is a sequence of statements separated by newlines or semicolons.
Statements are either assignments (`x = expr`), `export` declarations, or
bare expressions. The value of a bare expression at the top level is
printed by the REPL but discarded when running a file.

## Layout rules

Layout is forgiving. Inside `[]`, `{}`, and call argument lists, commas
and newlines are interchangeable separators:

```ff
xs = [
  1
  2, 3
  4,
]                                 # [1, 2, 3, 4]
```

Top-level statements are separated by newlines or `;`. Both work, and
you can mix them.
