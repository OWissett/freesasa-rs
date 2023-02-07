# RustSASA

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

* XCode should needs to be installed for C/C++ compilers and libraries
* `automake`

At the point of testing, I have lots of other tools installed on my mac, so
it is hard for me to know if there are any other dependencies.

Let me know if you have any errors when building this on a Mac (or update this
list).

## Install

As this package is not publicly availible, either add it to cargo.toml with
a local path to it as a git submodule, or add the repo to the cargo.toml file (this will
need Cargo to have the ability to use your credentials).
