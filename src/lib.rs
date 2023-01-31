// Copyright 2023. Oliver Wissett, Matt Greenig, and Pietro Sormanni. All rights reserved.

mod fs_ffi;
pub mod result;
pub mod structure;

use std::{ffi, os::raw};

// To expose the raw FFI bindings, compile with `RUSTFLAGS="--cfg expose_raw_ffi"`
#[cfg(expose_raw_ffi)]
pub use freesasa_ffi;

// Bring the needed freesasa functions into scope
use fs_ffi::{
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
