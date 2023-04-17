use std::ffi::CStr;

use freesasa_sys::{
    freesasa_node, freesasa_node_atom_is_mainchain,
    freesasa_node_atom_is_polar, freesasa_node_atom_radius,
    freesasa_node_chain_n_residues, freesasa_node_classified_by,
    freesasa_node_name, freesasa_node_parent,
    freesasa_node_residue_n_atoms, freesasa_node_residue_number,
    freesasa_node_structure_model, freesasa_node_structure_n_atoms,
};

use crate::utils::assert_nodetype;

use super::NodeType;

// TODO Use references to parents propeties, save memorys
// maybe use shared pointers and some sort of hash map
// to keep track of the parents

#[derive(Debug, Clone, serde::Serialize)]
pub struct AtomProperties {
    pub is_polar: bool, // Polar
    pub is_bb: bool,    // Is backbone
    pub radius: f64,    // Atomic radius
}

impl AtomProperties {
    pub(super) fn new(node: &*mut freesasa_node) -> Self {
        assert_nodetype(node, NodeType::Atom);

        let name = unsafe { freesasa_node_name(*node) };
        if name.is_null() {
            panic!("Invalid atom name");
        }

        let radius = unsafe { freesasa_node_atom_radius(*node) };

        let is_polar =
            unsafe { freesasa_node_atom_is_polar(*node) == 1 };

        let is_bb =
            unsafe { freesasa_node_atom_is_mainchain(*node) == 1 };

        Self {
            is_polar,
            is_bb,
            radius,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResidueProperties {
    pub n_atoms: i32,    // Number of atoms
    pub resname: String, // Residue name
}

impl ResidueProperties {
    pub(super) fn new(node: &*mut freesasa_node) -> Self {
        assert_nodetype(node, NodeType::Residue);

        let name = unsafe { freesasa_node_residue_number(*node) };
        if name.is_null() {
            panic!("Invalid residue number");
        }

        let name = unsafe {
            CStr::from_ptr(name)
                .to_str()
                .expect("Residue number containted invalid UTF-8 bytes")
                .trim()
                .to_owned()
        };

        // Check if the last character is an insertion code (e.g., non-numeric)

        let (resnum, inscode) =
            if name.chars().last().unwrap().is_numeric() {
                (name, None)
            } else {
                let resnum = name[..name.len() - 1].to_string();
                let inscode = name.chars().last().unwrap();
                (resnum, Some(inscode))
            };

        #[cfg(debug_assertions)]
        {
            trace!("Residue number: {}", resnum);
            if let Some(inscode) = inscode {
                println!("Insertion code: {}", inscode);
            }

            trace!("Residue name: {}", unsafe {
                CStr::from_ptr(freesasa_node_name(*node))
                    .to_str()
                    .unwrap()
            });
        }

        ResidueProperties {
            n_atoms: unsafe { freesasa_node_residue_n_atoms(*node) },
            resname: unsafe {
                let name = freesasa_node_name(*node);
                if name.is_null() {
                    panic!("Invalid residue name");
                }

                let name = CStr::from_ptr(name);

                name.to_str().unwrap().to_string()
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChainProperties {
    pub n_residues: i32, // Number of residues
    pub id: char,        // Chain name
    pub structure: i32,  // Structure name (model number)
}

impl ChainProperties {
    pub(super) fn new(node: &*mut freesasa_node) -> Self {
        assert_nodetype(node, NodeType::Chain);

        ChainProperties {
            n_residues: unsafe {
                freesasa_node_chain_n_residues(*node)
            },
            id: unsafe {
                let name = freesasa_node_name(*node);
                if name.is_null() {
                    panic!("Invalid chain name");
                }

                let name = CStr::from_ptr(name);

                if name.to_bytes().len() != 1 {
                    panic!("Invalid chain name");
                }

                name.to_bytes()[0] as char
            },
            structure: unsafe {
                let structure_ptr = freesasa_node_parent(*node);

                #[cfg(debug_assertions)]
                assert_nodetype(&structure_ptr, NodeType::Structure);

                if structure_ptr.is_null() {
                    panic!("Invalid parent node");
                }
                freesasa_node_structure_model(structure_ptr)
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StructureProperties {
    pub n_atoms: i32, // Number of atoms
}

impl StructureProperties {
    pub(super) fn new(node: &*mut freesasa_node) -> Self {
        assert_nodetype(node, NodeType::Structure);

        let n_atoms = unsafe { freesasa_node_structure_n_atoms(*node) };

        StructureProperties { n_atoms }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResultProperties {
    pub classified_by: String, // Classification method
}

impl ResultProperties {
    pub(super) fn new(result: &*mut freesasa_node) -> Self {
        let classified_by = unsafe {
            let method = freesasa_node_classified_by(*result);
            if method.is_null() {
                panic!("Invalid classification method");
            }

            let method = CStr::from_ptr(method);

            method.to_str().unwrap().to_string()
        };

        ResultProperties { classified_by }
    }
}
