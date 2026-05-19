# Atoms

`:name` is an **atom** — a self-evaluating constant that compares equal
only to itself. Atoms cost nothing to create, have no payload, and are
the idiomatic way to tag union variants and small enumerations.

```ff
:ok                          # :ok
:ok == :ok                   # true
:ok == :error                # false
```

## Where atoms show up

- **Tagged results** — `[:ok, value]` / `[:error, msg]` is the
  convention for fallible functions. See
  [Error Handling](./errors.md).
- **Object keys** — atom-keyed objects are read back with `.name`
  syntax, making them the "record" of ff: `{:name: "Ada"}.name`.
- **Actor messages** — `Actor.notify(pid, :reset)` or
  `Actor.request(pid, [:add, 5])`.
- **Patterns** — match arms on `:ok` / `:error` are the typical way to
  branch on a tagged result.

## Example: tagged unions

```ff
safe_div = (a, b) =>
  if b == 0
    then [:error, "div0"]
    else [:ok, a / b]

handle = match
  [:ok, v]      -> v,
  [:error, msg] -> -1

handle(safe_div(10, 2))      # 5
handle(safe_div(10, 0))      # -1
```

## Identifier rules

Atoms follow identifier rules after the `:` — they start with a letter
or underscore and continue with alphanumerics or underscores. There is
no `:"string"` form; if you need a non-identifier key, use a string
literal instead.

## typeof

```ff
typeof :ok           # :atom
```
