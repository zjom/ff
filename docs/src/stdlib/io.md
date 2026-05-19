# Io

Handles for standard streams. Import with:

```ff
Io = import "Io"
```

`Io` exposes three functions that return handles, each itself an
object with methods.

## `Io.stdin()`

Returns an object with two methods:

| Method               | Description                                          |
|----------------------|------------------------------------------------------|
| `stdin.lines()`      | `[:ok, stream]` of strings, one per line (lazy)      |
| `stdin.bytes()`      | `[:ok, stream]` of byte values (lazy)                |

Both produce lazy cons spines that terminate on EOF.

```ff
[:ok, lines] = Io.stdin().lines()

lines
  |> map(s => "> " + s)
  |> reduce((a, b) => a + "\n" + b, "")
  |> println
```

## `Io.stdout()` and `Io.stderr()`

Both return an object with `write`, `writeln`, and `flush`:

```ff
out = Io.stdout()
out.write("no newline")
out.writeln("with newline")
out.flush()
```

For typical use, the prelude's `print` / `println` write to stdout and
are easier to reach for.

## Note

In the current implementation, `Io.stdout()` also writes to stderr
internally — that quirk may change. For predictable script output,
prefer `println` from the prelude.
