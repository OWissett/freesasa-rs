use std::{ffi, fmt, os::raw, ptr};

use crate::freesasa::{
    fclose, fopen, freesasa_calc_structure, freesasa_calc_tree,
    freesasa_classifier, freesasa_error_codes_FREESASA_SUCCESS,
    freesasa_parameters, freesasa_protor_classifier,
    freesasa_structure, freesasa_structure_add_atom,
    freesasa_structure_free, freesasa_structure_from_pdb,
    freesasa_structure_new,
};
use crate::{char_to_c_char, free_raw_c_string, str_to_c_string};

use crate::result::{FSResult, FSResultTree};

/// Very similar to the macro definition for the default classifier found in the
/// freesasa.h file:
///
/// ```C++
/// #define freesasa_default_classifier freesasa_protor_classifier
/// ```
///
static DEFAULT_CLASSIFIER: &freesasa_classifier =
    unsafe { &freesasa_protor_classifier };

/// Set the default behaviour for PDB loading
const DEFAULT_STRUCTURE_OPTIONS: raw::c_int = 0 as raw::c_int;

const DEFAULT_CALCULATION_PARAMETERS: *const freesasa_parameters =
    ptr::null();

/// Simple Rust struct wrapper for freesasa_structure object.
///
///
/// Object currently can only be instantiated from a pdb_path or as
/// an empty structure. When creating an empty structure, you need
/// to then add atoms to it using `.add_atoms()` before attempting
/// to calculate the SASA.
#[derive(Debug)]
pub struct FSStructure {
    /// Raw pointer to the C-API freesasa_structure object.
    ///
    /// ### WARNING
    /// Don't mess with this unless you understand the risks...
    ptr: *mut freesasa_structure,

    /// Name of the PDB file which the structure was loaded from.
    name: String, // Note that this string must be C compatible, e.g., ASCII only
}

impl FSStructure {
    /// Creates an empty FSStructure
    ///
    /// ## Arguments
    /// * `name` - A string slice that provides the name of the pdb structure (default: "Unnamed")
    ///
    pub fn new_empty(
        name: Option<&str>,
    ) -> Result<FSStructure, &'static str> {
        let ptr = unsafe { freesasa_structure_new() };
        if ptr.is_null() {
            return Err("Failed to create empty FSStructure: freesasa_structure_new returned a null pointer!");
        }

        let name = name.unwrap_or("Unnamed").to_string();
        Ok(FSStructure { ptr, name })
    }

    /// Creates an FSStructure from a path to a valid PDB file.
    ///
    /// ## Arguments
    ///
    /// * `pdb_path` - A string slice that holds the path to the pdb file
    /// * `options` - An optional c-style integer which acts as a bit field for the structure loading.
    ///               If not given, the default option is 0.
    ///
    /// For more details about the options field, read the FreeSASA C-API documentation for
    /// `freesasa_structure_from_pdb`
    ///
    pub fn from_path(
        pdb_path: &str,
        options: Option<raw::c_int>,
    ) -> Result<FSStructure, &'static str> {
        let pdb_name = *pdb_path
            .split('/')
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .split('.')
            .collect::<Vec<&str>>()
            .first()
            .expect("Failed to get PDB name from path");

        // Bitfield
        let options =
            options.unwrap_or(DEFAULT_STRUCTURE_OPTIONS) as raw::c_int;

        // Define the file path and read mode as raw pointers
        let pdb_path = str_to_c_string(pdb_path)?.into_raw();
        let modes = str_to_c_string("r")?.into_raw();

        // Get a C-style file handle
        let file = unsafe { fopen(pdb_path, modes) };

        // Return ownership of pdb_path and modes to Rust
        free_raw_c_string!(pdb_path, modes);

        if file.is_null() {
            return Err(
                "fopen failed to open file and returned a null pointer",
            );
        }

        // Create the C freesasa_structure object from the file pointer
        let structure = unsafe {
            freesasa_structure_from_pdb(
                file,
                DEFAULT_CLASSIFIER as *const freesasa_classifier,
                options,
            )
        };

        // Close the file stream
        unsafe {
            fclose(file);
        }

        if structure.is_null() {
            return Err(
                "Unable to load structure for given path, freesasa returned a null pointer!",
            );
        }

        Ok(FSStructure {
            ptr: structure,
            name: String::from(pdb_name),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_atom(
        &self,
        atom_name: &str,
        res_name: &str,
        res_number: &str,
        chain_label: char,
        x: f64,
        y: f64,
        z: f64,
    ) -> Result<(), &'static str> {
        // Convert the types to C-style types
        let atom_name = str_to_c_string(atom_name)?.into_raw();
        let res_name = str_to_c_string(res_name)?.into_raw();
        let res_number = str_to_c_string(res_number)?.into_raw();
        let chain_label = char_to_c_char(chain_label)?;

        let res_code = unsafe {
            freesasa_structure_add_atom(
                self.ptr,
                atom_name,
                res_name,
                res_number,
                chain_label,
                x,
                y,
                z,
            )
        };

        // Retake ownership of CStrings - allowing for proper deallocation of memory
        free_raw_c_string![atom_name, res_name, res_number];

        if res_code == freesasa_error_codes_FREESASA_SUCCESS {
            Ok(())
        } else {
            Err("Failed to add atom to structure")
        }
    }

    /// Calculates the total SASA value of the structure using default parameters
    pub fn calculate_sasa(&self) -> Result<FSResult, &str> {
        unsafe {
            FSResult::new(freesasa_calc_structure(
                self.ptr,
                DEFAULT_CALCULATION_PARAMETERS,
            ))
        }
    }

    /// Calculates the SASA value as a tree using the default parameters
    pub fn calculate_sasa_tree(
        &self,
    ) -> Result<FSResultTree, &'static str> {
        let name = str_to_c_string(&self.name)?.into_raw();
        let root = unsafe {
            freesasa_calc_tree(
                self.ptr,
                DEFAULT_CALCULATION_PARAMETERS,
                name,
            )
        };

        // Retake CString ownership
        unsafe {
            let _ = ffi::CString::from_raw(name);
        }

        if root.is_null() {
            return Err("freesasa_calc_tree returned a null pointer!");
        }

        FSResultTree::new(root)
    }

    /// Returns a string slice to the name of the structure
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the underlying pointer to the freesasa_structure C object.
    ///
    /// ### WARNING
    /// You are dealing with a raw mutable pointer, you need to understand where
    /// this pointer is going to be used and ensure that the memory is not deallocated!
    /// If this pointer is deallocated early, you will get undefined behaviour since
    /// Drop will attempt to free the same memory (e.g. double free) when this FSStructure
    /// object is destroyed.
    pub(crate) fn as_ptr(&self) -> *mut freesasa_structure {
        self.ptr
    }
}

