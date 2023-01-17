// Copyright 2023. Oliver Wissett, Matt Greenig, and Pietro Sormanni. All rights reserved.
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{collections::HashMap, ffi, fmt, os::raw, ptr};

// We include the bindings in its own module so that we don't expose the raw FFI bindings directly
// when we publish the crate.
#[allow(unused)]
mod freesasa {
    // Macro Include the bindings into the scope.
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// To expose the raw FFI bindings, compile with `RUSTFLAGS="--cfg expose_raw_ffi"`
#[cfg(expose_raw_ffi)]
pub use freesasa;

// Bring the needed freesasa functions into scope
use freesasa::{
    fopen, freesasa_calc_structure, freesasa_calc_tree,
    freesasa_classifier, freesasa_error_codes_FREESASA_SUCCESS,
    freesasa_error_codes_FREESASA_WARN, freesasa_node,
    freesasa_node_area, freesasa_node_children, freesasa_node_free,
    freesasa_node_name, freesasa_node_next, freesasa_node_type,
    freesasa_nodetype, freesasa_nodetype_FREESASA_NODE_CHAIN,
    freesasa_nodetype_FREESASA_NODE_RESIDUE, freesasa_parameters,
    freesasa_protor_classifier, freesasa_result, freesasa_result_free,
    freesasa_set_verbosity, freesasa_structure,
    freesasa_structure_add_atom, freesasa_structure_free,
    freesasa_structure_from_pdb, freesasa_structure_new,
    freesasa_tree_init, freesasa_tree_join,
    freesasa_verbosity_FREESASA_V_DEBUG,
    freesasa_verbosity_FREESASA_V_NORMAL,
    freesasa_verbosity_FREESASA_V_NOWARNINGS,
    freesasa_verbosity_FREESASA_V_SILENT,
};

// ---------------- //
// Static Constants //
// ---------------- //

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
///
/// ## Example
/// ```
/// const DEFAULT_STRUCTURE_OPTIONS: raw::c_int = {
///     freesasa_structure_options_FREESASA_HALT_AT_UNKNOWN.bitor(
///         freesasa_structure_options_FREESASA_RADIUS_FROM_OCCUPANCY,
///     )
/// };
/// ```
const DEFAULT_STRUCTURE_OPTIONS: raw::c_int = 0 as raw::c_int;

const DEFAULT_CALCULATION_PARAMETERS: *const freesasa_parameters =
    ptr::null();

pub enum FreesasaVerbosity {
    Debug,
    Normal,
    NoWarnings,
    Silent,
}

//------------------//
//  Public Structs  //
//------------------//

/// Simple Rust struct wrapper for freesasa_structure object.
///
/// Object currently can only be instantiated from a pdb_path
pub struct FSStructure {
    /// Raw pointer to the C-API freesasa_structure object. Avoid using this directly, since there
    /// is potential for memory issues, only use this if you accept responsibility and understanding
    /// of the memory yourself, else risk memory leaks...
    ptr: *mut freesasa_structure,

    /// Name of the PDB file which the structure was loaded from.
    name: String,
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
            .split("/")
            .collect::<Vec<&str>>()
            .last()
            .unwrap()
            .split('.')
            .collect::<Vec<&str>>()
            .first()
            .expect("Failed to get PDB name from path");

        let pdb_path = ffi::CString::new(pdb_path).unwrap();

        // Bitfield
        let options =
            options.unwrap_or(DEFAULT_STRUCTURE_OPTIONS) as raw::c_int;
        let modes = ffi::CString::new("r").unwrap();

        unsafe {
            let file = fopen(pdb_path.as_ptr(), modes.as_ptr());

            if file.is_null() {
                return Err("fopen failed to open file and returned a null pointer");
            }

            let structure = freesasa_structure_from_pdb(
                file,
                DEFAULT_CLASSIFIER as *const freesasa_classifier,
                options,
            );

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
    }

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
        let atom_name = match ffi::CString::new(atom_name) {
            Ok(atom_name) => atom_name,
            Err(_) => {
                return Err("Failed to cast atom_name to CString")
            }
        };
        let res_name = match ffi::CString::new(res_name) {
            Ok(res_name) => res_name,
            Err(_) => return Err("Failed to cast res_name to CString"),
        };
        let res_number = match ffi::CString::new(res_number) {
            Ok(res_number) => res_number,
            Err(_) => {
                return Err("Failed to cast res_number to CString")
            }
        };
        let chain_label = chain_label as u32;
        assert!(
            chain_label <= 127,
            "Invalid chain label (must be ASCII)"
        );
        let chain_label = chain_label as raw::c_char;

