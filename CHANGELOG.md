# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- CI changelog entry enforcer

### Changed

### Fixed

- Use swatinem rust-cache for GHA CI

## [v1.0.0] - 2021-12-25

### Added

- Allow annotating resources to activate special resource locking behaviour.
  - `#[lock_free]`, there might be several tasks with the same priority accessing
    the resource without critical section.
  - `#[task_local]`, there must be only one task, similar to a task local
    resource, but (optionally) set-up by init. This is similar to move.

- `peripherals` is now enabled (*true*) by default, you no longer need to give
  `#[app(..., peripherals = true))`. This is the common case, and if forgotten
  it results in an error which may be confusing to the user.

- Improved ergonomics allowing separation of task signatures to actual
  implementation in extern block `extern "Rust" { #[task(..)] fn t(..); }`.

### Changed

- Edition 2021

- [breaking-change] Remove `Location`, changes analysis struct contents:
  `shared_resource_locations` -> `shared_resources`. Same for local.

- [breaking-change] "Resource handling take 2" implemented

- [breaking-change] Move of dispatchers (interrupts) from `extern` to app arguments.
  `app(..., dispatchers = [SSI0,...])`
  This should also work for ram functions and other attributes, see `examples/ramfunc.rs`.

- [breaking-change] Rework whole spawn/schedule, support `foo::spawn( ... )`,
  `foo::schedule( ... )`.

- [breaking-change] `struct Resources` changed to attribute `#[resources]` on a struct.

- [breaking-change] Mod over const, instead of `const APP: () = {` use `mod app {`.

- [breaking-change] Init function always return `LateResources` for a symmetric API.

- Multi-core support was removed to reduce overall complexity.

- CI Changed from Travis to GitHub Actions.

- [breaking-change] rtfm-syntax is now known as rtic-syntax.

## [v0.4.0] - 2019-11-14

### Added

- Several tests were ported from the cortex-m-rtic repository (pre-v0.5.0 state)

### Changed

- [breaking-change] syn, quote and proc-macro2 dependencies were bumped to
  version 1.0.

- [breaking-change] syntax was changed to match RFCs approved for cortex-m-rtic
  v0.5.0 release. The field of most structures changed as well as their APIs.

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

[Unreleased]: https://github.com/rtic-rs/rtic-syntax/compare/v1.0.0...HEAD
[v1.0.0]: https://github.com/rtic-rs/rtic-syntax/compare/v0.4.0...v1.0.0
[v0.4.0]: https://github.com/rtic-rs/rtic-syntax/compare/v0.3.4...v0.4.0
[v0.3.4]: https://github.com/rtic-rs/rtic-syntax/compare/v0.3.3...v0.3.4
[v0.3.3]: https://github.com/rtic-rs/rtic-syntax/compare/v0.3.2...v0.3.3
[v0.3.2]: https://github.com/rtic-rs/rtic-syntax/compare/v0.3.1...v0.3.2
[v0.3.1]: https://github.com/rtic-rs/rtic-syntax/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/rtic-rs/rtic-syntax/compare/v0.2.1...v0.3.0
[v0.2.1]: https://github.com/rtic-rs/rtic-syntax/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/rtic-rs/rtic-syntax/compare/v0.1.0...v0.2.0
