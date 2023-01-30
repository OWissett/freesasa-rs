use std::{collections::HashMap, ffi, fmt, ptr};

// Bring the needed freesasa functions into scope
use crate::{
    free_raw_c_string,
    freesasa_ffi::{
        freesasa_error_codes_FREESASA_SUCCESS,
        freesasa_error_codes_FREESASA_WARN, freesasa_node,
        freesasa_node_area, freesasa_node_children, freesasa_node_free,
        freesasa_node_name, freesasa_node_next, freesasa_node_type,
        freesasa_nodetype, freesasa_nodetype_FREESASA_NODE_CHAIN,
        freesasa_nodetype_FREESASA_NODE_RESIDUE, freesasa_result,
        freesasa_result_free, freesasa_tree_init, freesasa_tree_join,
    },
    str_to_c_string,
    structure::FSStructure,
};

/// Rust wrapper for FreeSASA C-API freesasa_result object
#[derive(Debug)]
pub struct FSResult {
    /// Pointer to C-API object
    ptr: *mut freesasa_result,

    // Total SASA valu
    pub total: f64,

    // Pointer to array
    sasa_ptr: *mut f64,
    pub n_atoms: i32,
}

impl FSResult {
    /// Creates a `FSResult` object from a raw `freesasa_result` pointer
    ///
    /// ### Safety
    ///
    /// This function will dereference the ptr provided. A null check is performed.
    /// If built with nightly compiler, the pointer's alignment is also checked.
    ///
    /// Do not use the pointer given after passing it to this function, since
    /// FSResult is now responsible for the pointer.
    ///
    pub unsafe fn new(
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

        let total: f64;
        let sasa_ptr: *mut f64;
        let n_atoms: i32;

        unsafe {
            total = (*ptr).total;
            sasa_ptr = (*ptr).sasa;
            n_atoms = (*ptr).n_atoms;
        }

        Ok(FSResult {
            ptr,
            total,
            sasa_ptr,
            n_atoms,
        })
    }

    /// Returns a vector of SASA values for each ATOM in the molecule
    pub fn get_sasa_vec(&self) -> Vec<f64> {
        let mut v: Vec<f64> = Vec::with_capacity(self.n_atoms as usize);
        for i in 0..self.n_atoms {
            unsafe {
                v.push(*self.sasa_ptr.offset(i as isize));
            }
        }
        v
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

#[derive(Debug)]
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
        let name = str_to_c_string(structure.get_name())?.into_raw();

        if structure.is_null() {
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
            freesasa_tree_init(result.ptr, structure.as_ptr(), name)
        };

        // Return ownership of CString
        free_raw_c_string![name];

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

        output_vector
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

        v
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
            Ok(())
        } else if code == freesasa_error_codes_FREESASA_WARN {
            println!("A warning occured when joining result trees!");
            Ok(()) // Everything is probably fine???
        } else {
            Err("An error occured whilst join result trees!")
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
        if node.is_null() {
            // Terminate if we have no children (e.g., end of tree)
            return node;
        }

        FSResultTree::get_node(node, node_type) // Then we go deeper!!!
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
        while !node.is_null() {
            let area = FSResultTree::get_node_area(node);
            let name = FSResultTree::get_node_name(node);
            if h.insert(name, (node, area)).is_some() {
                println!("WARNING: It appears that multiple siblings have the same name: {}", FSResultTree::get_node_name(node));
            }
            node = unsafe { freesasa_node_next(node) };
        }
        h
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

        while !node.is_null() {
            v.push(node);
            node = unsafe { freesasa_node_next(node) };
        }

        v
    }

    /// Returns the name of the node as a String
    fn get_node_name(node: *mut freesasa_node) -> String {
        let name = unsafe {
            ffi::CStr::from_ptr(freesasa_node_name(node)).to_str()
        };
        let name = name.unwrap_or("NoName");
        String::from(name)
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
