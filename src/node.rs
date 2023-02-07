// TODO
use freesasa_sys::{
    freesasa_nodetype, freesasa_nodetype_FREESASA_NODE_ATOM,
    freesasa_nodetype_FREESASA_NODE_CHAIN,
    freesasa_nodetype_FREESASA_NODE_NONE,
    freesasa_nodetype_FREESASA_NODE_RESIDUE,
    freesasa_nodetype_FREESASA_NODE_STRUCTURE,
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
            freesasa_nodetype_FREESASA_NODE_NONE => NodeLevel::None,
            freesasa_nodetype_FREESASA_NODE_ATOM => NodeLevel::Atom,
            freesasa_nodetype_FREESASA_NODE_RESIDUE => {
                NodeLevel::Residue
            }
            freesasa_nodetype_FREESASA_NODE_CHAIN => NodeLevel::Chain,
            freesasa_nodetype_FREESASA_NODE_STRUCTURE => {
                NodeLevel::Model
            }
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
        NodeLevel::None => freesasa_nodetype_FREESASA_NODE_NONE,
        NodeLevel::Atom => freesasa_nodetype_FREESASA_NODE_ATOM,
        NodeLevel::Residue => freesasa_nodetype_FREESASA_NODE_RESIDUE,
        NodeLevel::Chain => freesasa_nodetype_FREESASA_NODE_CHAIN,
        NodeLevel::Model => freesasa_nodetype_FREESASA_NODE_STRUCTURE,
    }
}
