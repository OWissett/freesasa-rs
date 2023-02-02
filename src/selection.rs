#![allow(dead_code)] // TODO: Remove this later once implemented. Just here to keep compiler happy
use crate::freesasa_ffi;

#[derive(Debug)]
pub struct Selection {
    ptr: *mut freesasa_ffi::freesasa_selection,
}

// TODO: Implement freesasa selection functions
