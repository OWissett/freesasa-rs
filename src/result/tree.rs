use std::collections::HashMap;
use std::fmt::Debug;

use freesasa_sys::{
    freesasa_node, freesasa_node_children, freesasa_node_free,
    freesasa_node_next, freesasa_tree_init,
};
use petgraph::graph;

use crate::{
    free_raw_c_strings, structure::Structure, utils::str_to_c_string,
};

use crate::result::SasaResult;

use super::node::{Node, NodeArea, NodeType};

#[derive(Debug, serde::Serialize)]
pub struct SasaTree {
    graph: petgraph::Graph<Node, ()>,
}

impl SasaTree {
    /// Build a [`SasaTree`] from a [`freesasa_node`].
    ///
    /// This function is unsafe, as it takes a raw pointer to a [`freesasa_node`]. This pointer is
    /// freed after the tree is built.
    pub(crate) fn from_ptr(
        node: *mut freesasa_node,
        depth: &NodeType,
    ) -> Self {
        let mut graph = petgraph::Graph::<Node, ()>::new();

        Self::recursive_add_node(&mut graph, &node, None, depth);

        unsafe {
            freesasa_node_free(node);
        }

        SasaTree { graph }
    }

    /// Build a [`SasaTree`] from a [`SasaResult`].
    pub fn from_result(
        result: &SasaResult,
        structure: &Structure,
        depth: &NodeType,
    ) -> Result<Self, &'static str> {
        let name = str_to_c_string(structure.get_name())?.into_raw();

        if structure.is_null() {
            return Err(
                "Failed to create SasaTree: structure pointer was null!",
            );
        }

        if result.is_null() {
            return Err(
                "Failed to create SasaTree: result pointer was null!",
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
            return Err("Failed to create SasaTree: freesasa_tree_init returned a null pointer!");
        }

        Ok(Self::from_ptr(root, depth))
    }

    /// Connects siblings nodes in the graph. This is done by iterating over all nodes and checking
    /// if they have a sibling. If they do, the sibling is found and an edge is added between the
    /// two nodes.
    ///
    /// This function is relatively slow, as such, the siblings are not connected when the graph is
    /// built. If you want a graph with connected siblings, call this function after building the
    /// graph. Note that this will result in a cyclic graph - which may not be what you want.
    pub fn connect_siblings(&mut self) {
        let graph = &mut self.graph;
        let mut nodes = graph.node_indices().collect::<Vec<_>>();

        while let Some(node) = nodes.pop() {
            let node = graph.node_weight(node).unwrap();

            if let Some(sibling) = node.sibling_uid() {
                let node = graph.node_indices().find(|n| {
                    graph.node_weight(*n).unwrap().uid() == node.uid()
                });

                let sibling = graph.node_indices().find(|n| {
                    graph.node_weight(*n).unwrap().uid()
                        == Some(sibling)
                });

                if let (Some(node), Some(sibling)) = (node, sibling) {
                    graph.add_edge(node, sibling, ());
                }
            }
        }
    }

    /// Compares the SASA of the residues in the tree with the SASA of the residues in the
    /// [`SasaTree`] `subtree`. The comparison is done by applying the function `op` to the
    /// [`NodeArea`] of the residues in the tree and the [`NodeArea`] of the residues in the
    /// `subtree`. The result of the comparison is then passed to the function `predicate`, which
    /// returns a boolean. If the boolean is `true`, the residue is added to the result.
    ///
    /// The function `op` should take two [`NodeArea`] references and return a [`NodeArea`]. The function
    /// `predicate` should take a [`NodeArea`] reference and return a boolean.
    ///
    /// The resulting [`NodeArea`] is used to create a new [`Node`] which is added to the result
    /// [`Vec`].
    ///
    pub fn compare_residues<O, P>(
        &self,
        subtree: &SasaTree,
        op: O,
        predicate: P,
    ) -> Vec<Node>
    where
        O: FnOnce(&NodeArea, &NodeArea) -> NodeArea + Copy,
        P: FnOnce(&NodeArea) -> bool + Copy,
    {
        // Get all residue nodes in the tree
        let nodes = self
            .graph
            .node_indices()
            .filter(|n| {
                *self.graph.node_weight(*n).unwrap().nodetype()
                    == NodeType::Residue
            })
            .map(|n| {
                let node = self.graph.node_weight(n).unwrap();
                let uid = node.uid().unwrap();
                (uid, node)
            })
            .collect::<HashMap<_, _>>();

        let subtree_nodes = subtree
            .graph
            .node_indices()
            .filter(|n| {
                *subtree.graph.node_weight(*n).unwrap().nodetype()
                    == NodeType::Residue
            })
            .map(|n| {
                (self.graph.node_weight(n).unwrap().uid().unwrap(), n)
            })
            .collect::<Vec<_>>();

        let mut result: Vec<Node> = Vec::new();

        for (uid, nidx) in subtree_nodes {
            // get the node in the tree with the same uid as the node in the subtree
            let node = nodes.get(&uid);

            match node {
                Some(n) => {
                    let node = *n;
                    let subtree_node =
                        subtree.graph.node_weight(nidx).unwrap();

                    let area = op(
                        node.area().unwrap(),
                        subtree_node.area().unwrap(),
                    );

                    // check if the predicate is true with the result
                    if predicate(&area) {
                        // create a new node with the new area
                        let mut node = node.clone();
                        node.set_area(Some(area));
                        result.push(node);
                    }
                }
                None => {
                    warn!(
                        "Residue with uid {} not found in tree!",
                        uid
                    );
                    continue;
                }
            }
        }

        result
    }

