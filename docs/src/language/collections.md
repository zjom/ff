# Collections

ff has three core collection types: **lists**, **objects**, and **sets**.
All three are immutable and persistent — updates produce new collections
that share structure with the original.

## Lists

Lists are ordered, heterogeneous, and indexed by position with `.N`:

```ff
[1, "two", true]
[10, 20, 30].0         # 10
[10, 20, 30].2         # 30
```

Building with cons:

```ff
0 :: [1, 2, 3]         # [0, 1, 2, 3]
```

Lists support all the usual functional operations through the prelude
(`map`, `filter`, `reduce`, `fold`, `take`, `rev`) and the `List`
module (`List.len`, `List.head`, `List.concat`, …). See
[List](../stdlib/list.md).

### Streaming with cons spines

`map` and friends use cons (`::`) rather than allocating an output list
eagerly, so they stream over lazy ranges and infinite sequences. You can
take a finite prefix of an infinite computation:

```ff
[0..]
  |> filter(x => x % 2 == 0)
  |> take 5                       # [0, 2, 4, 6, 8]
```

## Objects

Objects are unordered maps. Keys can be any value; the common case is
atom-keyed (record-like) and string-keyed (JSON-like).

```ff
person = {:name: "Ada", :age: 36}
config = {"host": "localhost", "port": 8080}
```

Atom-keyed entries can be read with `.name`:

```ff
person.name            # "Ada"
person.age             # 36
```

For arbitrary keys, use the `Object` module:

```ff
Object.get("host", config)         # "localhost"
Object.put("port", 9090, config)   # new object with port replaced
Object.keys(person)                # [:name, :age] (order unspecified)
```

Following the [prelude convention](./functions.md#pipes-and-composition),
multi-arg `Object` functions take the object **last** so they compose
under `|>`:

```ff
config
  |> Object.put("port", 9090)
  |> Object.put("retries", 3)
```

Cons works on objects when the left side is a `[key, value]` pair —
that's equivalent to `Object.put`:

```ff
["timeout", 30] :: config          # config with "timeout" → 30
```

### Object patterns

Destructure by key:

```ff
{:name: n, :age: a} = person
```

Missing keys fail the match. Use the shorthand `{name}` when the
binding name matches the key (handy for module imports):

```ff
{map, filter} = import "list_utils.ff"
```

## Sets

Sets are unordered collections of unique values. Duplicates are
collapsed at construction time:

```ff
{1, 2, 2, 3}             # {1, 2, 3}
```

Cons inserts an element (no-op if already present):

```ff
4 :: {1, 2, 3}           # {1, 2, 3, 4}
```

Equality is structural, regardless of insertion order: `{1, 2} == {2, 1}`.

## Equality

All three collection types use structural equality. Two lists are equal
if they have the same length and the same elements in order. Two
objects are equal if they have the same key/value pairs. Two sets are
equal if they have the same elements.

## Layout

Commas and newlines are interchangeable inside `[]` and `{}`, so the
same data can be laid out compactly or vertically:

```ff
short = [1, 2, 3]

tall = [
  1
  2
  3
]

mixed = [
  1, 2,
  3, 4,
]
```
