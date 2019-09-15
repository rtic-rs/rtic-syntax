# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [v0.3.4] - 2018-08-28

### Fixed

- The build with recent nightlies

## [v0.3.3] - 2018-04-28

### Fixed

- A compilation error on the latest nightly

## [v0.3.2] - 2018-04-23

### Changed

- Disabled the default features of the `either` dependency

## [v0.3.1] - 2018-04-23

### Changed

- Relax the version requirement of the `either` dependency

## [v0.3.0] - 2018-04-16

### Changed

- Updated the `syn` dependency to a recent release. Error and warning messages are now raised on
  parsing and checking issues.
- [breaking-change] Changed the parsing and checking API (structs).
- The checking pass now checks the specification to, for example, reject a specification where a
  task declares that it has access to a resource that wasn't declared in the top level `resources`
  field.

## [v0.2.1] - 2018-01-15

### Added

- An optional `init.resources` field

## [v0.2.0] - 2017-09-22

### Changed

- [breaking-change] The `Static.expr` field is now optional.

## v0.1.0 - 2017-07-28

- Initial release

[Unreleased]: https://github.com/rtfm-rs/rtfm-syntax/compare/v0.3.4...HEAD
[v0.3.4]: https://github.com/rtfm-rs/rtfm-syntax/compare/v0.3.3...v0.3.4
[v0.3.3]: https://github.com/rtfm-rs/rtfm-syntax/compare/v0.3.2...v0.3.3
[v0.3.2]: https://github.com/rtfm-rs/rtfm-syntax/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/rtfm-rs/rtfm-syntax/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/rtfm-rs/rtfm-syntax/compare/v0.2.1...v0.3.0
[v0.2.1]: https://github.com/rtfm-rs/rtfm-syntax/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/rtfm-rs/rtfm-syntax/compare/v0.1.0...v0.2.0