        let res_code = unsafe {
            freesasa_structure_add_atom(
                self.ptr,
                atom_name.as_ptr(),
                res_name.as_ptr(),
                res_number.as_ptr(),
                chain_label,
                x,
                y,
                z,
            )
        };

        if res_code == freesasa_error_codes_FREESASA_SUCCESS {
            return Ok(());
        } else {
            return Err("Failed to add atom to structure");
        }
    }

    /// Calculates the total SASA value of the structure using default parameters
    pub fn calculate_sasa(&self) -> Result<FSResult, &str> {
        FSResult::new(unsafe {
            freesasa_calc_structure(
                self.ptr,
                DEFAULT_CALCULATION_PARAMETERS,
            )
        })
    }

    /// Calculates the SASA value as a tree using the default parameters
    pub fn calculate_sasa_tree(
        &self,
    ) -> Result<FSResultTree, &'static str> {
        let name = self.get_cstyle_name();
        let root = unsafe {
            freesasa_calc_tree(
                self.ptr,
                DEFAULT_CALCULATION_PARAMETERS,
                name.as_ptr(),
            )
        };

        if root.is_null() {
            return Err("freesasa_calc_tree returned a null pointer!");
        }

        Ok(FSResultTree::new(root)?)
    }

    fn get_cstyle_name(&self) -> ffi::CString {
        ffi::CString::new(self.name.as_str()).unwrap()
    }
}

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
    /// Creates a `FSResult` object from a raw `freesasa_result` pointer
    pub fn new(
        ptr: *mut freesasa_result,
    ) -> Result<FSResult, &'static str> {
        if ptr.is_null() {
            return Err("Null pointer was given to FSResult::new");
        }

        #[cfg(feature = "nightly-features")]
        if !ptr.is_aligned() {
            return Err(
                "Incorrectly aligned pointer was given to FSResult::new",
            );
        }

        unsafe {
            let total = (*ptr).total;
            let sasa_ptr = (*ptr).sasa;
            let n_atoms = (*ptr).n_atoms;
            Ok(FSResult {
                ptr,
                total,
                sasa_ptr,
                n_atoms,
            })
        }
    }

    /// Returns a vector of SASA values for each ATOM in the molecule
    pub fn get_sasa_vec(&self) -> Vec<f64> {
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

impl fmt::Display for FSResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.total)
    }
}

pub struct FSResultTree {
    root: *mut freesasa_node,
}

