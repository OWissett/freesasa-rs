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
    pub(crate) unsafe fn new(
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

        let total = (*ptr).total;
        let sasa_ptr = (*ptr).sasa;
        let n_atoms = (*ptr).n_atoms;

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

    /// Returns a mutable pointer to the underlying C-API object
    #[cfg(not(feature = "unsafe-ops"))]
    #[allow(dead_code)]
    pub(crate) fn as_ptr(&self) -> *mut freesasa_result {
        self.ptr
    }

    #[cfg(feature = "unsafe-ops")]
    pub fn as_ptr(&self) -> *mut freesasa_result {
        self.ptr
    }

    /// Returns a const pointer to the underlying C-API object
    #[cfg(not(feature = "unsafe-ops"))]
    pub(crate) fn as_const_ptr(&self) -> *const freesasa_result {
        self.ptr as *const freesasa_result
    }

    #[cfg(feature = "unsafe-ops")]
    pub fn as_const_ptr(&self) -> *const freesasa_result {
        self.ptr as *const freesasa_result
    }

    /// Returns true if the pointer is null
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

        Some(unsafe { *self.sasa_ptr.add(index) })
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

        let value = unsafe { *self.result.sasa_ptr.add(self.index) };
        self.index += 1;
        Some(value)
    }
}

#[cfg(test)]
mod tests {

    use crate::{set_verbosity, structure, FreesasaVerbosity};

    use super::*;

    #[test]
    fn test_new() {
        let ptr = std::ptr::null_mut();
        let result = unsafe { SasaResult::new(ptr) };
        assert!(result.is_err());

        let structure = structure::Structure::from_path(
            "./data/single_chain.pdb",
            None,
        )
        .unwrap();

        let _ = structure.calculate_sasa().unwrap();
    }

    #[test]
    fn test_atom_sasa() {
        set_verbosity(FreesasaVerbosity::Debug);

        let structure = structure::Structure::from_path(
            "./data/single_chain.pdb",
            None,
        )
        .unwrap();

        let result = structure.calculate_sasa().unwrap();
        let sasa = result.atom_sasa();

        assert_eq!(sasa.len(), 1911);
    }

    #[test]
    fn test_iter() {
        let structure = structure::Structure::from_path(
            "./data/single_chain.pdb",
            None,
        )
        .unwrap();

        let result = structure.calculate_sasa().unwrap();

        assert_eq!(result.iter().count(), 1911);

        let sasa = result.iter().sum::<f64>();
        assert_eq!(sasa, result.total);

        let sasa = result.iter().fold(0.0, |acc, x| acc + x);
        assert_eq!(sasa, result.total);

        let sasa = result.iter().map(|x| x * 2.0).sum::<f64>();
        assert_eq!(sasa, result.total * 2.0);

        let sasa_count = result.iter().filter(|x| *x > 0.0).count();
        assert_eq!(sasa_count, 901);
    }
}
