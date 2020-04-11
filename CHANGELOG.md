# 0.2.0 (2020-04-03)

### Fixes

- Fixed docs.rs build [#93](https://github.com/microsoft/com-rs/pull/93)
- Support paths when defining super traits [#110](https://github.com/microsoft/com-rs/pull/110)
- Made interface pointers more correct [#125](https://github.com/microsoft/com-rs/pull/125)
- Fix multiple user generated fields not being generated [#132](https://github.com/microsoft/com-rs/pull/132)

### Added

- `Debug` for `IID` [#128](https://github.com/microsoft/com-rs/pull/128)
- Allow more interfaces [#135](https://github.com/microsoft/com-rs/pull/135)

### Changed

- Define IIDs as strings [#107](https://github.com/microsoft/com-rs/pull/107)
- Require unsafe for interface methods [#120](https://github.com/microsoft/com-rs/pull/120)
- Remove winapi as dependency [#122](https://github.com/microsoft/com-rs/pull/122)
- Interface{Ptr,Rc} => Com{Ptr,Rc} [#129](https://github.com/microsoft/com-rs/pull/129)
- Runtime is now done through stand alone functions [#136](https://github.com/microsoft/com-rs/pull/136)


# 0.1.0 (2019-10-01)

Initial release of the `com` crate. 

A one stop shop for all things related to [COM](https://docs.microsoft.com/en-us/windows/win32/com/component-object-model--com--portal) programming in Rust.

This library exposes various macros, structs and functions to the user for both producing and consuming COM components in an idiomatic manner.
