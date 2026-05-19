# Json

JSON parse and stringify. Import with:

```ff
Json = import "Json"
```

## Functions

| Function                   | Description                                       |
|----------------------------|---------------------------------------------------|
| `Json.parse(s)`            | `[:ok, value]` / `[:error, msg]`                  |
| `Json.stringify(v)`        | Compact JSON string                               |
| `Json.stringify_pretty(v)` | Indented JSON string                              |

```ff
[:ok, data] = Json.parse("{\"name\": \"Ada\", \"age\": 36}")
data.name                                      # "Ada"

Json.stringify({:ok: true, :n: 42})            # `{"ok":true,"n":42}`
```

## Mapping

JSON ↔ ff values map as you'd expect:

| JSON      | ff                                |
|-----------|-----------------------------------|
| `null`    | `()`                              |
| `true` / `false` | `true` / `false`           |
| number    | rational (exact)                  |
| string    | string, **unless** it starts with `:` and looks like an atom — then it round-trips as an atom |
| array     | list                              |
| object    | object (string-keyed)             |

Round-tripping is lossless for everything except: floats are kept as
rationals (no precision loss), and an emitted atom serializes as
`":name"` so it can come back as an atom.

`Json.stringify` will error if the value contains anything not
serializable to JSON (e.g. a function or a pid).
