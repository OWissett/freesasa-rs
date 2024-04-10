use std::ffi::{OsStr, OsString};
use std::str::FromStr;
use std::{fmt, os::raw, ptr};

use crate::classifier::DEFAULT_CLASSIFIER;
use crate::error::{FreesasaError, FreesasaErrorKind};
use crate::free_raw_c_strings;
use crate::result::node::NodeType;
use crate::utils::{char_to_c_char, str_to_c_string};
use freesasa_sys::{
    fclose, fopen, freesasa_calc_structure, freesasa_calc_tree,
    freesasa_classifier, freesasa_error_codes_FREESASA_SUCCESS,
    freesasa_parameters, freesasa_structure,
    freesasa_structure_add_atom, freesasa_structure_free,
    freesasa_structure_from_pdb, freesasa_structure_new,
    freesasa_structure_options_FREESASA_HALT_AT_UNKNOWN,
    freesasa_structure_options_FREESASA_INCLUDE_HETATM,
    freesasa_structure_options_FREESASA_INCLUDE_HYDROGEN,
    freesasa_structure_options_FREESASA_JOIN_MODELS,
    freesasa_structure_options_FREESASA_RADIUS_FROM_OCCUPANCY,
    freesasa_structure_options_FREESASA_SEPARATE_CHAINS,
    freesasa_structure_options_FREESASA_SEPARATE_MODELS,
    freesasa_structure_options_FREESASA_SKIP_UNKNOWN,
};

use crate::result::{SasaResult, SasaTree};

/// Bitfield to store structure loading options
type OptionsBitfield = u32;

/// Set the default behaviour for PDB loading
pub(crate) const DEFAULT_STRUCTURE_OPTIONS: OptionsBitfield =
    0 as OptionsBitfield;

/// Set the default behaviour for SASA calculation
pub(crate) const DEFAULT_CALCULATION_PARAMETERS:
    *const freesasa_parameters = ptr::null();

/// Rust struct wrapper for options when creating freesasa_structure object
/// Uses OptionsBitfield type to set booleans for freesasa_structure,
/// regarding options that can be included as a bitfield
/// when instantiated from a path to a pdb
#[derive(Debug)]
pub struct StructureOptions {
    /// Bitfield to determine options for freesasa_structure object
    bitfield: OptionsBitfield,
}

impl StructureOptions {
    /// Creates bitfield for freesasa_structure when instantiated from pdb,
    /// regarding the building of a freesasa_structure object
    ///
    /// ## Arguments
    /// * `include_hetam` - Boolean regarding inclusion of HETATM entries
    /// * `include_hydrogen` - Boolean regarding inclusion of hydrogen atoms
    /// * `separate_models` - Boolean regarding reading MODELs as different structures
    /// * `jseparate_chains` - Boolean regarding reading separate chains as separate structures
    /// * `join_models` - Boolean regarding reading MODELs as part of on structure
    /// * `halt_at_unknown` - Boolean regarding halting reading when unknown atom is encountered
    /// * `skip_unknown` - Boolean regarding skipping current atom when unknown atom is encountered
    /// * `radius_from_occupancy` - Boolean regarding reading atom radius from occupancy field
    fn new(
        include_hetatm: bool,
        include_hydrogen: bool,
        separate_models: bool,
        separate_chains: bool,
        join_models: bool,
        halt_at_unknown: bool,
        skip_unknown: bool,
        radius_from_occupancy: bool,
    ) -> Self {
        let mut bitfield = 0 as OptionsBitfield;
        if include_hetatm {
            bitfield = bitfield
                | freesasa_structure_options_FREESASA_INCLUDE_HETATM;
        }
        if include_hydrogen {
            bitfield = bitfield
                | freesasa_structure_options_FREESASA_INCLUDE_HYDROGEN;
        }
        if separate_models {
            bitfield = bitfield
                | freesasa_structure_options_FREESASA_SEPARATE_MODELS;
        }
        if separate_chains {
            bitfield = bitfield
                | freesasa_structure_options_FREESASA_SEPARATE_CHAINS;
        }
        if join_models {
            bitfield = bitfield
                | freesasa_structure_options_FREESASA_JOIN_MODELS;
        }
        if halt_at_unknown {
            bitfield = bitfield
                | freesasa_structure_options_FREESASA_HALT_AT_UNKNOWN;
        }
        if skip_unknown {
            bitfield = bitfield
                | freesasa_structure_options_FREESASA_SKIP_UNKNOWN;
        }
        if radius_from_occupancy {
            bitfield = bitfield | freesasa_structure_options_FREESASA_RADIUS_FROM_OCCUPANCY;
        }
        Self { bitfield }
    }
}

impl Default for StructureOptions {
    /// Defaults StructureOptions bitfield to 0
    fn default() -> Self {
        Self {
            bitfield: 0 as OptionsBitfield,
        }
    }
}

