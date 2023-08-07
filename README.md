# RustSASA

This crate is currently in development and the API is unstable and may change. The package is not currently listed on crates.io but will be in the future.

Rust FFI bindings for `freesasa` C-API.

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

Add this package using git to your Cargo.toml file. In the future it will be possible to simply use crates.io - but this has not been done yet.
