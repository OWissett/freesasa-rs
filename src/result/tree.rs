use std::fmt::{Debug, Display};
use std::{collections::HashMap, ffi, ptr};

use freesasa_sys::{
    freesasa_error_codes_FREESASA_SUCCESS,
    freesasa_error_codes_FREESASA_WARN, freesasa_node,
    freesasa_node_area, freesasa_node_children, freesasa_node_free,
    freesasa_node_name, freesasa_node_next, freesasa_node_parent,
    freesasa_node_residue_number, freesasa_node_type,
    freesasa_nodetype, freesasa_nodetype_FREESASA_NODE_CHAIN,
    freesasa_nodetype_FREESASA_NODE_RESIDUE, freesasa_tree_init,
    freesasa_tree_join,
};

use crate::{
    free_raw_c_strings, structure::Structure, utils::str_to_c_string,
};

use crate::result::SasaResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResidueUID(
    char,         // Chain
    i32,          // Residue number
    Option<char>, // Residue insertion code
);

impl ResidueUID {
    pub fn get_chain(&self) -> char {
        self.0
    }

    pub fn get_resnum(&self) -> i32 {
        self.1
    }

    pub fn get_rescode(&self) -> Option<char> {
        self.2
    }
}

impl Display for ResidueUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.2 {
            Some(code) => write!(f, "{}:{}{}", self.0, self.1, code),
            None => write!(f, "{}:{}", self.0, self.1),
        }
    }
}

type ResidueMap = HashMap<ResidueUID, (*mut freesasa_node, f64)>;

#[derive(Debug)]
pub struct SasaTree {
    root: *mut freesasa_node,
}