pub struct StructureBuilder {
    name: String,
    options: Option<StructureOptions>,
}



/// Simple Rust struct wrapper for freesasa_structure object.
///
/// Object currently can only be instantiated from a path to a pdb,
/// as an empty structure, or from a [`pdbtbx::PDB`] object.
///
/// When creating an empty structure, you need
/// to then add atoms to it using `.add_atoms()` before attempting
/// to calculate the SASA.
///
/// To access the raw pointer to the C-API freesasa_structure object,
/// the `unsafe-ops` feature needs to be enabled.
#[derive(Debug)]
pub struct Structure {
    /// Raw pointer to the C-API freesasa_structure object.
    ///
    /// ### WARNING
    /// Don't mess with this unless you understand the risks...
    ptr: *mut freesasa_structure,

    /// Name of the PDB file which the structure was loaded from.
    name: String, // Note that this string must be C compatible, e.g., ASCII only
}

impl Structure {
    /// Creates an empty FSStructure
    ///
    /// FreeSASA C-API function: `freesasa_structure_new`
    ///
    /// Unlike the C-API function, you do not need to manually call
    /// `freesasa_structure_free` on the returned structure. This is
    /// handled by the Drop trait. However, you do need to manually
    /// add atoms to the structure before attempting to calculate
    /// the SASA.
    ///
    /// ## Arguments
    /// * `name` - A string slice that provides the name of the pdb structure (default: "Unnamed")
    ///
    /// ## Errors
    /// * If [`freesasa_structure_new`] returns a null pointer - E.g., unable to allocate memory
    ///
    pub fn new_empty(
        name: Option<&str>,
    ) -> Result<Structure, FreesasaError> {
        let ptr = unsafe { freesasa_structure_new() };
        if ptr.is_null() {
            return Err(FreesasaError::new(
                "failed to create an empty Structure.",
                FreesasaErrorKind::Structure,
                None,
            ));
        }

        let name = name.unwrap_or("Unnamed").to_string();
        Ok(Structure { ptr, name })
    }

    /// Creates an FSStructure from a path to a valid PDB file.
    ///
    /// FreeSASA C-API function: `freesasa_structure_from_pdb`
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
        options: Option<StructureOptions>,
    ) -> Result<Structure, &'static str> {
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
            options.unwrap_or_default().bitfield as raw::c_int;

        // Define the file path and read mode as raw pointers
        let pdb_path = str_to_c_string(pdb_path)?.into_raw();
        let modes = str_to_c_string("r")?.into_raw();

        // Get a C-style file handle
        let file = unsafe { fopen(pdb_path, modes) };

        // Return ownership of pdb_path and modes to Rust
        free_raw_c_strings!(pdb_path, modes);

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

        Ok(Structure {
            ptr: structure,
            name: String::from(pdb_name),
        })
    }

    /// Creates a RustSASA [`Structure`] from a [`pdbtbx::PDB`].
    pub fn from_pdbtbx(
        pdbtbx_structure: &pdbtbx::PDB,
    ) -> Result<Self, &'static str> {
        let name = pdbtbx_structure
            .identifier
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());

        let mut fs_structure = Self::new_empty(Some(name.as_str()))?;

        // Build the structure
        for chain in pdbtbx_structure.chains() {
            for residue in chain.residues() {
                for atom in residue.atoms() {
                    let atom_name = atom.name();
                    let res_name = residue.name().unwrap_or("UNK");
                    let res_number = {
                        let (num, ic) = residue.id();
                        num.to_string() + ic.unwrap_or("")
                    };

                    let pos = atom.pos();

                    let chain_id = {
                        let cid = chain.id();
                        if cid.len() != 1 {
                            error!("Found {} as chain ID, it must be a single ASCII character!", chain.id());
                            return Err("Chain IDs must be single characters! Check logs.");
                        }
                        cid.chars().next().unwrap()
                    };

                    if fs_structure
                        .add_atom(
                            atom_name,
                            res_name,
                            res_number.as_str(),
                            chain_id,
                            pos,
                        )
                        .is_err()
                    {
                        warn!(
                            "Unable to add atom {} to {}",
                            atom_name, &name
                        );
                    }
                }
            }
        }

        Ok(fs_structure)
    }

    /// Adds atoms to the structure
    pub fn add_atom(
        &mut self, // We should indicate to the compiler, that this is a mutable reference, since we are modifying the underlying data structure
        atom_name: &str,
        res_name: &str,
        res_number: &str,
        chain_label: char,
        (x, y, z): (f64, f64, f64),
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
        free_raw_c_strings![atom_name, res_name, res_number];

        if res_code == freesasa_error_codes_FREESASA_SUCCESS {
            Ok(())
        } else {
            Err("Failed to add atom to structure") // Here we should return a more useful error message
        }
    }

    /// Calculates the total SASA value of the structure using default parameters
    pub fn calculate_sasa(&self) -> Result<SasaResult, &str> {
        unsafe {
            SasaResult::new(freesasa_calc_structure(
                self.ptr,
                DEFAULT_CALCULATION_PARAMETERS,
            ))
        }
    }

    /// Calculates the SASA value as a tree using the default parameters
    pub fn calculate_sasa_tree(
        &self,
        depth: &NodeType,
    ) -> Result<SasaTree, &'static str> {
        let name = str_to_c_string(&self.name)?.into_raw();
        let root = unsafe {
            freesasa_calc_tree(
                self.ptr,
                DEFAULT_CALCULATION_PARAMETERS,
                name,
            )
        };

        // Retake CString ownership
        free_raw_c_strings!(name);

        if root.is_null() {
            return Err("freesasa_calc_tree returned a null pointer!");
        }

        Ok(SasaTree::new(root, depth))
    }

    /// Returns a string slice to the name of the structure
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    // --------------- //
    // Pointer Methods //
    // --------------- //

    /// Returns the underlying pointer to the freesasa_structure C object.
    ///
    /// ### WARNING
    /// You are dealing with a raw mutable pointer, you need to understand where
    /// this pointer is going to be used and ensure that the memory is not deallocated!
    /// If this pointer is deallocated early, you will get undefined behaviour since
    /// Drop will attempt to free the same memory (e.g. double free) when this FSStructure
    /// object is destroyed.
    #[cfg(not(feature = "unsafe-ops"))]
    #[allow(dead_code)]
    pub(crate) fn as_ptr(&self) -> *mut freesasa_structure {
        self.ptr
    }

    #[cfg(feature = "unsafe-ops")]
    pub fn as_ptr(&self) -> *mut freesasa_structure {
        self.ptr
    }

    #[cfg(not(feature = "unsafe-ops"))]
    pub(crate) fn as_const_ptr(&self) -> *const freesasa_structure {
        self.ptr as *const freesasa_structure
    }

    #[cfg(feature = "unsafe-ops")]
    pub fn as_const_ptr(&self) -> *const freesasa_structure {
        self.ptr as *const freesasa_structure
    }
}

