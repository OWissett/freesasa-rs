use std::ffi;
use std::fmt::Display;

use freesasa_sys::{
    freesasa_node, freesasa_node_name, freesasa_node_parent,
    freesasa_node_residue_number,
};

use crate::{result::node::NodeType, utils::assert_nodetype};

/// ID for a residue, which is a tuple of the residue number and insertion code.
type ResID = (i32, Option<char>);

// Structure, chain, residue, atom
type UidPrimitive = (char, Option<ResID>, Option<String>);

/// Unique ID for a structure node (e.g. a chain, residue, atom, etc.).
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct NodeUid {
    // NOTE: The ordering of the fields is important, because it determines the
    // precedence of the fields in derive `Ord` and `PartialOrd` implementations.
    //
    /// Chain ID.
    chain: char,

    /// Residue ID - tuple of residue number and insertion code.
    res_id: Option<ResID>,

    /// Atom name - Such as "CA" or "O".
    atom_name: Option<String>,
}

impl NodeUid {
    pub fn new(
        chain: char,
        res_id: Option<ResID>,
        atom_name: Option<String>,
    ) -> Self {
        Self {
            chain,
            res_id,
            atom_name,
        }
    }

    // Create a new `NodeUid` from a `UidPrimitive`.
    fn from_primitive(
        (chain, res_id, atom_name): UidPrimitive,
    ) -> Self {
        #[cfg(debug_assertions)]
        {
            if atom_name.is_some() {
                assert!(
                    res_id.is_some(),
                    "Atom name provided without residue ID"
                );
            }
        }

        Self {
            chain,
            res_id,
            atom_name,
        }
    }

    pub(crate) fn from_ptr(node: *mut freesasa_node) -> Option<Self> {
        let node_type = NodeType::nodetype_of_ptr(node);

        match node_type {
            NodeType::None => None,
            NodeType::Root => None,
            NodeType::Result => None,
            NodeType::Structure => None,
            NodeType::Chain => {
                Some(Self::from_primitive(Self::from_chain_ptr(node)))
            }
            NodeType::Residue => {
                Some(Self::from_primitive(Self::from_residue_ptr(node)))
            }
            NodeType::Atom => {
                Some(Self::from_primitive(Self::from_atom_ptr(node)))
            }
        }
    }

    pub fn chain(&self) -> &char {
        &self.chain
    }

    pub fn res_id(&self) -> Option<&ResID> {
        self.res_id.as_ref()
    }

    pub fn atom_name(&self) -> Option<&str> {
        self.atom_name.as_deref()
    }

    fn from_chain_ptr(node: *mut freesasa_node) -> UidPrimitive {
        #[cfg(debug_assertions)]
        assert_nodetype(&node, NodeType::Chain);

        let chain = unsafe { freesasa_node_name(node) };

        // convert from c-style string to String
        let chain = unsafe {
            match ffi::CStr::from_ptr(chain)
                .to_str()
                .unwrap()
                .chars()
                .next()
            {
                Some(c) => c,
                None => {
                    warn!("Chain name contained invalid UTF-8 bytes, using * as chain ID");
                    '*'
                }
            }
        };

        (chain, None, None)
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

        uid.1 = Some((resnum.parse().unwrap(), inscode));

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

        uid.2 = Some(atom_name);

        uid
    }
}

impl Display for NodeUid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut uid = String::new();

        uid.push(self.chain);

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
