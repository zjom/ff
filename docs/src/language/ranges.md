# Ranges and Laziness

Three range forms produce sequences without materializing a list:

| Syntax     | Meaning                          |
|------------|----------------------------------|
| `[a..b]`   | half-open: `a, a+1, …, b-1`      |
| `[a..=b]`  | inclusive: `a, a+1, …, b`        |
| `[a..]`    | open-ended: `a, a+1, …`          |

```ff
[0..5]              # 0, 1, 2, 3, 4
[1..=3]             # 1, 2, 3
[10..]              # infinite
```

Ranges are themselves values (`typeof [0..] == :range`) and behave like
sequences for the purpose of cons-patterns, `map`, `filter`, `take`,
`reduce`, and friends.

## Laziness

Both ranges and the cons (`::`) operator are **lazy in their tails**.
Prelude combinators like `map` and `filter` build cons spines whose
tails are thunks, so a pipeline only computes as much as a downstream
consumer demands.

```ff
[0..]                              # infinite
  |> filter(x => x % 2 == 0)
  |> map(x => x * x)
  |> take 5                        # [0, 4, 16, 36, 64]
```

`take n xs` walks `n` elements and stops; the rest of the spine is
never produced.

`reduce` forces the entire spine because it has to. So does
materializing into a finite collection (e.g. `List.len`). Don't reduce
or count an infinite stream.

## Matching ranges

Cons patterns work on ranges directly:

```ff
[first, second, ..rest] = [0..]
first                              # 0
second                             # 1
# rest is the lazy tail starting at 2

match [10..]
  a :: b :: c :: _ -> [a, b, c]    # [10, 11, 12]
```

## Why exact rationals matter here

Because numbers don't drift, ranges are deterministic even when stepped
through with `+ 1/3`-style increments — just write your own generator
with cons recursion if you need a non-integer step.
