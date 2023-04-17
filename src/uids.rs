use std::ffi;
use std::fmt::Display;

use freesasa_sys::{
    freesasa_node, freesasa_node_name, freesasa_node_parent,
    freesasa_node_residue_number, freesasa_node_structure_model,
};

use crate::{result::node::NodeType, utils::assert_nodetype};

/// ID for a residue, which is a tuple of the residue number and insertion code.
type ResID = (i32, Option<char>);

// Structure, chain, residue, atom
type UidPrimitive = (i32, Option<char>, Option<ResID>, Option<String>);

/// Unique ID for a structure node (e.g. a chain, residue, atom, etc.).
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct NodeUid {
    // NOTE: The ordering of the fields is important, because it determines the
    // precedence of the fields in derive `Ord` and `PartialOrd` implementations.
    //
    /// This is technically the model number, but it's usually just 1.
    /// The node name of the structure node in `freesasa-sys` is the
    /// concatenation of the chain IDs, so we don't really need this.
    structure: i32,

    /// Chain ID.
    chain: Option<char>,

    /// Residue ID - tuple of residue number and insertion code.
    res_id: Option<ResID>,

    /// Atom name - Such as "CA" or "O".
    atom_name: Option<String>,
}

impl NodeUid {
    pub fn new(
        structure: i32,
        chain: Option<char>,
        res_id: Option<ResID>,
        atom_name: Option<String>,
    ) -> Self {
        Self {
            structure,
            chain,
            res_id,
            atom_name,
        }
    }

    // Create a new `NodeUid` from a `UidPrimitive`.
    fn from_primitive(
        (structure, chain, res_id, atom_name): UidPrimitive,
    ) -> Self {
        #[cfg(debug_assertions)]
        {
            if atom_name.is_some() {
                assert!(
                    res_id.is_some(),
                    "Atom name provided without residue ID"
                );
                assert!(
                    chain.is_some(),
                    "Atom name provided without chain ID"
                );
            }

            if res_id.is_some() {
                assert!(
                    chain.is_some(),
                    "Residue ID provided without chain ID"
                );
            }
        }

        Self {
            structure,
            chain,
            res_id,
            atom_name,
        }
    }

    pub(crate) fn from_ptr(node: *mut freesasa_node) -> Option<Self> {
        let node_type = NodeType::nodetype_of_ptr(node);

        match node_type {
            NodeType::Structure => Some(Self::from_primitive(
                Self::from_structure_ptr(node),
            )),
            NodeType::Chain => {
                Some(Self::from_primitive(Self::from_chain_ptr(node)))
            }
            NodeType::Residue => {
                Some(Self::from_primitive(Self::from_residue_ptr(node)))
            }
            NodeType::Atom => {
                Some(Self::from_primitive(Self::from_atom_ptr(node)))
            }
            NodeType::None => None,
            NodeType::Result => None,
            NodeType::Root => None,
        }
    }

    pub fn structure(&self) -> i32 {
        self.structure
    }

    pub fn chain(&self) -> Option<&char> {
        self.chain.as_ref()
    }

    pub fn res_id(&self) -> Option<&ResID> {
        self.res_id.as_ref()
    }

    pub fn atom_name(&self) -> Option<&str> {
        self.atom_name.as_deref()
    }

    fn from_structure_ptr(node: *mut freesasa_node) -> UidPrimitive {
        #[cfg(debug_assertions)]
        assert_nodetype(&node, NodeType::Structure);

        let structure = unsafe { freesasa_node_structure_model(node) };

        (structure, None, None, None)
    }

    fn from_chain_ptr(node: *mut freesasa_node) -> UidPrimitive {
        #[cfg(debug_assertions)]
        assert_nodetype(&node, NodeType::Chain);

        let structure = unsafe {
            freesasa_node_structure_model(freesasa_node_parent(node))
        };
        let chain = unsafe { freesasa_node_name(node) };

        // convert from c-style string to String
        let chain = unsafe {
            ffi::CStr::from_ptr(chain).to_str().unwrap().chars().next()
        };

        #[cfg(debug_assertions)]
        {
            assert!(
                chain.is_some(),
                "Chain ID is None, but node type is Chain"
            );
        }

        (structure, chain, None, None)
    }

    fn from_residue_ptr(node: *mut freesasa_node) -> UidPrimitive {
        #[cfg(debug_assertions)]
        assert_nodetype(&node, NodeType::Residue);

        let chain_ptr = unsafe { freesasa_node_parent(node) };

        let mut uid = Self::from_chain_ptr(chain_ptr);

        let res_id = unsafe { freesasa_node_residue_number(node) };

        // convert from c-style string to String
        let res_id = unsafe {
            ffi::CStr::from_ptr(res_id)
                .to_str()
                .expect("Residue number containted invalid UTF-8 bytes")
                .trim()
                .to_owned()
        };

        let (resnum, inscode) =
            if res_id.chars().last().unwrap().is_numeric() {
                (res_id, None)
            } else {
                let resnum = res_id[..res_id.len() - 1].to_string();
                let inscode = res_id.chars().last().unwrap();
                (resnum, Some(inscode))
            };

        uid.2 = Some((resnum.parse().unwrap(), inscode));

        uid
    }

    fn from_atom_ptr(node: *mut freesasa_node) -> UidPrimitive {
        #[cfg(debug_assertions)]
        assert_nodetype(&node, NodeType::Atom);

        let residue_ptr = unsafe { freesasa_node_parent(node) };

        let mut uid = Self::from_residue_ptr(residue_ptr);

        let atom_name = unsafe { freesasa_node_name(node) };

        // convert from c-style string to String
        let atom_name = unsafe {
            ffi::CStr::from_ptr(atom_name).to_str().unwrap().to_string()
        };

        uid.3 = Some(atom_name);

        uid
    }
}

impl Display for NodeUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut uid = format!("{}:", self.structure);

        // Add the chain ID if it exists...
        if let Some(chain) = self.chain {
            uid.push(chain);
        }
        // ...else return.
        else {
            return write!(f, "{}", uid);
        };

        // Add the residue ID if it exists...
        if let Some((resnum, inscode)) = self.res_id {
            uid.push(':');
            uid.push_str(&resnum.to_string());
            if let Some(code) = inscode {
                uid.push(code);
            }
        }
        // ...else return.
        else {
            return write!(f, "{}", uid);
        };

        // Add the atom name if it exists...
        if let Some(atom_name) = &self.atom_name {
            uid.push(':');
            uid.push_str(atom_name);
        };

        write!(f, "{}", uid)
    }
}

impl serde::Serialize for NodeUid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