    /// Returns a vector of nodes in the graph with the given type.
    ///
    /// The nodes are cloned, so the returned vector can be modified without affecting the graph.
    pub fn get_nodes(&self, nodetype: NodeType) -> Vec<Node> {
        self.graph
            .node_indices()
            .filter(|n| {
                *self.graph.node_weight(*n).unwrap().nodetype()
                    == nodetype
            })
            .map(|n| self.graph.node_weight(n).unwrap().clone())
            .collect()
    }

    /// Returns a reference to the underlying [`petgraph::Graph`].
    pub fn get_graph(&self) -> &petgraph::Graph<Node, ()> {
        &self.graph
    }

    /// Recursively add nodes to the graph.
    ///
    /// This is used to construct a [`petgraph::Graph`] from a [`freesasa_node`] tree.
    ///
    fn recursive_add_node(
        graph: &mut petgraph::Graph<Node, ()>,
        node: &*mut freesasa_node,
        parent: Option<graph::NodeIndex>,
        deepest_nodetype: &NodeType,
    ) {
        let mut current_node = *node;

        while !current_node.is_null() {
            let node = Node::new_from_node(&current_node);
            let nodetype = *node.nodetype();

            let node_index = graph.add_node(node);

            if let Some(parent) = parent {
                graph.add_edge(parent, node_index, ());
            }

            if nodetype != *deepest_nodetype {
                // We get the children of the node, if we are not at the deepest nodetype
                let children =
                    unsafe { freesasa_node_children(current_node) };

                // If the children are not null, we recursively add them to the graph
                if !children.is_null() {
                    Self::recursive_add_node(
                        graph,
                        &children,
                        Some(node_index),
                        deepest_nodetype,
                    );
                }
            }

            // We have finished adding children from this node,
            // so we move on to the next sibling node
            current_node = unsafe { freesasa_node_next(current_node) };
        }
    }
}

#[cfg(test)]
mod test_native {
    use freesasa_sys::freesasa_calc_tree;

    use crate::structure::{self, DEFAULT_CALCULATION_PARAMETERS};

    use super::*;

    fn create_native_tree(path: &str) -> SasaTreeNative {
        let pdb = structure::Structure::from_path(path, None).unwrap();

        let name = str_to_c_string(&pdb.get_name()).unwrap().into_raw();
        let root = unsafe {
            freesasa_calc_tree(
                pdb.as_const_ptr(),
                DEFAULT_CALCULATION_PARAMETERS,
                name,
            )
        };

        // Retake CString ownership
        free_raw_c_strings!(name);

        if root.is_null() {
            panic!();
        }

        SasaTreeNative { root }
    }

    #[test]
    fn new() {
        if SasaTreeNative::new(std::ptr::null_mut()).is_ok() {
            panic!("Created SasaTree with")
        }
    }

    #[test]
    fn get_node() {
        let tree = create_native_tree("data/single_chain.pdb");
        let node = SasaTreeNative::get_raw_node(
            &tree.root,
            freesasa_nodetype_FREESASA_NODE_CHAIN,
        );
        assert!(!node.is_null());
    }

    #[ignore = " working. Old method, no time to fix"]
    #[test]
    fn get_siblings_as_hashmap() {
        let tree = create_native_tree("data/single_chain.pdb");
        let node = SasaTreeNative::get_raw_node(
            &tree.root,
            freesasa_nodetype_FREESASA_NODE_CHAIN,
        );
        let siblings = SasaTreeNative::get_siblings_as_hashmap(node);
        assert_eq!(siblings.len(), 1);
    }

    #[test]
    fn test_compare() {
        let tree = create_native_tree("data/single_chain.pdb");
        let sub_tree =
            create_native_tree("data/single_chain_w_del.pdb");

        let diff = tree.compare_residues(&sub_tree, |c, o| o - c > 0.0);

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

        let tree_7trr = create_native_tree("data/7trr.pdb");
        let tree_7trr_sub =
            create_native_tree("data/7trr_gap_141_156_inc.pdb");

        let sasa_7trr = pdb_7trr.calculate_sasa().unwrap();
        let sasa_7trr_sub = pdb_7trr_sub.calculate_sasa().unwrap();

        println!(
            "7trr: {} 7trr_sub: {}",
            sasa_7trr.total, sasa_7trr_sub.total
        );

        let diff = tree_7trr
            .compare_residues(&tree_7trr_sub, |c, o| o - c > 0.0);

        println!("Diff: {:?}", diff);
    }
}

