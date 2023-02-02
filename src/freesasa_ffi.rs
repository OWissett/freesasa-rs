// We include the bindings in its own module so that we don't expose the raw FFI bindings directly
// when we publish the crate.
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused)]
#[allow(clippy::upper_case_acronyms)]
mod internal_ {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Re-export the bindings from the internal module
pub(crate) use internal_::*;

#[cfg(test)]
mod tests {
    use crate::freesasa_ffi::{
        fopen, freesasa_calc_structure, freesasa_classifier,
        freesasa_protor_classifier, freesasa_structure_from_pdb,
    };
    use std::{ffi, ptr};

    #[test]
    fn freesasa_calculation() {
        unsafe {
            // Define the file name
            let pdb_filename =
                ffi::CString::new("./data/single_chain.pdb").unwrap();

            // Define the file mode
            let modes = ffi::CString::new("r").unwrap();
            // Create the default classifier
            //

            let classifier: *const freesasa_classifier =
                &freesasa_protor_classifier;

            // Load file as C-style FILE pointer
            let pdb_file = fopen(pdb_filename.as_ptr(), modes.as_ptr());

            // Load structure
            let structure =
                freesasa_structure_from_pdb(pdb_file, classifier, 0);

            let fs_result =
                freesasa_calc_structure(structure, ptr::null());

            println!("Total SASA: {}", *(*fs_result).sasa);
        }
    }
}
