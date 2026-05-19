# String

String operations. Import with:

```ff
String = import "String"
```

All multi-argument functions take the string **last** for pipe-friendly
composition.

## Inspection

| Function                       | Description                              |
|--------------------------------|------------------------------------------|
| `String.len(s)`                | Character count (Unicode-aware)          |
| `String.contains(needle, s)`   | `true` if `s` contains `needle`          |
| `String.starts_with(p, s)`     | Boolean                                  |
| `String.ends_with(suf, s)`     | Boolean                                  |
| `String.index_of(needle, s)`   | Character index of first match, or `()`  |

```ff
String.len("héllo")                       # 5
String.contains("ell", "hello")           # true
String.index_of("l", "hello")             # 2
```

## Case and trimming

| Function                  | Description                |
|---------------------------|----------------------------|
| `String.upper(s)`         | Uppercase                  |
| `String.lower(s)`         | Lowercase                  |
| `String.trim(s)`          | Trim whitespace both ends  |
| `String.trim_start(s)`    | Trim leading whitespace    |
| `String.trim_end(s)`      | Trim trailing whitespace   |

## Modify

| Function                            | Description                          |
|-------------------------------------|--------------------------------------|
| `String.replace(from, to, s)`       | Replace all occurrences              |
| `String.repeat(n, s)`               | `s` repeated `n` times               |
| `String.reverse(s)`                 | Reversed by character                |
| `String.slice(start, end, s)`       | Char-position slice, bounds-clamped  |

```ff
String.replace("o", "0", "foobar")        # "f00bar"
String.slice(1, 4, "hello")               # "ell"
```

## Split and join

| Function                  | Description                                       |
|---------------------------|---------------------------------------------------|
| `String.split(sep, s)`    | List of parts                                     |
| `String.lines(s)`         | List of lines (split on `\n`)                     |
| `String.chars(s)`         | List of single-character strings                  |
| `String.join(sep, parts)` | Join a list of strings with `sep`                 |

```ff
String.split(",", "a,b,c")                # ["a", "b", "c"]
String.join("-", ["a", "b", "c"])         # "a-b-c"
```

## Parsing

| Function                  | Description                                       |
|---------------------------|---------------------------------------------------|
| `String.parse_int(s)`     | `[:ok, n]` or `[:error, msg]`                     |
| `String.parse_float(s)`   | `[:ok, x]` or `[:error, msg]`                     |

```ff
String.parse_int("  42 ")                 # [:ok, 42]
String.parse_int("nope")                  # [:error, "invalid digit found in string"]
```

`parse_float` returns a finite-precision floating-point value as
encoded — note that arithmetic in ff stays exact, so the value
materializes back as a rational on use.

## Concatenation

Strings concatenate with `+`. `::` between two strings is also
concatenation:

```ff
"foo" + "bar"               # "foobar"
"a" :: "bc"                 # "abc"
```

Cons-patterns peel one character off the front of a string, which is
the basis for character-by-character processing without an explicit
index.
