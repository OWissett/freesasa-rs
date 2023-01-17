// Copyright 2023. Oliver Wissett, Matt Greenig, and Pietro Sormanni. All rights reserved.

pub mod result;
pub mod structure;

// We include the bindings in its own module so that we don't expose the raw FFI bindings directly
// when we publish the crate.
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused)]
#[allow(clippy::upper_case_acronyms)]
mod freesasa {
    // Macro Include the bindings into the scope.
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::{ffi, os::raw};

// To expose the raw FFI bindings, compile with `RUSTFLAGS="--cfg expose_raw_ffi"`
#[cfg(expose_raw_ffi)]
pub use freesasa;

// Bring the needed freesasa functions into scope
use freesasa::{
    freesasa_set_verbosity, freesasa_verbosity_FREESASA_V_DEBUG,
    freesasa_verbosity_FREESASA_V_NORMAL,
    freesasa_verbosity_FREESASA_V_NOWARNINGS,
    freesasa_verbosity_FREESASA_V_SILENT,
};

pub enum FreesasaVerbosity {
    Debug,
    Normal,
    NoWarnings,
    Silent,
}

// ---------------- //
// Public Functions //
// ---------------- //
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

    unsafe {
        freesasa_set_verbosity(verbosity);
    }
}

// ------------------ //
// Internal Functions //
// ------------------ //
pub(crate) fn char_to_c_char(
    _char: char,
) -> Result<raw::c_char, &'static str> {
    let _char = _char as u32;
    if _char <= 127 {
        Ok(_char as raw::c_char)
    } else {
        Err("Failed to cast char to c_char: non-ASCII char")
    }
}

pub(crate) fn str_to_c_string(
    _str: &str,
) -> Result<ffi::CString, &'static str> {
    match ffi::CString::new(_str) {
        Ok(_str) => Ok(_str),
        Err(_) => Err("Failed to cast str to CString: NulError"),
    }
}

// ------ //
// Macros //
// ------ //

macro_rules! free_raw_c_string {
    ( $( $x:expr ),* ) => {
        {unsafe {
            $(
                if $x.is_null() {
                    panic!();
                }
                let _ = std::ffi::CString::from_raw($x);
            )*
        }}
    };
}

pub(crate) use free_raw_c_string;

//---------//
// Testing //
//---------//

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod raw_ffi_tests {
        use crate::freesasa::{
            fopen, freesasa_calc_structure, freesasa_classifier,
            freesasa_protor_classifier, freesasa_structure_from_pdb,
        };
        use std::{ffi, ptr};

        #[test]
        fn freesasa_calculation() {
            unsafe {
                // Define the file name
                let pdb_filename =
                    ffi::CString::new("./data/single_chain.pdb")
                        .unwrap();

                // Define the file mode
                let modes = ffi::CString::new("r").unwrap();
                // Create the default classifier
                //

                let classifier: *const freesasa_classifier =
                    &freesasa_protor_classifier;

                // Load file as C-style FILE pointer
                let pdb_file =
                    fopen(pdb_filename.as_ptr(), modes.as_ptr());

                // Load structure
                let structure = freesasa_structure_from_pdb(
                    pdb_file, classifier, 0,
                );

                let fs_result =
                    freesasa_calc_structure(structure, ptr::null());

                println!("Total SASA: {}", *(*fs_result).sasa);
            }
        }
    }
}
