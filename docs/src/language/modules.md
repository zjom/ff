# Modules

A module is just an object. `import "path.ff"` loads a file and returns
an object containing whatever names that file marked with `export`.
There's no language-level concept of a "module" beyond this.

## Exporting

```ff
# math.ff
square = x => x * x
cube   = x => x * x * x
helper = x => x + 1     # private — not exported

export square, cube
```

You can also `export *` to snapshot every binding visible at the
`export` statement:

```ff
# bag.ff
foo = 1
bar = 2
export *
```

Operators are exported by name in parens:

```ff
(<|>) = (x, y) => if x != default(x) then x else y
export (<|>)
```

## Importing

`import` is an expression. When used as the right-hand side of an
assignment, it returns the module object:

```ff
M = import "math.ff"
M.square(7)             # 49
M.cube(3)               # 27
```

Used as a **bare statement** (not the RHS of `=`), it *splats* the
module's exports into the current scope:

```ff
import "math.ff"
square(7)               # 49
```

Use whichever form matches the situation: bare for ergonomics, named
for namespacing.

## Selective import via destructuring

Because a module is an object, you can pull names out with an object
pattern (the shorthand form):

```ff
{square, cube} = import "math.ff"
square(7)               # 49
```

## Resolution

Paths are resolved relative to the importing file. The interpreter
also recognises a set of built-in module names that don't refer to
files at all: `Object`, `Actor`, `List`, `String`, `Io`, `Fs`, `Env`,
`Http`, `Json`. These are documented in the [Standard
Library](../stdlib/overview.md) section. The prelude already binds
`Object` and `Actor` for you, so most scripts can use them without an
explicit import.

## Module evaluation

Each `import` evaluates the file in a fresh child scope. Top-level
statements run in order, and only `export`ed names end up on the
returned object. The interpreter caches nothing — a re-import re-runs
the file. Keep top-level work to pure definitions; put side-effecting
setup in a function the importer can call explicitly.