impl FSResultTree {
    pub fn new(
        root: *mut freesasa_node,
    ) -> Result<FSResultTree, &'static str> {
        if root.is_null() {
            return Err("Failed to create FSResultTree, the root node was null!");
        }
        Ok(FSResultTree { root })
    }

    pub fn from_result(
        result: &FSResult,
        structure: &FSStructure,
    ) -> Result<FSResultTree, &'static str> {
        let name = structure.get_cstyle_name();

        if structure.ptr.is_null() {
            return Err(
                "Failed to create FSResultTree: structure.ptr was null!",
            );
        }

        if result.ptr.is_null() {
            return Err(
                "Failed to create FSResultTree: result.ptr was null!",
            );
        }

        let root = unsafe {
            freesasa_tree_init(result.ptr, structure.ptr, name.as_ptr())
        };

        if root.is_null() {
            return Err("Failed to create FSResultTree: freesasa_tree_init returned a null pointer!");
        }

        Ok(FSResultTree { root })
    }

    /// Returns the differences with this tree and another. Note, it is assumed that the other tree,
    /// is a subtree (as in all nodes contained in subtree and also present in this tree)
    pub fn get_subtree_difference(
        &self,
        subtree: &FSResultTree,
    ) -> Vec<String> {
        // Psuedo code:
        // 1. Find the chains which contain differences, push a tuple of which each node pointer to
        //    to a vector.
        // 2. For each chain with a difference, calculate the pair-wise residue differences
        // 3. Store information about the residues with a change in values
        //
        //
        // NOTE: This function should probably be re-written using recursion, since we do the same
        //       for chains and residues, but since it is only two levels deep I didn't bother...

        // By calculating the differences between chains first, we can identify which chains need to
        // be searched for the exact residues. This will likely increase the speed since proteins
        // have few chains (typlically less than 10, and I am being generous) but have many residues,
        // as such, we have reduced the search space. One thing to note is that the chain
        // in which the deletion has occurred in will be always be searched on a residue level. This
        // is because deletion of residues will change the SASA. There are possibilities: 1, the
        // deleted region was surface exposed; or 2, the deleted region was buried. Both possibilities
        // will cause a change in SASA area for that chain.
        //
        //  A little bit of time analysis can show this:
        //
        //  Time: O(1 + 1 + m + 2 * (m * n))
        //        => O(2mn + m + 2)
        //        => O(2mn + m)
        //
        //  As m -> 1 and n -> 1, then O(2mn + m) -> O(3) ~ O(1)    This is not going to happen though
        //  As m -> N and n -> 1, then O(2mn + m) -> O(3N) ~ O(N)
        //  If m = 1, then O(2n) and n = N, therefore, O(2N) ~ O(N)
        //
        //  The amortized time complexity is O(N), however, in practice it is faster

        // Get the first chains as a HashMap with the pointers and areas.
        // Time: O(1)
        let chains = FSResultTree::get_node(
            self.root,
            freesasa_nodetype_FREESASA_NODE_CHAIN,
        );

        // Get the second tree's chains
        // Time: O(1)
        let subtree_chains = FSResultTree::get_node(
            subtree.root,
            freesasa_nodetype_FREESASA_NODE_CHAIN,
        );

        // Find the chains which have different SASA values
        // Time: O(m) where m is the number of chains
        let chain_diffs = FSResultTree::nodes_with_differences(
            chains,
            subtree_chains,
        );

        // Find the residues which have differences
        let mut residue_diffs: HashMap<
            String,
            Vec<(*mut freesasa_node, *mut freesasa_node)>,
        > = HashMap::new();

        // Time: O(m * n) - where m is the number of chains with differences and n is the number of
        //                  residues in the chain (this is different for each chain)
        //
        //                  This is realistically faster than computing all residues which is O(N),
        //                  where N is the total number of residues in the residues in the structure
        for chain in chain_diffs {
            let name = FSResultTree::get_node_name(chain.0);
            let res_node = FSResultTree::get_node(
                chain.0,
                freesasa_nodetype_FREESASA_NODE_RESIDUE,
            );
            let subtree_res_node = FSResultTree::get_node(
                chain.1,
                freesasa_nodetype_FREESASA_NODE_RESIDUE,
            );
            residue_diffs.insert(
                name,
                FSResultTree::nodes_with_differences(
                    res_node,
                    subtree_res_node,
                ),
            );
        }

        // Convert the HashMap to vector following FragDB UID residue naming scheme
        // (maybe move this to its own function)
        //
        // Time: O(m * n) - Same as above...
        let mut output_vector = Vec::new();
        for chain in residue_diffs {
            let i = chain.1.iter().map(|res| -> String {
                chain.0.clone()
                    + ":"
                    + FSResultTree::get_node_name(res.0).as_str()
            });
            output_vector.extend(i);
        }

        return output_vector;
    }

    fn nodes_with_differences(
        node: *mut freesasa_node,
        subtree_node: *mut freesasa_node,
    ) -> Vec<(*mut freesasa_node, *mut freesasa_node)> {
        let siblings = FSResultTree::get_siblings_as_vector(node, None);
        let subtree_siblings =
            FSResultTree::get_siblings_as_hashmap(subtree_node);

        let mut v = Vec::new();

        // Find the chains which have different SASA values
        for sibling in siblings {
            let name = FSResultTree::get_node_name(sibling);
            let area = FSResultTree::get_node_area(sibling);

            match subtree_siblings.get(&name) {
                Some((subtree_node, subtree_area)) => {
                    if (area - subtree_area).abs() != 0.0 {
                        v.push((sibling, *subtree_node));
                    }
                }
                None => continue,
            };
        }

        return v;
    }

    /// Joins the given tree with the current tree
    ///
    /// ## Arguments
    /// * `other_tree` - The tree to join. Note that the passed in tree's ownership
    ///             moves to this function, and then memory is deallocated.
    pub fn join(
        &self,
        mut other_tree: FSResultTree,
    ) -> Result<(), &'static str> {
        let code = unsafe {
            freesasa_tree_join(
                self.root,
                ptr::addr_of_mut!(other_tree.root),
            )
        };

        // Set the root of the other tree to null to prevent double freeing of
        // memory.
        other_tree.root = ptr::null_mut();

        if code == freesasa_error_codes_FREESASA_SUCCESS {
            return Ok(());
        } else if code == freesasa_error_codes_FREESASA_WARN {
            println!("A warning occured when joining result trees!");
            return Ok(()); // Everything is probably fine???
        } else {
            return Err("An error occured whilst join result trees!");
        }
    }

    /// Recursively finds the decendent of the node which matches node_type.
    ///
    /// Time: O(n) where n is the depth of the node type decendent.
    ///
    /// ## Arguments
    /// * `node` - The node to decend from
    /// * `node_type` - The type of node to return at
    ///
    /// ## Returns
    /// A mutable pointer to the matching node or a null pointer if no match was found.
    ///
    fn get_node(
        node: *mut freesasa_node,
        node_type: freesasa_nodetype,
    ) -> *mut freesasa_node {
        let current_node_type = unsafe { freesasa_node_type(node) };
        if current_node_type == node_type {
            return node;
        }

        let node = unsafe { freesasa_node_children(node) };
        if node == ptr::null_mut() {
            // Terminate if we have no children (e.g., end of tree)
            return node;
        }

        return FSResultTree::get_node(node, node_type); // Then we go deeper!!!
    }

    /// Makes a HashMap with the node names as the keys, and the values tuples of node pointer
    /// and total SASA area.
    ///
    /// Time: O(n) where n is the number of siblings
    fn get_siblings_as_hashmap(
        node: *mut freesasa_node,
    ) -> HashMap<String, (*mut freesasa_node, f64)> {
        let mut node = node;
        let mut h = HashMap::<String, (*mut freesasa_node, f64)>::new();
        while node != ptr::null_mut() {
            let area = FSResultTree::get_node_area(node);
            let name = FSResultTree::get_node_name(node);
            if h.insert(name, (node, area)) != None {
                println!("WARNING: It appears that multiple siblings have the same name: {}", FSResultTree::get_node_name(node));
            }
            node = unsafe { freesasa_node_next(node) };
        }

        return h;
    }

    /// Retrieves the names and total areas of sibling nodes.
    ///
    /// Time: O(n) where n is the number of sibling nodes
    ///
    /// ## Arguments
    /// * `node` - The node to find all of the siblings of. If the node is not the first in the
    ///            sequence, only nodes after will be added.
    /// * `capacity` - Optionally can provide a capacity which will be used to pre-allocate the
    ///                vector.
    fn get_siblings_as_vector(
        node: *mut freesasa_node,
        capacity: Option<usize>,
    ) -> Vec<*mut freesasa_node> {
        let mut node = node;
        let mut v = match capacity {
            None => Vec::new(),
            Some(capacity) => Vec::with_capacity(capacity),
        };

        while node != ptr::null_mut() {
            v.push(node);
            node = unsafe { freesasa_node_next(node) };
        }

        return v;
    }

    /// Returns the name of the node as a String
    fn get_node_name(node: *mut freesasa_node) -> String {
        let name = unsafe {
            ffi::CStr::from_ptr(freesasa_node_name(node)).to_str()
        };
        let name = match name {
            Ok(name) => name,
            Err(_) => "NoName",
        };
        return String::from(name);
    }

    /// Returns the total area of the node as a f64
    fn get_node_area(node: *mut freesasa_node) -> f64 {
        unsafe { (*freesasa_node_area(node)).total }
    }
}

