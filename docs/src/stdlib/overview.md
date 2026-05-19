# Standard Library Overview

The standard library is split into the **prelude** — values bound into
every program automatically — and a set of **built-in modules** you
opt into with `import`.

## Prelude

The prelude is installed before your code runs. It provides:

- The collection combinators `map`, `filter`, `reduce`, `fold`, `take`,
  `rev`.
- The pipe and composition operators `(|>)`, `(<<)`, `(>>)`.
- The control helper `assert` and the diagnostic `panic`.
- The introspection functions `typeof` and `default`.
- The I/O entry points `print` and `println`.
- The two most common modules already bound: `Object` and `Actor`.

See [Prelude](./prelude.md).

## Built-in modules

These are not files — they're native modules wired into the
interpreter, available via `import "Name"`:

| Module    | Purpose                                          |
|-----------|--------------------------------------------------|
| [`List`](./list.md)     | Operations on lists                        |
| [`Object`](./object.md) | Operations on objects                       |
| [`String`](./string.md) | Operations on strings                       |
| [`Io`](./io.md)         | stdin, stdout, stderr handles               |
| [`Fs`](./fs.md)         | File and directory access                   |
| [`Env`](./env.md)       | Process environment and CLI args            |
| [`Http`](./http.md)     | Blocking HTTP client                        |
| [`Json`](./json.md)     | JSON parse and stringify                    |
| [`Actor`](./actor.md)   | Actor primitives                            |

`Object` and `Actor` are pre-imported by the prelude. Everything else
takes one line:

```ff
String = import "String"
List   = import "List"
Fs     = import "Fs"
Json   = import "Json"
```

## Argument convention

Multi-argument native functions in the prelude and built-in modules
take the **data argument last**, so calls compose under `|>`:

```ff
xs
  |> filter(p)
  |> map(f)
  |> List.take(5)
  |> reduce((a, b) => a + b, 0)

config
  |> Object.put("retries", 3)
  |> Object.put("timeout", 30)
```

Whenever you're not sure, the data argument is the last one. The same
shape extends to the pipe-friendly String functions:
`String.replace(from, to, s)`, `String.split(sep, s)`, …
