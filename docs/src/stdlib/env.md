# Env

Process environment and CLI args. Import with:

```ff
Env = import "Env"
```

## Functions

| Function           | Description                                                |
|--------------------|------------------------------------------------------------|
| `Env.args()`       | List of CLI args, starting with the program name           |
| `Env.vars()`       | Object of all environment variables                         |
| `Env.var(name)`    | `[:ok, value]` / `[:error, msg]` for a single var          |

```ff
match Env.args()
  [_, path, ..] -> println("running on " + path),
  _ -> println("usage: prog <path>")

match Env.var("HOME")
  [:ok, h]      -> println(h),
  [:error, _]   -> println("HOME unset")
```

`Env.var` accepts either a string `"PATH"` or an atom-as-string
`":PATH"` (the leading `:` is stripped).