impl Drop for FSResultTree {
    fn drop(&mut self) {
        unsafe {
            if self.root.is_null() {
                return; // Do need to free if null, tree probably was moved
            }
            freesasa_node_free(self.root);
        }
    }
}

// ---------------- //
// Public Functions //
// ---------------- //
pub fn set_fs_verbosity(verbosity: FreesasaVerbosity) {
    let verbosity = match verbosity {
        FreesasaVerbosity::Debug => freesasa_verbosity_FREESASA_V_DEBUG,
        FreesasaVerbosity::Normal => {
            freesasa_verbosity_FREESASA_V_NORMAL
        }
        FreesasaVerbosity::NoWarnings => {
            freesasa_verbosity_FREESASA_V_NOWARNINGS
        }
        FreesasaVerbosity::Silent => {
            freesasa_verbosity_FREESASA_V_SILENT
        }
    };

    unsafe {
        freesasa_set_verbosity(verbosity);
    }
}

//-------------------//
// Private Functions //
//-------------------//

//---------//
// Testing //
//---------//

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod rusty_tests {

        use crate::{
            freesasa::{
                freesasa_structure_chain_labels,
                freesasa_structure_get_chains,
            },
            *,
        };

        #[test]
        fn test_get_chains() {
            let structure =
                FSStructure::from_path("./data/8bee.pdb", Some(0))
                    .unwrap();

            let chains = ffi::CString::new("P").unwrap();

            let chains = unsafe {
                freesasa_structure_get_chains(
                    structure.ptr,
                    chains.as_ptr(),
                    DEFAULT_CLASSIFIER,
                    0,
                )
            };

            unsafe {
                println!(
                    "{:?}",
                    ffi::CStr::from_ptr(
                        freesasa_structure_chain_labels(chains)
                    )
                    .to_str()
                )
            }
        }

        /// To run this test do (assuming running linux):
        ///     `RUSTFLAGS="--cfg leak_test" cargo test`
        #[cfg(leak_test)]
        #[test]
        fn leak_test() {
            for _ in 1..10000 {
                let _ =
                    fs_structure_from_path("./data/5XH3.pdb", Some(0));
            }
        }

        #[test]
        fn test_fs_structure_from_path() {
            let structure =
                fs_structure_from_path("./data/5XH3.pdb", None)
                    .unwrap();
            let structure_inc_het =
                fs_structure_from_path("./data/5XH3.pdb", Some(1))
                    .unwrap();

            println!("Structure: {}", structure);

            let fs_result: *mut freesasa_result;
            let fs_result_inc_het: *mut freesasa_result;

            unsafe {
                fs_result =
                    freesasa_calc_structure(structure.ptr, ptr::null());
                fs_result_inc_het = freesasa_calc_structure(
                    structure_inc_het.ptr,
                    ptr::null(),
                );

                println!("Total SASA: {}", (*fs_result).total);
                println!("Total SASA: {}", (*fs_result_inc_het).total);
            }

            let fs_result = FSResult::new(fs_result).unwrap();
            let fs_result_inc_het =
                FSResult::new(fs_result_inc_het).unwrap();

            println!("{:?}", fs_result.get_sasa_vec());
            println!("{:?}", fs_result_inc_het.get_sasa_vec());

            // Check that the total and the sum of get_atom_sasa_values is nearly the same
            // Not checking if they are identical since we may have some floating point
            // errors.

            let total = fs_result.total;
            let total_via_atoms: f64 =
                fs_result.get_sasa_vec().iter().sum();
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
                let pdb_filename =
                    ffi::CString::new("./data/5XH3.pdb").unwrap();

                // Define the file mode
                let modes = ffi::CString::new("r").unwrap();
                // Create the default classifier
                //

                let classifier: *const freesasa_classifier =
                    &freesasa_protor_classifier;

                // Load file as C-style FILE pointer
                let pdb_file =
                    fopen(pdb_filename.as_ptr(), modes.as_ptr());

                // Load structure
                let structure = freesasa_structure_from_pdb(
                    pdb_file, classifier, 0,
                );

                let fs_result =
                    freesasa_calc_structure(structure, ptr::null());

                println!("Total SASA: {}", *(*fs_result).sasa);
            }
        }

        #[test]
        fn freesasa_selection() {}
    }
}
