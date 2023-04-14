//! Node level enum and conversion functions
//!
//! The `NodeLevel` enum is used to specify the level of the tree to be used for
//! the calculation. The enum is used to convert between the `freesasa_nodetype`
//! enum used by the `freesasa-sys` library and the `NodeLevel` enum used by the
//! `rust-sasa` library.

use std::{
    ops::{Add, Sub},
    str::FromStr,
};

use freesasa_sys::{
    freesasa_node, freesasa_node_area, freesasa_node_type,
    freesasa_nodetype,
    freesasa_nodetype_FREESASA_NODE_ATOM as FREESASA_NODE_ATOM,
    freesasa_nodetype_FREESASA_NODE_CHAIN as FREESASA_NODE_CHAIN,
    freesasa_nodetype_FREESASA_NODE_NONE as FREESASA_NODE_NONE,
    freesasa_nodetype_FREESASA_NODE_RESIDUE as FREESASA_NODE_RESIDUE,
    freesasa_nodetype_FREESASA_NODE_RESULT as FREESASA_NODE_RESULT,
    freesasa_nodetype_FREESASA_NODE_ROOT as FREESASA_NODE_ROOT,
    freesasa_nodetype_FREESASA_NODE_STRUCTURE as FREESASA_NODE_STRUCTURE,
};

use crate::{uids::NodeUid, utils::assert_nodetype};

use super::properties::{
    AtomProperties, ChainProperties, ResidueProperties,
    ResultProperties, StructureProperties,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum NodeType {
    None,
    Atom,
    Residue,
    Chain,
    Structure,
    Result,
    Root,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::None
    }
}

impl FromStr for NodeType {
    type Err = String;

    fn from_str(level: &str) -> Result<Self, Self::Err> {
        match level.to_lowercase().as_str() {
            "none" => Ok(NodeType::None),
            "atom" => Ok(NodeType::Atom),
            "residue" => Ok(NodeType::Residue),
            "chain" => Ok(NodeType::Chain),
            "structure" => Ok(NodeType::Structure),
            _ => Err(format!("Invalid node level: {}", level)),
        }
    }
}

impl NodeType {
    pub fn from_fs_level(level: freesasa_nodetype) -> Self {
        match level {
            FREESASA_NODE_NONE => NodeType::None,
            FREESASA_NODE_ATOM => NodeType::Atom,
            FREESASA_NODE_RESIDUE => NodeType::Residue,
            FREESASA_NODE_CHAIN => NodeType::Chain,
            FREESASA_NODE_STRUCTURE => NodeType::Structure,
            FREESASA_NODE_RESULT => NodeType::Result,
            FREESASA_NODE_ROOT => NodeType::Root,
            _ => panic!("Invalid freesasa_nodetype"),
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            NodeType::None => "none",
            NodeType::Atom => "atom",
            NodeType::Residue => "residue",
            NodeType::Chain => "chain",
            NodeType::Structure => "structure",
            NodeType::Result => "result",
            NodeType::Root => "root",
        }
    }

    pub fn to_fs_level(&self) -> freesasa_nodetype {
        match *self {
            NodeType::None => FREESASA_NODE_NONE,
            NodeType::Atom => FREESASA_NODE_ATOM,
            NodeType::Residue => FREESASA_NODE_RESIDUE,
            NodeType::Chain => FREESASA_NODE_CHAIN,
            NodeType::Structure => FREESASA_NODE_STRUCTURE,
            NodeType::Result => FREESASA_NODE_RESULT,
            NodeType::Root => FREESASA_NODE_ROOT,
        }
    }

    pub(crate) fn nodetype_of_ptr(node: *const freesasa_node) -> Self {
        assert!(!node.is_null());
        let level = unsafe { freesasa_node_type(node) };
        NodeType::from_fs_level(level)
    }
}

/// Struct for storing SASA area values for a node.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeArea {
    total: f64,
    main_chain: f64,
    side_chain: f64,
    polar: f64,
    apolar: f64,
    unknown: f64,
}

impl Default for NodeArea {
    fn default() -> Self {
        Self {
            total: 0.0,
            main_chain: 0.0,
            side_chain: 0.0,
            polar: 0.0,
            apolar: 0.0,
            unknown: 0.0,
        }
    }
}

