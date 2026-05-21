
# Change Log
all notable changes to this project will be documented in this file.
 
the format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [UNPUBLISHED] - 2025-05-21

### Added
- String escape sequences:
  - `\n`, `\t`, `\r`, `\0`, `\\`
- `String.format(template, args)`
  Rust-`format!`-style interpolation. `{}` consumes the next arg, `{N}` picks by
  index, `{:?}` / `{N:?}` quote strings (and otherwise match the default value
  display). `{{` and `}}` are literal braces. Args are a list (or any
  cons-spine), so `args |> String.format("...")` composes naturally.

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
