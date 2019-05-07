# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added

### Changed

### Removed

## [3.1.0] - 2019-05-07
### Added
- Add `swagger::client::MakeService` trait

## [3.0.0] - 2019-03-08
### Changed
- Hyper 0.12 support.

  This creates large scale changes with corresponding renaming (e.g. `NewService` -> `MakeService`) and other fall out.

  Hyper Services don't have Request as a type parameters -  instead the body of the request / response are type parameters.

  As such context for requests, such as authorization data is kept in a `ContextualPayload` struct, instead of a tuple.

### Removed
- `AddContext` structs which we were previously deprecated are now removed.

## [2.0.2] - 2018-12-13
### Added
- Allow ContextWrapper to be cloned.

## [2.0.1] - 2018-11-12
### Changed
* Make compatible with clippy on stable (1.30.1)

## [2.0.0] - 2018-09-28

### Changed
- Added the `AddContextNewService` and `AddContextService` structs, and deprecated the old `AddContext` struct. One or other of the new structs should be a drop-in replacement for the `AddContext`, depending on whether it was being used as a `NewService` or `Service`.
- modified the `new_context_type` macro to only implement `Push`, `Pop` and `Has` for types explicitly passed to the macro. This is a breaking change, which should only require minor changes such as adding type annotations if the macro was used as recommended in the docs.

## [1.0.2] - 2018-07-23
### Added
- Added (non-HTTPS) support for Windows/MacOS/iOS

## [1.0.1] - 2018-05-24
### Added
- `SwaggerService` trait used by swagger-codegen middlewares.

## [1.0.0] - 2018-04-30
No changes. We now think we've got enough to declare this crate stable.

## [0.12.1] - 2018-04-27
### Added
- `RequestParser` trait for retrieving Swagger related info in middlewares.

### Changed
- Fixed `DropContext` to remove trait bounds on the type of context it can drop

## [0.12.0] - 2018-04-26
### Added
- `DropContext` to pass a raw (context-less) `hyper::Request` to a service.

## [0.11.0] - 2018-04-11
### Added
- `Has<T>`, `Pop<T>` and `Push<T>` traits for specifying requirements on context types in hyper services, and providing methods for manipulating them
- `new_context_type!` macro for defining structs that can be used to build concrete context types that implement `Has`, `Pop` and `Push`
- `make_context!` and `make_context_ty!` for conveniently creating contexts at value and type level

### Removed
- Old `Context` struct

### Changed
- Renamed `NoAuthentication` to `MiddlewareWrapper` and moved it to its own module.

## [0.10.0] - 2018-03-16
### Added
- Structs for combining multiple hyper services

## [0.9.0] - 2018-01-25
### Added
- Connector functions for instantiating easy-mode clients
- The ability to pass in a `slog::Logger` with Context

## [0.8.1] - 2017-12-20
### Changed
- Fix build error and clippy warning.

## [0.8.0] - 2017-12-15
### Added
- Asynchronous HTTP client/server support

### Removed
- Synchronous HTTP client/server support - if you're still using synchronous swagger-codegen, stay at 0.7.0

### Changed
- `AllowAllMiddleware` (an Iron middleware) has been replaced by `AllowAllAuthenticator` (a Hyper Service wrapper)

## [0.7.0] - 2017-10-02
### Added
- `ContextWrapper` - wraps an `Api` with a `Context`

## [0.6.0] - 2017-09-25
### Changed
- Authorization struct now has new field `issuer`.

## [0.5.0] - 2017-09-18
- Start of changelog.

[Unreleased]: https://github.com/Metaswitch/swagger-rs/compare/3.1.0...HEAD
[3.1.0]: https://github.com/Metaswitch/swagger-rs/compare/3.0.0...3.1.0
[3.0.0]: https://github.com/Metaswitch/swagger-rs/compare/2.0.2...3.0.0
[2.0.2]: https://github.com/Metaswitch/swagger-rs/compare/2.0.1...2.0.2
[2.0.1]: https://github.com/Metaswitch/swagger-rs/compare/2.0.0...2.0.1
[2.0.0]: https://github.com/Metaswitch/swagger-rs/compare/1.0.2...2.0.0
[1.0.2]: https://github.com/Metaswitch/swagger-rs/compare/1.0.1...1.0.2
[1.0.1]: https://github.com/Metaswitch/swagger-rs/compare/1.0.0...1.0.1
[1.0.0]: https://github.com/Metaswitch/swagger-rs/compare/0.12.1...1.0.0
[0.12.1]: https://github.com/Metaswitch/swagger-rs/compare/0.12.0...0.12.1
[0.12.0]: https://github.com/Metaswitch/swagger-rs/compare/0.11.0...0.12.0
[0.11.0]: https://github.com/Metaswitch/swagger-rs/compare/0.10.0...0.11.0
[0.10.0]: https://github.com/Metaswitch/swagger-rs/compare/0.9.0...0.10.0
[0.9.0]: https://github.com/Metaswitch/swagger-rs/compare/0.8.1...0.9.0
[0.8.1]: https://github.com/Metaswitch/swagger-rs/compare/0.8.0...0.8.1
[0.8.0]: https://github.com/Metaswitch/swagger-rs/compare/0.7.0...0.8.0
[0.7.0]: https://github.com/Metaswitch/swagger-rs/compare/0.6.0...0.7.0
[0.6.0]: https://github.com/Metaswitch/swagger-rs/compare/0.5.0...0.6.0
[0.5.0]: https://github.com/Metaswitch/swagger-rs/compare/0.4.0...0.5.0
