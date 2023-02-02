use crate::freesasa_ffi;

#[derive(Debug)]
pub struct Selection {
    ptr: *mut freesasa_ffi::freesasa_selection,
}
