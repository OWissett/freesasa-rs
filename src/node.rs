//! Node level enum and conversion functions
//!
//! The `NodeLevel` enum is used to specify the level of the tree to be used for
//! the calculation. The enum is used to convert between the `freesasa_nodetype`
//! enum used by the `freesasa-sys` library and the `NodeLevel` enum used by the
//! `rust-sasa` library.

use freesasa_sys::{
    freesasa_nodetype,
    freesasa_nodetype_FREESASA_NODE_ATOM as FREESASA_NODE_ATOM,
    freesasa_nodetype_FREESASA_NODE_CHAIN as FREESASA_NODE_CHAIN,
    freesasa_nodetype_FREESASA_NODE_NONE as FREESASA_NODE_NONE,
    freesasa_nodetype_FREESASA_NODE_RESIDUE as FREESASA_NODE_RESIDUE,
    freesasa_nodetype_FREESASA_NODE_STRUCTURE as FREESASA_NODE_STRUCTURE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeLevel {
    None,
    Atom,
    Residue,
    Chain,
    Model,
}

impl Default for NodeLevel {
    fn default() -> Self {
        NodeLevel::None
    }
}

impl NodeLevel {
    pub fn from_fs_level(level: freesasa_nodetype) -> Self {
        match level {
            FREESASA_NODE_NONE => NodeLevel::None,
            FREESASA_NODE_ATOM => NodeLevel::Atom,
            FREESASA_NODE_RESIDUE => NodeLevel::Residue,
            FREESASA_NODE_CHAIN => NodeLevel::Chain,
            FREESASA_NODE_STRUCTURE => NodeLevel::Model,
            _ => panic!("Invalid freesasa_nodetype"),
        }
    }

    pub fn from_str(level: &str) -> Option<Self> {
        match level.to_lowercase().as_str() {
            "none" => Some(NodeLevel::None),
            "atom" => Some(NodeLevel::Atom),
            "residue" => Some(NodeLevel::Residue),
            "chain" => Some(NodeLevel::Chain),
            "model" => Some(NodeLevel::Model),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            NodeLevel::None => "none",
            NodeLevel::Atom => "atom",
            NodeLevel::Residue => "residue",
            NodeLevel::Chain => "chain",
            NodeLevel::Model => "model",
        }
    }

    pub fn to_fs_level(&self) -> freesasa_nodetype {
        fs_level_from_node_level(*self)
    }
}

fn fs_level_from_node_level(level: NodeLevel) -> freesasa_nodetype {
    match level {
        NodeLevel::None => FREESASA_NODE_NONE,
        NodeLevel::Atom => FREESASA_NODE_ATOM,
        NodeLevel::Residue => FREESASA_NODE_RESIDUE,
        NodeLevel::Chain => FREESASA_NODE_CHAIN,
        NodeLevel::Model => FREESASA_NODE_STRUCTURE,
    }
}
