//! This crate aims to provides a safe interface to the
//! [freesasa](https://freesasa.github.io/doxygen/index.html) C library, developed by
//! [Simon Mittinatten](https://github.com/mittinatten) \[1\]. FreeSASA allows you to
//! calculate the solvent accessible surface area (SASA) of a protein from its atomic
//! coordinates. The library is written in C, and is available under the MIT license.
//!
//! Additionally, I am for this crate to provide additional functionality, such as
//! providing improved dynamic structure building and to provide utilities for
//! finding differences between multiple structures. I am also aiming for this
//! library to be thread-safe, however, this is ultimately dependent on the
//! underlying C library (which the author has stated 'should' be thread-safe, however
//! this has not been tested).
//!
//! It is possible to expose the raw FFI bindings to the C library. This may be useful
//! if you want to use the C library directly, or if you want to add your own C code.
//!
//! ## Installation
//! The library is currently in a very early stage of development, and is not yet
//! ready for use. However, if you would like to try it out, you can do so by
//! adding the git repository as a submodule to your project, and then adding
//! the following to your `Cargo.toml` file:
//! ```toml
//! [dependencies]
//! rustsasa = { path = "path/to/rustsasa" }
//! ```
//!
//! To build the library, you will need to have the `freesasa` C library installed as a
//! static library. See the [freesasa repo](https://github.com/mittinatten/freesasa/) for
//! details on how to do this.
//!
//! The build script will look for the `freesasa` static library in /usr/local/lib. However,
//! if you have installed the library in a different location, you can set the `FREESASA_STATIC_LIB`
//! environment variable to the directory containing the library. For example, if you have installed
//! the library in `~/software/lib`, you can set the environment variable as follows:
//! ```bash
//! # Add the following to your .bashrc or .zshrc (or similar)
//! export FREESASA_STATIC_LIB=~/software/lib
//!
//! # Or, set it for just this command
//! FREESSASA_STATIC_LIB=~/software/lib cargo build
//! ```
//!
//! Alternatively, you can use the dockerfile (`Dockerfile.dev`) to set up a dev-container. This
//! dockerfile will install the `freesasa` library in `/usr/local/lib`, and set-up a Rust development
//! environment. The dockerfile can be found in the root of the repository.
//!
//! ## Example
//! ```rust
//! use rustsasa::{structure::Structure, FreesasaVerbosity, set_fs_verbosity};
//!
//! // Set the verbosity of the freesasa library
//! set_fs_verbosity(FreesasaVerbosity::Normal);
//!
//! // Create a new structure from a PDB file
//! let structure = Structure::from_pdb("tests/data/1ubq.pdb").unwrap();
//!
//! // Calculate the SASA for the structure
//! let result = structure.calculate_sasa().unwrap();
//!
//! // Print the SASA for each atom
//! let atom_sasa = result.atom_sasa();
//! for (i, sasa) in atom_sasa.iter().enumerate() {
//!    println!("Atom {}: {:.2}", i, sasa);
//! }
//!
//!
//! ```
//!
//! ## References
//! \[1\] Simon Mitternacht (2016) FreeSASA: An open source C library for solvent accessible surface area calculations. F1000Research 5:189. (doi: [10.12688/f1000research.7931.1](https://f1000research.com/articles/5-189))
#[macro_use]
extern crate log;

// To expose the raw FFI bindings, compile with `RUSTFLAGS="--cfg expose_ffi"`
#[cfg(expose_ffi)]
pub mod freesasa_ffi;

#[cfg(not(expose_ffi))]
mod freesasa_ffi;

pub mod classifier;
pub mod node;
pub mod result;
pub mod selection;
pub mod structure;
mod utils;

// Bring the needed freesasa functions into scope
use freesasa_ffi::{
    freesasa_set_verbosity, freesasa_verbosity_FREESASA_V_DEBUG,
    freesasa_verbosity_FREESASA_V_NORMAL,
    freesasa_verbosity_FREESASA_V_NOWARNINGS,
    freesasa_verbosity_FREESASA_V_SILENT,
};

#[derive(Debug, Clone, Copy)]
pub enum FreesasaVerbosity {
    Debug,
    Normal,
    NoWarnings,
    Silent,
}

pub fn set_fs_verbosity(verbosity: FreesasaVerbosity) {
    let verbosity = match verbosity {
        FreesasaVerbosity::Debug => freesasa_verbosity_FREESASA_V_DEBUG,
        FreesasaVerbosity::Normal => {
            freesasa_verbosity_FREESASA_V_NORMAL
        }
        FreesasaVerbosity::NoWarnings => {
            freesasa_verbosity_FREESASA_V_NOWARNINGS
        }
        FreesasaVerbosity::Silent => {
            freesasa_verbosity_FREESASA_V_SILENT
        }
    };

    debug!("Setting freesasa verbosity to {:?}", verbosity);

    unsafe {
        freesasa_set_verbosity(verbosity);
    }
}
