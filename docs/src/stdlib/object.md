# Object

Operations on objects. Pre-imported by the prelude as `Object`.

All multi-argument functions take the object **last**.

## Functions

| Function                  | Description                                    |
|---------------------------|------------------------------------------------|
| `Object.get(key, obj)`    | Value for `key`, or `()` if absent             |
| `Object.put(key, val, obj)` | New object with `key` → `val`                |
| `Object.keys(obj)`        | List of keys (unordered)                       |
| `Object.values(obj)`      | List of values (unordered)                     |

```ff
config = {"host": "localhost", "port": 8080}

Object.get("host", config)               # "localhost"
Object.put("port", 9090, config)         # {"host": "localhost", "port": 9090}
Object.keys(config)                      # ["host", "port"]   (order unspecified)
```

## Alternatives

- **Atom-keyed dot access**: `obj.name` reads the value for atom key
  `:name`. Faster to type than `Object.get(:name, obj)` when the key is
  literal.
- **Cons for insertion**: `["k", v] :: obj` is equivalent to
  `Object.put("k", v, obj)`. Handy in spines built from pairs.

## Building objects with `|>`

```ff
{}
  |> Object.put("host", "localhost")
  |> Object.put("port", 8080)
  |> Object.put("retries", 3)
```

## Membership and equality

Two objects are equal iff they have the same key/value pairs.
`Object.get` returns `()` for missing keys; if `()` is itself a valid
value you store, distinguish using `Object.keys` or a pattern with
`{:key: v, ..}`.
