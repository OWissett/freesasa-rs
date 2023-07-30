//! This crate aims to provides a safe interface to the
//! [freesasa](https://freesasa.github.io/doxygen/index.html) C library, developed by
//! [Simon Mitternacht](https://github.com/mittinatten) \[1\]. FreeSASA allows you to
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
//!
//! ## Example
//! ```rust
//! use freesasa_rs::{structure::Structure, FreesasaVerbosity, set_fs_verbosity};
//!
//! // Set the verbosity of the freesasa library
//! set_fs_verbosity(FreesasaVerbosity::Info);
//!
//! // Create a new structure from a PDB file
//! let structure = Structure::from_path("./data/single_chain.pdb", None).unwrap();
//!
//! // Calculate the SASA for the structure
//! let result = structure.calculate_sasa().unwrap();
//!
//! // Print the SASA for each atom
//! let atom_sasa = result.atom_sasa();
//! for (i, sasa) in atom_sasa.iter().enumerate() {
//!    println!("Atom {}: {:.2}", i, sasa);
//! }
//! ```
//!
//! ## References
//! \[1\] Simon Mitternacht (2016) FreeSASA: An open source C library for solvent accessible
//! surface area calculations. F1000Research 5:189.
//! (doi: [10.12688/f1000research.7931.1](https://f1000research.com/articles/5-189))
#[macro_use]
extern crate log;

pub mod classifier;
pub mod result;
pub mod selection;
pub mod structure;
pub mod uids;
mod utils;

// Bring the needed freesasa functions into scope
use freesasa_sys::{
    freesasa_set_err_out, freesasa_set_verbosity,
    freesasa_verbosity_FREESASA_V_DEBUG,
    freesasa_verbosity_FREESASA_V_NORMAL,
    freesasa_verbosity_FREESASA_V_NOWARNINGS,
    freesasa_verbosity_FREESASA_V_SILENT,
};

#[derive(Debug, Clone, Copy)]
pub enum FreesasaVerbosity {
    Debug,
    Info,
    Error,
    Silent,
}

/// Sets the verbosity of the freesasa library.
pub fn set_verbosity(verbosity: FreesasaVerbosity) {
    let verbosity = match verbosity {
        FreesasaVerbosity::Debug => freesasa_verbosity_FREESASA_V_DEBUG,
        FreesasaVerbosity::Info => freesasa_verbosity_FREESASA_V_NORMAL,
        FreesasaVerbosity::Error => {
            freesasa_verbosity_FREESASA_V_NOWARNINGS
        }
        FreesasaVerbosity::Silent => {
            freesasa_verbosity_FREESASA_V_SILENT
        }
    };

    debug!("Setting freesasa verbosity to {:?}", verbosity);

    // We should also use this function to set the verbosity of
    // the rust logging crate

    unsafe {
        freesasa_set_verbosity(verbosity);
    }
}

pub fn set_err_out(
    file: Option<&std::path::Path>,
) -> Result<(), &'static str> {
    if let Some(file) = file {
        debug!("Setting freesasa error output to {:?}", file);
        let file =
            std::ffi::CString::new(file.to_str().unwrap()).unwrap();
        unsafe {
            let mode = std::ffi::CString::new("w").unwrap();
            let file_ptr =
                freesasa_sys::fopen(file.as_ptr(), mode.as_ptr());

            if file_ptr.is_null() {
                return Err("Could not open file");
            }

            freesasa_set_err_out(file_ptr);
        }
    }
    Ok(())
}
