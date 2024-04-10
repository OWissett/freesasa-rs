# RustSASA

High-level Rust FFI bindings for `freesasa` C-API. Currently the library provides all basic functionality of `freesasa` C-API.

If you require lower level bindings, you can use `freesasa_sys` crate directly - this is not recommended unless you are sure what you are doing, as you require the use of `unsafe` Rust code and manual memory management of `C` objects.

It is a work in progress and is not yet feature complete, as such, some functions may not be available and the API may change (although I will try to keep it as stable as possible).

## Requirements

Requires `freesasa_sys` which is built from source. As such following build tools are required:

### Linux (Debian)

All of the following are availible as `apt` packages:

* `build-essential`
* `make`
* `autoconf`
* `libc++-dev`
* `libc++abi-dev`

### MacOS

Availible as `brew` packages:

* XCode should be installed for C/C++ compilers and libraries
* `automake`

At the point of testing, I have lots of other tools installed on my mac, so
it is hard for me to know if there are any other dependencies.

Let me know if you have any errors when building this on a Mac (or update this
list).

## Install

Add the following to your `Cargo.toml`:

```toml
[dependencies]
rustsasa = "0.1.1"
```
