use std::fmt;

use freesasa_sys::{freesasa_result, freesasa_result_free};

/// Rust wrapper for FreeSASA C-API freesasa_result object
#[derive(Debug)]
pub struct SasaResult {
    /// Pointer to C-API object
    ptr: *mut freesasa_result,

    /// Total SASA value
    pub total: f64,

    /// Pointer to underlying C-API SASA array
    sasa_ptr: *mut f64,

    /// Number of atoms in the structure
    pub n_atoms: i32,
}

impl SasaResult {
    /// Creates a [`SasaResult`] object from a raw `freesasa_result` pointer
    ///
    /// ### Safety
    ///
    /// This function will dereference the ptr provided. A null check is performed.
    /// If built with nightly compiler, the pointer's alignment is also checked.
    ///
    /// Do not use the pointer given after passing it to this function, since
    /// [`SasaResult`] is now responsible for the pointer.
    ///
    pub unsafe fn new(
        ptr: *mut freesasa_result,
    ) -> Result<SasaResult, &'static str> {
        if ptr.is_null() {
            return Err("Null pointer was given to FSResult::new");
        }

        #[cfg(feature = "nightly-features")]
        if !ptr.is_aligned() {
            return Err(
                "Incorrectly aligned pointer was given to FSResult::new",
            );
        }

        let total: f64;
        let sasa_ptr: *mut f64;
        let n_atoms: i32;

        unsafe {
            total = (*ptr).total;
            sasa_ptr = (*ptr).sasa;
            n_atoms = (*ptr).n_atoms;
        }

        Ok(SasaResult {
            ptr,
            total,
            sasa_ptr,
            n_atoms,
        })
    }

    /// Returns a vector of SASA values for each ATOM in the molecule
    pub fn atom_sasa(&self) -> Vec<f64> {
        let mut v: Vec<f64> = Vec::with_capacity(self.n_atoms as usize);
        for i in 0..self.n_atoms {
            unsafe {
                v.push(*self.sasa_ptr.offset(i as isize));
            }
        }
        v
    }

    pub(crate) fn as_ptr(&self) -> *mut freesasa_result {
        self.ptr
    }

    pub(crate) fn as_const_ptr(&self) -> *const freesasa_result {
        self.ptr as *const freesasa_result
    }

    pub(crate) fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Returns an iterator over the SASA values in the result
    pub fn iter(&self) -> SasaResultIter {
        SasaResultIter {
            result: self,
            index: 0,
        }
    }

    /// Returns the SASA value for the atom at the given index
    pub fn get(&self, index: usize) -> Option<f64> {
        if index >= self.n_atoms as usize {
            return None;
        }

        Some(unsafe { *self.sasa_ptr.offset(index as isize) })
    }
}

impl Drop for SasaResult {
    /// Releases the memory allocated for the underlying C-API object
    /// when the object goes out of scope. This is called automatically
    /// by the compiler - DO NOT CALL THIS FUNCTION YOURSELF.
    fn drop(&mut self) {
        unsafe {
            freesasa_result_free(self.ptr);
        }
    }
}

impl fmt::Display for SasaResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.total)
    }
}

pub struct SasaResultIter<'a> {
    result: &'a SasaResult,
    index: usize,
}

impl<'a> Iterator for SasaResultIter<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.result.n_atoms as usize {
            return None;
        }

        let value = unsafe {
            *self.result.sasa_ptr.offset(self.index as isize)
        };
        self.index += 1;
        Some(value)
    }
}