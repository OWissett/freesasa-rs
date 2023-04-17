use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;

use freesasa_sys::{
    freesasa_node, freesasa_node_children, freesasa_node_free,
    freesasa_node_next, freesasa_tree_init,
};
use serde_with::{serde_as, DisplayFromStr};

use crate::uids::NodeUid;
use crate::{
    free_raw_c_strings, structure::Structure, utils::str_to_c_string,
};

use crate::result::SasaResult;

use super::node::{Node, NodeArea, NodeType};

#[serde_as]
#[derive(Debug, serde::Serialize)]
pub struct SasaTree {
    /// Stores the data of the current node.
    #[serde(flatten)]
    node: Node,

    /// Stores the children of the current node.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<HashMap<DisplayFromStr, _>>")]
    children: Option<HashMap<NodeUid, SasaTree>>,
}

impl SasaTree {
    // ------------ //
    // Construction //
    // ------------ //

    /// Creates a new [`SasaTree`] from a [`freesasa_node`] pointer to an
    /// underlying C object.
    ///
    /// It is assumed that the tree contains a single structure node. If
    /// this is not the case, only the first structure node will be used.
    pub(crate) fn new(
        c_node: *mut freesasa_node,
        depth: &NodeType,
    ) -> Self {
        let mut structure_ptr = c_node;

        while NodeType::nodetype_of_ptr(structure_ptr)
            != NodeType::Structure
        {
            structure_ptr =
                unsafe { freesasa_node_children(structure_ptr) };
        }

        let mut root = Self {
            node: unsafe { Node::from_ptr(structure_ptr) },
            children: None,
        };

        Self::recursive_build(&mut root, structure_ptr, depth);

        trace!("SasaTree::new(): Freeing C node pointer {:p}", c_node);
        unsafe { freesasa_node_free(c_node) };

        root
    }

    /// Creates a new [`SasaTree`] from a [`SasaResult`].
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