impl Sub for NodeArea {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            total: self.total - rhs.total,
            main_chain: self.main_chain - rhs.main_chain,
            side_chain: self.side_chain - rhs.side_chain,
            polar: self.polar - rhs.polar,
            apolar: self.apolar - rhs.apolar,
            unknown: self.unknown - rhs.unknown,
        }
    }
}

impl Sub for &NodeArea {
    type Output = NodeArea;

    fn sub(self, rhs: Self) -> Self::Output {
        NodeArea {
            total: self.total - rhs.total,
            main_chain: self.main_chain - rhs.main_chain,
            side_chain: self.side_chain - rhs.side_chain,
            polar: self.polar - rhs.polar,
            apolar: self.apolar - rhs.apolar,
            unknown: self.unknown - rhs.unknown,
        }
    }
}

impl Add for NodeArea {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            total: self.total + rhs.total,
            main_chain: self.main_chain + rhs.main_chain,
            side_chain: self.side_chain + rhs.side_chain,
            polar: self.polar + rhs.polar,
            apolar: self.apolar + rhs.apolar,
            unknown: self.unknown + rhs.unknown,
        }
    }
}

impl Add for &NodeArea {
    type Output = NodeArea;

    fn add(self, rhs: Self) -> Self::Output {
        NodeArea {
            total: self.total + rhs.total,
            main_chain: self.main_chain + rhs.main_chain,
            side_chain: self.side_chain + rhs.side_chain,
            polar: self.polar + rhs.polar,
            apolar: self.apolar + rhs.apolar,
            unknown: self.unknown + rhs.unknown,
        }
    }
}

impl NodeArea {
    pub(super) fn new_from_node(node: &*mut freesasa_node) -> Self {
        #[cfg(debug_assertions)]
        {
            trace!(
                "NodeType: {:?}",
                NodeType::from_fs_level(unsafe {
                    freesasa_node_type(*node)
                })
            );
        }

        let area_ptr = unsafe { freesasa_node_area(*node) };

        let total = unsafe { (*area_ptr).total };
        let main_chain = unsafe { (*area_ptr).main_chain };
        let side_chain = unsafe { (*area_ptr).side_chain };
        let polar = unsafe { (*area_ptr).polar };
        let apolar = unsafe { (*area_ptr).apolar };
        let unknown = unsafe { (*area_ptr).unknown };

        Self {
            total,
            main_chain,
            side_chain,
            polar,
            apolar,
            unknown,
        }
    }

    /// Returns the total SASA area for the node.
    pub fn total(&self) -> f64 {
        self.total
    }

    /// Returns the main chain SASA area for the node.
    pub fn main_chain(&self) -> f64 {
        self.main_chain
    }

    /// Returns the side chain SASA area for the node.
    pub fn side_chain(&self) -> f64 {
        self.side_chain
    }

    /// Returns the polar SASA area for the node.
    pub fn polar(&self) -> f64 {
        self.polar
    }

    /// Returns the apolar SASA area for the node.
    pub fn apolar(&self) -> f64 {
        self.apolar
    }

    /// Returns the unknown SASA area for the node.
    pub fn unknown(&self) -> f64 {
        self.unknown
    }
}

/// Enum for storing different types of node properties.
#[derive(Debug, serde::Serialize, Clone)]
#[serde(untagged)]
pub enum NodeProperties {
    Atom(AtomProperties),
    Residue(ResidueProperties),
    Chain(ChainProperties),
    Structure(StructureProperties),
    Result(ResultProperties),
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct Node {
    area: Option<NodeArea>,

    #[serde(skip)]
    uid: Option<NodeUid>,
    nodetype: NodeType,
    #[serde(skip)]
    properties: Option<NodeProperties>,
}

impl Node {
    pub fn new(
        nodetype: NodeType,
        properties: Option<NodeProperties>,
        area: Option<NodeArea>,
        uid: Option<NodeUid>,
    ) -> Self {
        Self {
            properties,
            area,
            uid,
            nodetype,
        }
    }