// --------------------- //
// Trait Implementations //
// --------------------- //

impl Drop for Structure {
    fn drop(&mut self) {
        unsafe {
            freesasa_structure_free(self.ptr);
        }
    }
}

impl fmt::Display for Structure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(pdb_name: {})", self.name)
    }
}

// ----- //
// Tests //
// ----- //

#[cfg(test)]
mod tests {

    use std::ffi;

    use freesasa_sys::{
        freesasa_structure_chain_labels, freesasa_structure_get_chains,
    };

    use crate::{classifier::DEFAULT_CLASSIFIER, set_verbosity};

    use super::*;

    #[test]
    fn from_path() {
        let _ = Structure::from_path(
            "./data/single_chain.pdb",
            Some(StructureOptions::default()),
        )
        .unwrap();
    }

    #[test]
    fn from_pdbtbx() {
        let (pdb, _e) = pdbtbx::open(
            "./data/7trr.pdb",
            pdbtbx::StrictnessLevel::Loose,
        )
        .unwrap();

        let pdb_from_pdbtbx = Structure::from_pdbtbx(&pdb).unwrap();

        let pdb_from_path =
            Structure::from_path("./data/7trr.pdb", None).unwrap();

        let tree_pdbtbx = pdb_from_pdbtbx.calculate_sasa().unwrap();
        let tree_path = pdb_from_path.calculate_sasa().unwrap();

        let percent_diff = (tree_pdbtbx.total() - tree_path.total())
            / tree_pdbtbx.total()
            * 100.0;

        assert!(percent_diff < 0.1);
    }

    #[test]
    fn new_empty() {
        let hello = Structure::new_empty(Some("hello")).unwrap();
        assert!(hello.get_name() == "hello");
    }

    #[test]
    fn add_atom() {
        set_verbosity(crate::FreesasaVerbosity::Silent);
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

        let mut structure = Structure::new_empty(Some("test")).unwrap();

        for atom in atoms {
            structure
                .add_atom(
                    atom.0,
                    atom.1,
                    atom.2,
                    atom.3,
                    (atom.4, atom.5, atom.6),
                )
                .unwrap();
        }

        let full_sasa = structure.calculate_sasa().unwrap().total();

        println!("full: {}\n\n", full_sasa);

        assert_eq!(full_sasa, 257.35019683715666);
    }

    #[test]
    fn test_get_chains() {
        let structure = Structure::from_path(
            "./data/multi_chain.pdb",
            Some(StructureOptions::default()),
        )
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