        Ok(Self::new(root, depth))
    }

    /// Depth-first recursive build of the tree.
    fn recursive_build(
        root: &mut SasaTree,
        c_node: *mut freesasa_node,
        depth: &NodeType,
    ) {
        // Get the children of the current node,
        // and add them to the tree.

        // Then recursively call this function on each child.

        let mut children = VecDeque::new();

        let mut child_ptr = unsafe { freesasa_node_children(c_node) };

        while !child_ptr.is_null() {
            children.push_back(child_ptr);
            child_ptr = unsafe { freesasa_node_next(child_ptr) };
        }

        if children.is_empty() {
            return;
        }

        let mut children_map = HashMap::new();

        for child in children {
            let child_node = unsafe { Node::from_ptr(child) };

            if child_node.nodetype() == depth {
                children_map.insert(
                    child_node.uid().unwrap().to_owned(),
                    SasaTree {
                        node: child_node,
                        children: None,
                    },
                );
            } else {
                let mut child_tree = SasaTree {
                    node: child_node,
                    children: None,
                };

                Self::recursive_build(&mut child_tree, child, depth);

                children_map.insert(
                    child_tree.node.uid().unwrap().to_owned(),
                    child_tree,
                );
            }
        }

        root.children = Some(children_map);
    }

    // ------- //
    // Compute //
    // ------- //

    /// Compares nodes at the given depth between two trees and returns a
    /// vector of nodes that are different, where the differences are stored
    /// in the `area` field.
    ///
    /// ### Arguments
    /// - `other`: The other tree to compare to.
    /// - `node_filter`: The type of nodes to compare.
    /// - `op`: The operation to perform on the two nodes' areas.
    /// - `predicate`: The predicate to test the result of the operation. This is
    /// typically a comparison operator.
    ///
    /// ### Panics
    /// - If the `node_filter` is a non-area node type, such as `NodeType::Root` or
    pub fn predicate_trees<O, P>(
        &self,
        other: &Self,
        node_filter: &NodeType,
        op: O,
        predicate: P,
    ) -> Vec<Node>
    where
        O: FnOnce(&NodeArea, &NodeArea) -> NodeArea + Copy,
        P: FnOnce(&NodeArea) -> bool + Copy,
    {
        // Create a HashMap of the nodes in the other tree
        let other_nodes = other
            .nodes()
            .filter(|node| node.nodetype() == node_filter)
            .fold(HashMap::new(), |mut map, node| {
                map.insert(
                    node.uid().unwrap().to_owned(),
                    node.to_owned(),
                );
                map
            });

        let mut differences = Vec::new();

        for node in
            self.nodes().filter(|node| node.nodetype() == node_filter)
        {
            if let Some(other_node) =
                other_nodes.get(node.uid().unwrap())
            {
                if node.area().is_none() || other_node.area().is_none()
                {
                    continue;
                }

                let area = op(
                    node.area().unwrap(),
                    other_node.area().unwrap(),
                );

                if predicate(&area) {
                    differences.push(Node::new(
                        node.nodetype().to_owned(),
                        None,
                        Some(area),
                        node.uid().map(|uid| uid.to_owned()),
                    ));
                }
            }
        }

        differences
    }

    // --------- //
    // Accessors //
    // --------- //

    /// Returns the [`Node`] of the current node.
    pub fn node(&self) -> &Node {
        &self.node
    }

    pub fn child_map(&self) -> &Option<HashMap<NodeUid, SasaTree>> {
        &self.children
    }

    /// Provides an iterator over the nodes in the tree.
    pub fn nodes<'a>(&'a self) -> Box<dyn Iterator<Item = &Node> + 'a> {
        // We want to flatten the tree into a Vec of nodes, so we need to
        // traverse the tree in a breadth-first manner. We use a VecDeque
        // to store the nodes we need to visit, and a Vec to store the
        // nodes we have visited.

        let mut nodes_to_visit = VecDeque::new();
        let mut visited_nodes = Vec::new();

        nodes_to_visit.push_back(self);

        while let Some(node) = nodes_to_visit.pop_front() {
            visited_nodes.push(node.node());

            if let Some(children) = &node.children {
                for child in children.values() {
                    nodes_to_visit.push_back(child);
                }
            }
        }

        Box::new(visited_nodes.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::node::NodeType;
    use crate::structure;

    #[test]
    fn test_sasa_tree_from_result() {
        let pdb =
            structure::Structure::from_path("data/3b7y_matt.pdb", None)
                .unwrap();

        let result = pdb.calculate_sasa().unwrap();

        let tree =
            SasaTree::from_result(&result, &pdb, &NodeType::Residue)
                .unwrap();

        // Check that the tree has the correct number of layers
        assert_eq!(tree.node.nodetype(), &NodeType::Structure);
        assert_eq!(tree.children.as_ref().unwrap().len(), 2); // chains

        let chain_a = tree
            .children
            .as_ref()
            .unwrap()
            .get(&NodeUid::new('A', None, None))
            .unwrap();

        assert_eq!(chain_a.node.nodetype(), &NodeType::Chain);

        let chain_b = tree
            .children
            .as_ref()
            .unwrap()
            .get(&NodeUid::new('B', None, None))
            .unwrap();

        assert_eq!(chain_b.node.nodetype(), &NodeType::Chain);

        assert_eq!(chain_a.children.as_ref().unwrap().len(), 144); // residues
        assert_eq!(chain_b.children.as_ref().unwrap().len(), 146); // residues

        // load the expected tree from a JSON file
        let expected_tree: HashMap<String, HashMap<String, f64>> =
            serde_json::from_str(
                &std::fs::read_to_string("data/3b7y_matt_sasa.json")
                    .unwrap(),
            )
            .unwrap();

        // Flatten keys
        let expected_tree = expected_tree
            .into_iter()
            .flat_map(|(chain_key, value)| {
                value.into_iter().map(move |(res_key, value)| {
                    ((chain_key.clone(), res_key), value)
                })
            })
            .collect::<HashMap<(String, String), f64>>();

        let nodes = tree
            .nodes()
            .filter(|node| node.nodetype() == &NodeType::Residue);

        // Varify the calculations against values calculated using Python
        for node in nodes {
            let res_id = node.uid().unwrap();
            let sasa = node.area().unwrap().total();

            let diff = sasa
                - expected_tree
                    .get(&(
                        res_id.chain().to_string(),
                        res_id.res_id().unwrap().0.to_string(),
                    ))
                    .unwrap_or_else(|| {
                        panic!(
                            "No expected value for residue {}",
                            res_id
                        )
                    });

            // Allow a small difference since we seem to get
            // slightly different results, I think it is because
            // of floating point rounding errors.
            if diff.abs() > 0.0001 {
                panic!(
                    "SASA for residue {} is {}, expected {}",
                    res_id,
                    sasa,
                    expected_tree[&(
                        res_id.chain().to_string(),
                        res_id.res_id().unwrap().0.to_string()
                    )]
                );
            }
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
            structure::Structure::from_path("data/3b7y_matt.pdb", None)
                .unwrap();

        let sub_pdb = structure::Structure::from_path(
            "data/3b7y_matt_match_removed.pdb",
            None,
        )
        .unwrap();
        let base_tree =
            base_pdb.calculate_sasa_tree(&NodeType::Residue).unwrap();

        let sub_tree =
            sub_pdb.calculate_sasa_tree(&NodeType::Residue).unwrap();

        // Compute the difference between the two trees
        let diffs = base_tree
            .predicate_trees(
                &sub_tree,
                &NodeType::Residue,
                |s, o| o - s,
                |area| area.total() > 0.0,
            )
            .iter()
            .map(|node| {
                let res_id = match node.uid().unwrap().res_id() {
                    Some(res_id) => res_id,
                    _ => panic!("NodeUID was not a ResidueUID!"),
                };
                let sasa = node.area().unwrap().total();

                (res_id.0.to_string(), sasa)
            })
            .collect::<HashMap<_, _>>();

        //// Read the expected results from the JSON file using serde
        let expected_results: HashMap<String, f64> =
            serde_json::from_str(
                &std::fs::read_to_string("data/3b7y_B_sasa_diffs.json")
                    .unwrap(),
            )
            .unwrap();

        // Remove expected_results which are zero
        let expected_results: HashMap<String, f64> = expected_results
            .into_iter()
            .filter(|(_, sasa)| *sasa > 0.0)
            .collect();

        println!("Diffs has {} ", diffs.len());
        println!("Expected has {} ", expected_results.len());

        println!("Diffs: {:#?}", diffs);
        println!("Expected: {:#?}", expected_results);
    }

    #[test]
    fn test_serialise() {
        let base_pdb =
            structure::Structure::from_path("data/3b7y_matt.pdb", None)
                .unwrap();

        let base_tree =
            base_pdb.calculate_sasa_tree(&NodeType::Residue).unwrap();

        let _ = serde_json::to_string(&base_tree).unwrap();
    }
}
