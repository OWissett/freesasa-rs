#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn fs_structure_from_path(
    pdb_path: &str,
    options: Option<std::os::raw::c_int>,
) -> Result<*mut freesasa_structure, &'static str> {

    let pdb_path = std::ffi::CString::new(pdb_path).unwrap();

    let classifier: *const freesasa_classifier = unsafe {&freesasa_protor_classifier};
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
            return Err("Unable to load structure from given path, freesasa returned a null pointer!");
        }

        Ok(structure)
    }
}

pub fn fs_calc_structure(
    structure: *mut freesasa_structure
) -> Result<freesasa_result> {

}

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod rusty_tests {

        use crate::*;

        #[test]
        fn test_fs_structure_from_path() {
            let structure = fs_structure_from_path("./data/5XH3.pdb", None).unwrap();
            let structure_inc_het = fs_structure_from_path("./data/5XH3.pdb", Some(1)).unwrap();

            unsafe  {
                let fs_result = freesasa_calc_structure(structure, std::ptr::null());
                let fs_result_inc_het = freesasa_calc_structure(structure_inc_het, std::ptr::null());

                println!("Total SASA: {}", (*fs_result).total);
                println!("Total SASA: {}", (*fs_result_inc_het).total);
            }
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