impl SasaTree {
    /// Creates a [`SasaTree`] object from a raw `freesasa_node` pointer
    pub fn new(
        root: *mut freesasa_node,
    ) -> Result<SasaTree, &'static str> {
        if root.is_null() {
            return Err("Failed to create FSResultTree, the root node was null!");
        }
        Ok(SasaTree { root })
    }

    pub fn from_result(
        result: &SasaResult,
        structure: &Structure,
    ) -> Result<SasaTree, &'static str> {
        let name = str_to_c_string(structure.get_name())?.into_raw();

        if structure.is_null() {
            return Err(
                "Failed to create FSResultTree: structure.ptr was null!",
            );
        }

        if result.is_null() {
            return Err(
                "Failed to create FSResultTree: result.ptr was null!",
            );
        }

        let root = unsafe {
            freesasa_tree_init(
                result.as_const_ptr(),
                structure.as_const_ptr(),
                name,
            )
        };

        // Return ownership of CString
        free_raw_c_strings![name];

        if root.is_null() {
            return Err("Failed to create FSResultTree: freesasa_tree_init returned a null pointer!");
        }

        Ok(SasaTree { root })
    }

    /// Returns the differences with this tree and another. Note, it is assumed that the other tree,
    /// is a subtree (as in all nodes contained in subtree and also present in this tree)
    ///
    /// ## Arguments
    /// * `subtree` - The subtree to compare to
    /// * `predicate` - The function to use to compare the SASA values (fn(current: f64, other: f64) -> bool)
    ///

    pub fn compare_residues(
        &self,
        subtree: &SasaTree,
        predicate: fn(f64, f64) -> bool,
    ) -> Vec<ResidueUID> {
        // ## For Developers
        // ### Psuedo code:
        // 1. Find the chains which contain differences, push a tuple of which each node pointer to
        //    to a vector.
        // 2. For each chain with a difference, calculate the pair-wise residue differences
        // 3. Store information about the residues with a change in values
        //
        //
        // NOTE: This function should probably be re-written using recursion, since we do the same
        //       for chains and residues, but since it is only two levels deep I didn't bother...
        //
        // ### Time Analysis:
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
        // Let m = be the number of chains
        // Let n = be the average number of residues per chain
        // Let N be the total number of residues
        //
        // As such, N = m * n
        //
        // Time: O(1 + 1 + m + 2 * (m * n))
        //       => O(2mn + m + 2)
        //       => O(2mn + m)
        //
        // As m -> 1 and n -> 1, then O(2mn + m) -> O(3) ~ O(1)
        // As m -> N and n -> 1, then O(2mn + m) -> O(3N) ~ O(N)
        // If m = 1, then O(2n) and n = N, therefore, O(2N) ~ O(N)
        //
        //
        // Time: Best: O(1), Worst: O(N) where N is all residues in the tree
        //
        // Space: O(N)
        //
        let chains = SasaTree::get_raw_node(
            &self.root,
            freesasa_nodetype_FREESASA_NODE_CHAIN,
        );

        // Get the second tree's chains
        // Time: O(1)
        let subtree_chains = SasaTree::get_raw_node(
            &subtree.root,
            freesasa_nodetype_FREESASA_NODE_CHAIN,
        );

        // Find the chains which have different SASA values
        // Time: O(m) where m is the number of chains
        let chain_diffs = SasaTree::predicate_siblings(
            chains,
            subtree_chains,
            predicate,
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
            let chain_id = SasaTree::get_node_name(chain.0);
            let res_node = SasaTree::get_raw_node(
                &chain.0,
                freesasa_nodetype_FREESASA_NODE_RESIDUE,
            );
            let subtree_res_node = SasaTree::get_raw_node(
                &chain.1,
                freesasa_nodetype_FREESASA_NODE_RESIDUE,
            );
            residue_diffs.insert(
                chain_id,
                SasaTree::predicate_siblings(
                    res_node,
                    subtree_res_node,
                    predicate,
                ),
            );
        }

        // Convert the HashMap to vector following FragDB RUID naming scheme
        // (maybe move this to its own function)
        //
        // Time: O(m * n) - Same as above...
        let mut output_vector = Vec::new();
        for chain in residue_diffs {
            let i = chain.1.iter().map(|res| -> ResidueUID {
                let uid = SasaTree::get_node_uid(res.0)
                    .expect("Could not get UID");
                uid
            });
            output_vector.extend(i);
        }

        output_vector
    }

    /// Returns the pairs of nodes which have different SASA values.
    ///
    /// The subtree_node must be a valid subtree of the node, meaning that all nodes in subtree_node
    /// must also be present in node, but not necessarily the other way around.
    ///
    ///
    /// This function only operates on the siblings of the nodes, and does not compare children.
    ///
    ///
    /// ## Arguments
    /// - `node`: The first node to compare
    /// - `subtree_node`: The second node to compare
    ///
    /// ## Returns
    /// A vector of tuples of pointers to the nodes which have different SASA values
    ///
    /// ## Time Complexity
    /// O(n) where n is the number of nodes in the tree
    ///
    /// ## Space Complexity
    /// O(n) where n is the number of nodes in the tree
    fn predicate_siblings(
        node: *mut freesasa_node,
        subtree_node: *mut freesasa_node,
        predicate: fn(f64, f64) -> bool,
    ) -> Vec<(*mut freesasa_node, *mut freesasa_node)> {
        let siblings = SasaTree::get_siblings_as_vector(node, None);
        let subtree_siblings =
            SasaTree::get_siblings_as_hashmap(subtree_node);

        let mut v = Vec::new();

        // Find the chains which have different SASA values
        for sibling in siblings {
            let residue_uid = match SasaTree::get_node_uid(sibling) {
                Ok(uid) => uid,
                Err(e) => {
                    println!("Error: {:?}", e);
                    warn!("Could not get residue UID for node: {:?}. Skipping...", sibling);
                    continue;
                }
            };
            let area = SasaTree::get_node_area(sibling);

            match subtree_siblings.get(&residue_uid) {
                Some((subtree_node, subtree_area)) => {
                    if predicate(area, *subtree_area) {
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
    /// - `other_tree` - The tree to join. Note that the passed in tree's ownership
    ///             moves to this function, and then memory is deallocated.
    pub fn join(
        &self,
        mut other_tree: SasaTree,
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
            warn!("Freesasa returned a warning code when joining result trees!");
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
    /// - `node` - The node to decend from
    /// - `node_type` - The type of node to return at
    ///
    /// ## Returns
    /// A mutable pointer to the matching node or a null pointer if no match was found.
    ///
    fn get_raw_node(
        node: &*mut freesasa_node,
        node_type: freesasa_nodetype,
    ) -> *mut freesasa_node {
        let current_node_type = unsafe { freesasa_node_type(*node) };
        if current_node_type == node_type {
            return *node;
        }

        let node = unsafe { freesasa_node_children(*node) };
        if node.is_null() {
            // Terminate if we have no children (e.g., end of tree)
            return node;
        }

        SasaTree::get_raw_node(&node, node_type) // Then we go deeper!!!
    }

    /// Makes a HashMap with the node names as the keys, and the values tuples of node pointer
    /// and total SASA area.
    ///
    /// Time: O(n) where n is the number of siblings
    fn get_siblings_as_hashmap(node: *mut freesasa_node) -> ResidueMap {
        let mut node = node;
        let mut h = ResidueMap::new();
        while !node.is_null() {
            let area = SasaTree::get_node_area(node);
            let sibling_uid = match SasaTree::get_node_uid(node) {
                Ok(uid) => uid,
                Err(_) => {
                    warn!("Could not get residue UID for node: {:?}. Skipping...", node);

                    node = unsafe { freesasa_node_next(node) };
                    continue;
                }
            };
            if h.insert(sibling_uid, (node, area)).is_some() {
                println!(
                    "WARNING: It appears that multiple siblings have the same name: {:?}",
                    SasaTree::get_node_uid(node)
                );
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
    /// - `node` - The node to find all of the siblings of. If the node is not the first in the
    ///            sequence, only nodes after will be added.
    /// - `capacity` - Optionally can provide a capacity which will be used to pre-allocate the
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

    fn get_node_uid(
        node: *mut freesasa_node,
    ) -> Result<ResidueUID, &'static str> {
        unsafe {
            if freesasa_node_type(node)
                == freesasa_nodetype_FREESASA_NODE_CHAIN
            {
                let chain = freesasa_node_name(node);

                if chain.is_null() {
                    return Err("Chain name is null!");
                }

                let chain = ffi::CStr::from_ptr(chain)
                    .to_str()
                    .map_err(|_| "Chain name is not valid UTF-8!")?
                    .chars()
                    .nth(0)
                    .ok_or("Chain name is empty!")?;

                return Ok(ResidueUID(chain, 0, None));
            } else if freesasa_node_type(node)
                == freesasa_nodetype_FREESASA_NODE_RESIDUE
            {
                let name = freesasa_node_residue_number(node);

                if name.is_null() {
                    return Err("Residue name is null!");
                }

                let chain =
                    freesasa_node_name(freesasa_node_parent(node));

                if chain.is_null() {
                    return Err("Chain name is null!");
                }

                let chain = ffi::CStr::from_ptr(chain)
                    .to_str()
                    .map_err(|_| "Chain name is not valid UTF-8!")?
                    .chars()
                    .nth(0)
                    .ok_or("Chain name is empty!")?;

                let name = ffi::CStr::from_ptr(name);
                let mut name = name
                    .to_str()
                    .map_err(|_| "Residue name is not valid UTF-8!")?;

                let inscode = {
                    if name.chars().last().unwrap().is_alphabetic() {
                        name = &name[..name.len() - 1];
                        Some(name.chars().last().unwrap())
                    } else {
                        None
                    }
                };

                let name =
                    name.trim().parse::<i32>().map_err(|_| {
                        println!("Residue name:{}", name);
                        "Residue name is not a number!"
                    })?;

                Ok(ResidueUID(chain, name, inscode))
            } else {
                Err("The node type is not a chain or residue!")
            }
        }
    }

    /// Returns the total area of the node as a f64
    fn get_node_area(node: *mut freesasa_node) -> f64 {
        unsafe { (*freesasa_node_area(node)).total }
    }
}

impl Drop for SasaTree {
    fn drop(&mut self) {
        unsafe {
            if self.root.is_null() {
                trace!(
                    "SasaTree::drop() - root is null, nothing to free"
                );
                return; // Do need to free if null, tree probably was moved
            }
            freesasa_node_free(self.root);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::structure;

    use super::*;

    #[test]
    fn new() {
        if SasaTree::new(std::ptr::null_mut()).is_ok() {
            panic!("Created SasaTree with")
        }
    }

    #[test]
    fn get_node() {
        let pdb = structure::Structure::from_path(
            "data/single_chain.pdb",
            None,
        )
        .unwrap();
        let tree = pdb.calculate_sasa_tree().unwrap();

        let node = SasaTree::get_raw_node(
            &tree.root,
            freesasa_nodetype_FREESASA_NODE_CHAIN,
        );
        let name = SasaTree::get_node_name(node);
        assert_eq!(name, "A");
    }

    #[test]
    fn get_siblings_as_hashmap() {
        let pdb = structure::Structure::from_path(
            "data/single_chain.pdb",
            None,
        )
        .unwrap();
        let tree = pdb.calculate_sasa_tree().unwrap();

        let node = SasaTree::get_raw_node(
            &tree.root,
            freesasa_nodetype_FREESASA_NODE_CHAIN,
        );
        let siblings = SasaTree::get_siblings_as_hashmap(node);

        assert_eq!(siblings.len(), 1);
    }

    #[test]
    fn test_compare() {
        let pdb = structure::Structure::from_path(
            "data/single_chain.pdb",
            None,
        )
        .unwrap();

        let sub_pdb = structure::Structure::from_path(
            "data/single_chain_w_del.pdb",
            None,
        );

        let tree = pdb.calculate_sasa_tree().unwrap();
        let sub_tree = sub_pdb.unwrap().calculate_sasa_tree().unwrap();

        let diff = tree.compare_residues(&sub_tree, |c, o| {
            if o - c > 0.0 {
                true
            } else {
                false
            }
        });

        for uid in diff {
            println!("{}", uid);
        }

        let pdb_7trr =
            structure::Structure::from_path("data/7trr.pdb", None)
                .unwrap();
        // 7trr_gap_141_156_inc.pdb is a subset of 7trr.pdb, with residues 141-156 removed
        let pdb_7trr_sub = structure::Structure::from_path(
            "data/7trr_gap_141_156_inc.pdb",
            None,
        )
        .unwrap();

        let tree_7trr = pdb_7trr.calculate_sasa_tree().unwrap();
        let tree_7trr_sub = pdb_7trr_sub.calculate_sasa_tree().unwrap();

        let sasa_7trr = pdb_7trr.calculate_sasa().unwrap();
        let sasa_7trr_sub = pdb_7trr_sub.calculate_sasa().unwrap();

        println!(
            "7trr: {} 7trr_sub: {}",
            sasa_7trr.total, sasa_7trr_sub.total
        );

        let diff =
            tree_7trr.compare_residues(&tree_7trr_sub, |c, o| {
                if o - c > 0.0 {
                    true
                } else {
                    false
                }
            });

        println!("Diff: {:?}", diff);
    }
}

// Todo: work out why this isn't working
// Todo: Add methods for displaying the tree for debugginh
// todo: Tidy up the code
//       - Function out some large chunks of code
//       - Add comments
// todo: add utils:
//
//       - get residues -> Vec<Node> or maybe Linked list of nodes
//       - get chains
//       - get atoms
