# Fs

Filesystem access. Import with:

```ff
Fs = import "Fs"
```

## Module functions

| Function              | Description                                          |
|-----------------------|------------------------------------------------------|
| `Fs.exists(path)`     | `[:ok, bool]` / `[:error, msg]`                      |
| `Fs.read_dir(path)`   | `[:ok, stream]` of entry objects (lazy) / `[:error, msg]` |
| `Fs.open(path)`       | A file handle object                                 |

```ff
[:ok, true] = Fs.exists("README.md")

[:ok, entries] = Fs.read_dir("./src")
entries
  |> filter(e => e.is_file)
  |> map(e => e.name)
  |> take 10
```

Each entry from `Fs.read_dir` is an object:

```ff
{
  :name: "main.rs",
  :path: "./src/main.rs",
  :is_file: true,
  :is_dir: false,
  :is_symlink: false
}
```

## File handles

`Fs.open(path)` returns an object representing the file:

```ff
{
  :path: "...",
  :write: fn,
  :append: fn,
  :read_all: fn,
  :lines: fn,
  :bytes: fn,
  :metadata: fn
}
```

| Method                | Description                                          |
|-----------------------|------------------------------------------------------|
| `file.read_all()`     | `[:ok, contents]` / `[:error, msg]`                  |
| `file.write(data)`    | Truncate-and-write; `[:ok, ()]` / `[:error, msg]`    |
| `file.append(data)`   | Append; `[:ok, ()]` / `[:error, msg]`                |
| `file.lines()`        | `[:ok, stream]` of lines (lazy) / `[:error, msg]`    |
| `file.bytes()`        | `[:ok, stream]` of bytes (lazy) / `[:error, msg]`    |
| `file.metadata()`     | `[:ok, {size, is_file, is_dir}]` / `[:error, msg]`   |

```ff
f = Fs.open("notes.md")
[:ok, text] = f.read_all()

# overwrite
f.write("new contents")

# stream lines
[:ok, lines] = f.lines()
lines |> take 5 |> println
```

`Fs.open` doesn't fail for a missing file — it returns a handle, and
the individual operations report their errors when called. Use
`Fs.exists` first if you want to branch on presence.
