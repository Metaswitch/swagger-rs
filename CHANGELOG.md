# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Allow ContextWrapper to be cloned.

### Changed

### Removed

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

[Unreleased]: https://github.com/Metaswitch/swagger-rs/compare/1.0.2...HEAD
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
