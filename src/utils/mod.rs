/// # Utils
///
/// This module contains utility functions and macros used throughout the crate.
/// These functions and macros are not intended to be used by the end user.
///
/// ## Functions
///
/// - [`char_to_c_char`] - Casts a `char` to a `raw::c_char` and checks that the
///  `char` is ASCII.
/// - [`str_to_c_string`] - Casts a `str` to a `ffi::CString` and checks that the
/// `str` does not contain any null bytes.
///
use std::{ffi, os::raw};

pub(crate) mod macros;

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
