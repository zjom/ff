
# Change Log
all notable changes to this project will be documented in this file.
 
the format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).
 
## [0.1.4] - 2025-05-21
 
improvements to prelude/ stdlib
 
### Added
- `take_while`
  accepts a predicate and a consable
 
### Changed
- **BREAKING** `IO.stdin().lines()` includes line sep (`\n`, `\r\n`)
  behaviour matches rust's `std::io::BufReader::lines`
 
### Fixed
