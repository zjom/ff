
# Change Log
all notable changes to this project will be documented in this file.
 
the format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [0.1.6] - 2025-05-21

### Added
- String escape sequences:
  - `\n`, `\t`, `\r`, `\0`, `\\`
- `String.format(template, args)`
  Rust-`format!`-style interpolation. `{}` consumes the next arg, `{N}` picks by
  index, `{:?}` / `{N:?}` quote strings (and otherwise match the default value
  display). `{{` and `}}` are literal braces. Args are a list (or any
  cons-spine), so `args |> String.format("...")` composes naturally.

### Changed
- **BREAKING** Built-in infix operators (`+`, `-`, `*`, `/`, `%`, `**`, `==`,
  `!=`, `<`, `<=`, `>`, `>=`, `~`, `!~`) are now ordinary variables bound in
  the prelude — same model as user-defined operators. `1 + 2` desugars to
  `(+)(1)(2)`, so you can write `inc = (+)(1)`, `plus = (+)`, or shadow them
  with `(+) = (a, b) => ...`. `&&`, `||`, and `::` keep their special AST
  forms because they have non-strict semantics regular functions can't
  reproduce.
- **BREAKING** `+` is number-only. String concatenation lives on `::`
  (which already supported it), e.g. `"foo" :: "bar"` → `"foobar"`.
- **BREAKING** `^` removed as a power-operator alias. Use `**`.

### Fixed
- Dot access now binds tighter than juxtaposition, so `map String.trim` parses
  as `map(String.trim)` instead of `(map String).trim`. Lets module members
  flow through `|>` without parens: `xs |> map String.trim`.


## [0.1.5] - 2025-05-21

improvements to stdlib `String` module

### Added
- `String.split_at(i, s)`
  splits the string `s` at position `i` and returns the pair.
  if `i` is oob, returns `[s, ""]`

 
## [0.1.4] - 2025-05-21
 
improvements to prelude/ stdlib
 
### Added
- `take_while`
  accepts a predicate and a consable
 
### Changed
- **BREAKING** `IO.stdin().lines()` includes line sep (`\n`, `\r\n`)
  behaviour matches rust's `std::io::BufReader::lines`
 
### Fixed