// --------------------- //
// Trait Implementations //
// --------------------- //

impl Drop for FSStructure {
    fn drop(&mut self) {
        unsafe {
            freesasa_structure_free(self.ptr);
        }
    }
}

impl fmt::Display for FSStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(pdb_name: {})", self.name)
    }
}

// ----- //
// Tests //
// ----- //

#[cfg(test)]
mod tests {

    use crate::freesasa::{
        freesasa_structure_chain_labels, freesasa_structure_get_chains,
    };

    use super::*;

    #[test]
    fn from_path() {
        let _ =
            FSStructure::from_path("./data/single_chain.pdb", Some(0))
                .unwrap();
    }

    #[test]
    fn new_empty() {
        let hello = FSStructure::new_empty(Some("hello")).unwrap();
        assert!(hello.get_name() == "hello");
    }

    #[test]
    fn add_atom() {
        let atoms = vec![
            // Atom, ResName, ResNum, Chain, X, Y, Z
            ("N", "ASN", "1", 'A', 10.287, 10.947, 12.500),
            ("CA", "ASN", "1", 'A', 9.479, 9.890, 11.823),
            ("C", "ASN", "1", 'A', 9.495, 10.042, 10.301),
            ("O", "ASN", "1", 'A', 8.855, 10.945, 9.740),
            ("CB", "ASN", "1", 'A', 8.047, 10.028, 12.320),
            ("CG", "ASN", "1", 'A', 7.154, 8.882, 11.864),
            ("OD1", "ASN", "1", 'A', 6.016, 8.731, 12.328),
            ("ND2", "ASN", "1", 'A', 7.658, 8.070, 10.981),
        ];

        let structure = FSStructure::new_empty(Some("test")).unwrap();

        for atom in atoms {
            structure
                .add_atom(
                    atom.0, atom.1, atom.2, atom.3, atom.4, atom.5,
                    atom.6,
                )
                .unwrap();
        }

        let full_sasa = structure.calculate_sasa().unwrap().total;

        println!("full: {}\n\n", full_sasa);
    }

    #[test]
    fn test_get_chains() {
        let structure =
            FSStructure::from_path("./data/multi_chain.pdb", Some(0))
                .unwrap();

        let chains = ffi::CString::new("P").unwrap();

        let chains = unsafe {
            freesasa_structure_get_chains(
                structure.as_ptr(),
                chains.as_ptr(),
                DEFAULT_CLASSIFIER,
                0,
            )
        };

        unsafe {
            println!(
                "{:?}",
                ffi::CStr::from_ptr(freesasa_structure_chain_labels(
                    chains
                ))
                .to_str()
            )
        }
    }
}