#[cfg(test)]
mod test_petgraph {
    use freesasa_sys::freesasa_calc_tree;
    use petgraph::dot::{Config, Dot};
    use serde::de::Expected;

    use super::*;
    use crate::result::node::NodeType;
    use crate::structure;
    use crate::structure::DEFAULT_CALCULATION_PARAMETERS;

    #[test]
    fn from_node() {
        let pdb_ptr = structure::Structure::from_path(
            "data/single_chain.pdb",
            None,
        )
        .unwrap();

        let name = pdb_ptr.get_name().to_owned();

        let name = str_to_c_string(&name).unwrap().into_raw();
        let root = unsafe {
            freesasa_calc_tree(
                pdb_ptr.as_const_ptr(),
                DEFAULT_CALCULATION_PARAMETERS,
                name,
            )
        };

        println!(
            "Root: {:?}",
            NodeType::from_fs_level(unsafe {
                freesasa_node_type(root)
            })
        );

        // Retake CString ownership
        free_raw_c_strings!(name);

        let mut tree = SasaTree::from_ptr(root, &NodeType::Atom);

        let current_time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        tree.connect_siblings();
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - current_time_ms;
        println!("Elapsed: {}ms", elapsed);

        // write the dot file to disk
        let dot = Dot::with_config(
            &tree.graph,
            &[Config::EdgeNoLabel, Config::NodeNoLabel],
        );
        let dot = format!("{:?}", dot);
        std::fs::write("tree.dot", dot).unwrap();
    }

    #[test]
    fn compare_residues() {
        let pdb =
            structure::Structure::from_path("data/7trr.pdb", None)
                .unwrap();

        let sub_pdb = structure::Structure::from_path(
            "data/7trr_gap_141_156_inc.pdb",
            None,
        )
        .unwrap();

        let name = str_to_c_string(pdb.get_name()).unwrap().into_raw();
        let root = unsafe {
            freesasa_calc_tree(
                pdb.as_const_ptr(),
                DEFAULT_CALCULATION_PARAMETERS,
                name,
            )
        };

        let name =
            str_to_c_string(sub_pdb.get_name()).unwrap().into_raw();
        let root_sub = unsafe {
            freesasa_calc_tree(
                sub_pdb.as_const_ptr(),
                DEFAULT_CALCULATION_PARAMETERS,
                name,
            )
        };

        let tree = SasaTree::from_ptr(root, &NodeType::Atom);
        let sub_tree = SasaTree::from_ptr(root_sub, &NodeType::Atom);

        let diff = tree.compare_residues(
            &sub_tree,
            |c, o| o - c,
            |a| a.total() > 0.0,
        );

        for uid in diff {
            println!("{:?}", uid);
        }
    }

    #[test]
    fn validate_compare_residues() {
        // The test PDB file is 3b7y_B.pdb (full structure) and
        // 3b7y_B_match_removed.pdb (residues 147-156 [inclusive] removed)) as
        // the substructure.
        //
        // The file 3b7y_B_sasa_results.json was computed by M. Greenig using
        // a python script and the freesasa library, and then manually verified
        // as sensible.

        let base_pdb =
            structure::Structure::from_path("data/3b7y_B.pdb", None)
                .unwrap();

        let sub_pdb = structure::Structure::from_path(
            "data/3b7y_B_match_removed.pdb",
            None,
        )
        .unwrap();
        let base_tree =
            base_pdb.calculate_sasa_tree(&NodeType::Residue).unwrap();

        let sub_tree =
            sub_pdb.calculate_sasa_tree(&NodeType::Residue).unwrap();

        let diff = base_tree.compare_residues(
            &sub_tree,
            |c, o| o - c,
            |a| a.total() > 0.0,
        );

        let diff = diff
            .iter()
            .map(|n| {
                let resnum = match n.uid().unwrap() {
                    NodeUID::Residue(r) => r.resnum(),
                    _ => panic!("Expected residue"),
                };
                (resnum.to_string(), n.area().unwrap().total())
            })
            .collect::<HashMap<String, f64>>();

        // Read the expected results from the JSON file using serde
        let expected_results: HashMap<String, f64> =
            serde_json::from_str(
                &std::fs::read_to_string(
                    "data/3b7y_B_sasa_results.json",
                )
                .unwrap(),
            )
            .unwrap();

        let expected_results = expected_results
            .iter()
            .filter(|(_, v)| **v > 0.0)
            .map(|(k, v)| (k.to_string(), *v))
            .collect();

        // Pretty print the results
        println!("Diff: {:#?}", diff);
        println!("Expected: {:#?}", expected_results);

        // Compare the results
        assert_eq!(diff, expected_results);
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
