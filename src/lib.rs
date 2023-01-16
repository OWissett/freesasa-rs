// Not all FreeSASA C-API bindings conform
// to Rust naming standards, so we need
// to disable compiler warnings about naming
// conventions.
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Macro Include the bindings into the scope.
// This in essensence is the same as #include directive in C/C++
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


//------------------//
//  Public Structs  //
//------------------//

/// Simple Rust struct wrapper for freesasa_structure object.
///
/// Object currently can only be instantiated from a pdb_path
pub struct PDBStructure {
    /// Raw pointer to the C-API freesasa_structure object. Avoid using this directly, since there
    /// is potential for memory issues, only use this if you accept responsibility for understanding
    /// the memory yourself, else risk memory leaks...
    ptr: *mut freesasa_structure,

    /// Name of the PDB file which the structure was loaded from.
    pdb_name: String,
}

impl Drop for PDBStructure {
    fn drop(&mut self) {
        unsafe {
            freesasa_structure_free(self.ptr);
        }
    }
}

impl std::fmt::Display for PDBStructure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(pdb_name: {})", self.pdb_name)
    }
}

impl PDBStructure {
    pub fn from_pdb(
        path: &str,
        options: Option<std::os::raw::c_int>
    ) -> PDBStructure {
        fs_structure_from_path(path, options).unwrap()
    }

    // TODO - Implement the selection logic wrapper
    // TODO - Implement the calculation wrapper
}

/// Rust wrapper for FreeSASA C-API freesasa_result object
pub struct FSResult {
    /// Pointer to C-API object
    ptr: *mut freesasa_result,

    // Total SASA valu
    total: f64,

    // Pointer to array
    sasa_ptr: *mut f64,
    n_atoms: i32,
}

impl FSResult {
    pub fn new(ptr: *mut freesasa_result) -> Result<FSResult, &'static str> {

        if ptr.is_null() {
            return Err("Null pointer was given to FSResult::new");
        }

        #[cfg(feature = "nightly-features")]
        if !ptr.is_aligned() {
            return Err("Incorrectly aligned pointer was given to FSResult::new");
        }

        unsafe {
            let total = (*ptr).total;
            let sasa_ptr = (*ptr).sasa;
            let n_atoms = (*ptr).n_atoms;
            Ok(FSResult { ptr: ptr, total: total, sasa_ptr: sasa_ptr, n_atoms: n_atoms })
        }
    }

    pub fn get_atom_sasa_values(&self) -> Vec<f64> {
        let mut v: Vec<f64> = Vec::with_capacity(self.n_atoms as usize);
        for i in 0..self.n_atoms {
            unsafe {
                v.push(*self.sasa_ptr.offset(i as isize));
            }
        }
        return v;
    }
}

impl Drop for FSResult {
    fn drop(&mut self) {
        unsafe {
            freesasa_result_free(self.ptr);
        }
    }
}

impl std::fmt::Display for FSResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.total)
    }
}

pub struct FSResultTree {

}

//-------------------//
// Private Functions //
//-------------------//

/// Returns a result containing the PDBStructure or error. This function can panic.
///
/// # Arguments
///
/// * `pdb_path` - A string slice that holds the path to the pdb file
/// * `options` - An optional c-style integer which acts as a bit field for the structure loading
///
/// For more details about the options field, read the FreeSASA C-API documentation.
///
fn fs_structure_from_path(
    pdb_path: &str,
    options: Option<std::os::raw::c_int>,
) -> Result<PDBStructure, &'static str> {

    let pdb_name = *pdb_path.split("/")
                                  .collect::<Vec<&str>>()
                                  .last()
                                  .unwrap()
                                  .split('.')
                                  .collect::<Vec<&str>>()
                                  .first()
                                  .unwrap();

    let pdb_path = std::ffi::CString::new(pdb_path).unwrap();

    let classifier: *const freesasa_classifier = unsafe {&freesasa_protor_classifier};

    // Bitfield
    let options = options.unwrap_or(0) as std::os::raw::c_int;
    let modes = std::ffi::CString::new("r").unwrap();

    unsafe {
        let file = fopen(pdb_path.as_ptr(), modes.as_ptr());
        let structure = freesasa_structure_from_pdb(
            file,
            classifier as *const freesasa_classifier,
            options,
        );

        if structure.is_null() {
            return Err("Unable to load structure for given path, freesasa returned a null pointer!");
        }

        Ok(PDBStructure{ptr: structure, pdb_name: String::from(pdb_name)})
    }
}


//---------//
// Testing //
//---------//

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod rusty_tests {

        use crate::*;

        /// To run this test do (assuming running linux):
        ///     `RUSTFLAGS="--cfg leak_test" cargo test`
        #[cfg(leak_test)]
        #[test]
        fn leak_test() {
            for _ in 1..10000 {
                let _ = fs_structure_from_path("./data/5XH3.pdb", Some(0));
            }
        }

        #[test]
        fn test_fs_structure_from_path() {
            let structure = fs_structure_from_path("./data/5XH3.pdb", None).unwrap();
            let structure_inc_het = fs_structure_from_path("./data/5XH3.pdb", Some(1)).unwrap();

            println!("Structure: {}", structure);

            let fs_result: *mut freesasa_result;
            let fs_result_inc_het: *mut freesasa_result;

            unsafe  {
                fs_result = freesasa_calc_structure(structure.ptr, std::ptr::null());
                fs_result_inc_het = freesasa_calc_structure(structure_inc_het.ptr, std::ptr::null());

                println!("Total SASA: {}", (*fs_result).total);
                println!("Total SASA: {}", (*fs_result_inc_het).total);
            }

            let fs_result = FSResult::new(fs_result).unwrap();
            let fs_result_inc_het = FSResult::new(fs_result_inc_het).unwrap();

            println!("{:?}", fs_result.get_atom_sasa_values());
            println!("{:?}", fs_result_inc_het.get_atom_sasa_values());

            // Check that the total and the sum of get_atom_sasa_values is nearly the same
            // Not checking if they are identical since we may have some floating point
            // errors.

            let total = fs_result.total;
            let total_via_atoms: f64 = fs_result.get_atom_sasa_values().iter().sum();
            let diff = (total - total_via_atoms).abs();

            assert!(diff <= (total / 100.0f64)); // Allow 1% error
            println!("Diff: {}", diff);

        }

    }

    #[cfg(test)]
    mod raw_ffi_tests {
        use crate::*;
        use std::ffi;

        #[test]
        fn freesasa_calculation() {
            unsafe {
                // Define the file name
                let pdb_filename = ffi::CString::new("./data/5XH3.pdb").unwrap();

                // Define the file mode
                let modes = ffi::CString::new("r").unwrap();
                // Create the default classifier
                //

                let classifier: *const freesasa_classifier = &freesasa_protor_classifier;

                // Load file as C-style FILE pointer
                let pdb_file = fopen(pdb_filename.as_ptr(), modes.as_ptr());

                // Load structure
                let structure = freesasa_structure_from_pdb(pdb_file, classifier, 0);

                let fs_result = freesasa_calc_structure(structure, std::ptr::null());

                println!("Total SASA: {}", *(*fs_result).sasa);
            }
        }

        #[test]
        fn freesasa_selection() {



        }
    }
}