    pub(crate) unsafe fn from_ptr(node: &*mut freesasa_node) -> Self {
        let nodetype =
            NodeType::from_fs_level(freesasa_node_type(*node));

        match nodetype {
            NodeType::Atom => new_atom_node(node),
            NodeType::Residue => new_residue_node(node),
            NodeType::Chain => new_chain_node(node),
            NodeType::Structure => new_structure_node(node),
            NodeType::Root => Node {
                nodetype: NodeType::Root,
                properties: None,
                area: None,
                uid: None,
            },
            NodeType::Result => new_result_node(node),
            _ => panic!("Invalid node type: {:?}", nodetype),
        }
    }

    pub fn properties(&self) -> Option<&NodeProperties> {
        self.properties.as_ref()
    }

    pub fn area(&self) -> Option<&NodeArea> {
        self.area.as_ref()
    }

    pub fn uid(&self) -> Option<&NodeUid> {
        self.uid.as_ref()
    }

    pub fn nodetype(&self) -> &NodeType {
        &self.nodetype
    }

    pub fn set_area(&mut self, area: Option<NodeArea>) {
        self.area = area;
    }
}

/// Constructs a new `Node` from a `freesasa_node` pointer,
/// where the node type is known to be an atom.
///
/// # Panics
/// If the node type is not an atom.
fn new_atom_node(node: &*mut freesasa_node) -> Node {
    assert_nodetype(node, NodeType::Atom);

    let properties = AtomProperties::new(node);
    let area = NodeArea::new_from_node(node);

    let uid = NodeUid::new(
        properties.residue.chain.structure,
        Some(properties.residue.chain.id),
        Some((properties.residue.resnum, properties.residue.inscode)),
        Some(properties.name.clone()),
    );

    Node {
        properties: Some(NodeProperties::Atom(properties)),
        area: Some(area),
        uid,
        nodetype: NodeType::Atom,
    }
}

/// Constructs a new `Node` from a `freesasa_node` pointer,
/// where the node type is known to be a residue.
///
/// # Panics
/// If the node type is not a residue.
fn new_residue_node(node: &*mut freesasa_node) -> Node {
    assert_nodetype(node, NodeType::Residue);

    let properties = ResidueProperties::new(node);
    let area = NodeArea::new_from_node(node);

    let uid = NodeUID::Residue(ResidueUID::new(
        properties.chain.structure,
        properties.chain.id,
        properties.resnum,
        properties.inscode,
    ));

    Node {
        properties: Some(NodeProperties::Residue(properties)),
        area: Some(area),
        uid,
        nodetype: NodeType::Residue,
    }
}

/// Constructs a new `Node` from a `freesasa_node` pointer,
/// where the node type is known to be a chain.
///
/// # Panics
/// If the node type is not a chain.
fn new_chain_node(node: &*mut freesasa_node) -> Node {
    assert_nodetype(node, NodeType::Chain);

    let properties = ChainProperties::new(node);
    let area = NodeArea::new_from_node(node);

    let uid = NodeUID::Chain(ChainUID::new(
        properties.structure,
        properties.id,
    ));

    Node {
        properties: Some(NodeProperties::Chain(properties)),
        area: Some(area),
        uid,
        nodetype: NodeType::Chain,
    }
}

/// Constructs a new `Node` from a `freesasa_node` pointer,
/// where the node type is known to be a structure.
///
/// # Panics
/// If the node type is not a structure.
fn new_structure_node(node: &*mut freesasa_node) -> Node {
    assert_nodetype(node, NodeType::Structure);

    let properties = StructureProperties::new(node);
    let area = NodeArea::new_from_node(node);

    let uid = NodeUID::Structure(properties.name.clone());

    Node {
        properties: Some(NodeProperties::Structure(properties)),
        area: Some(area),
        uid,
        nodetype: NodeType::Structure,
    }
}

fn new_result_node(node: &*mut freesasa_node) -> Node {
    assert_nodetype(node, NodeType::Result);

    let properties = ResultProperties::new(node);

    Node {
        properties: Some(NodeProperties::Result(properties)),
        area: None,
        uid: NodeUID::Result,
        nodetype: NodeType::Result,
    }
}
