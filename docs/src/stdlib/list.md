# List

Native list operations. Import with:

```ff
List = import "List"
```

All multi-argument functions take the list **last** for pipe-friendly
composition.

## Accessors

| Function              | Description                                        |
|-----------------------|----------------------------------------------------|
| `List.len(xs)`        | Element count                                      |
| `List.head(xs)`       | First element, or `()` if empty                    |
| `List.tail(xs)`       | All but the first, or `()` if empty                |
| `List.last(xs)`       | Last element, or `()` if empty                     |
| `List.nth(n, xs)`     | Element at index `n` (0-based), or `()` out of range |
| `List.is_empty(xs)`   | Boolean                                            |

```ff
xs = [10, 20, 30, 40]
List.len(xs)            # 4
List.head(xs)           # 10
List.last(xs)           # 40
List.nth(2, xs)         # 30
```

## Search

| Function                  | Description                                |
|---------------------------|--------------------------------------------|
| `List.contains(v, xs)`    | `true` if any element equals `v`           |
| `List.index_of(v, xs)`    | Index of first match, or `()` if absent    |

```ff
List.contains(20, xs)      # true
List.index_of(30, xs)      # 2
```

## Transforms

| Function                   | Description                                |
|----------------------------|--------------------------------------------|
| `List.reverse(xs)`         | Reversed list                              |
| `List.concat(a, b)`        | `a` followed by `b`                        |
| `List.flatten(xss)`        | One level of flattening                    |
| `List.zip(a, b)`           | List of `[a_i, b_i]` pairs                 |
| `List.drop(n, xs)`         | Drop the first `n` elements                |
| `List.take(n, xs)`         | Take the first `n` elements                |

```ff
List.concat([1, 2], [3, 4])           # [1, 2, 3, 4]
List.flatten([[1, 2], [3, 4]])        # [1, 2, 3, 4]
List.zip([1, 2, 3], ["a", "b", "c"])  # [[1, "a"], [2, "b"], [3, "c"]]
List.drop(2, [10, 20, 30, 40])        # [30, 40]
```

## Note on `take`

The prelude `take` (from `stdlib.ff`) and `List.take` differ subtly:

- Prelude `take` walks the spine via cons recursion and works on lazy
  ranges and infinite streams.
- `List.take` materializes its argument into a Rust `Vec` first; use it
  on already-finite lists when you don't want to recurse in ff.

Prefer the prelude version for pipelines over `[0..]`-style sources.

## Streaming

For the streaming combinators (`map`, `filter`, `reduce`, `fold`,
`rev`), see [Prelude](./prelude.md) — those live above `List` since
they're polymorphic across lists, ranges, strings, and cons spines.
