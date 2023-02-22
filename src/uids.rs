use std::{collections::HashMap, fmt::Display};

use freesasa_sys::freesasa_node;

pub type ResidueUIDMap = HashMap<ResidueUID, (*mut freesasa_node, f64)>;

#[derive(Debug, serde::Serialize, PartialEq, Eq, Hash, Clone)]
pub struct AtomUID {
    #[serde(flatten)]
    residue: ResidueUID, // Residue UID
    name: String, // Atom name (e.g. CA, CB, etc.)
}

impl AtomUID {
    pub fn new(
        structure: i32,
        chain: char,
        resnum: i32,
        inscode: Option<char>,
        name: String,
    ) -> Self {
        Self {
            residue: ResidueUID::new(structure, chain, resnum, inscode),
            name,
        }
    }

    pub fn residue(&self) -> &ResidueUID {
        &self.residue
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ResidueUID {
    #[serde(flatten)]
    chain: ChainUID, // Chain
    resnum: i32,           // Residue number
    inscode: Option<char>, // Residue insertion code
}

impl ResidueUID {
    pub fn new(
        structure: i32,
        chain: char,
        resnum: i32,
        inscode: Option<char>,
    ) -> Self {
        Self {
            chain: ChainUID::new(structure, chain),
            resnum,
            inscode,
        }
    }

    pub fn chain(&self) -> &ChainUID {
        &self.chain
    }

    pub fn resnum(&self) -> i32 {
        self.resnum
    }

    pub fn inscode(&self) -> Option<char> {
        self.inscode
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ChainUID {
    chain: char, // Chain

    #[serde(skip)]
    structure: i32, // Structure UID
}

impl ChainUID {
    pub fn new(structure: i32, chain: char) -> Self {
        Self { chain, structure }
    }

    pub fn chain(&self) -> char {
        self.chain
    }

    pub fn structure(&self) -> i32 {
        self.structure
    }
}

// ----- //
// Enums //
// ----- //

/// Enum for storing different types of node UIDs.
#[derive(Debug, serde::Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(untagged)]
pub enum NodeUID {
    Atom(AtomUID),       // Unique ID for an atom. e.g., A:1A:CA
    Residue(ResidueUID), // Unique ID for a residue. e.g., A:1A
    Chain(ChainUID),     // Unique ID for a chain. e.g., A
    Structure(String),   // Unique ID for a structure. e.g., 1A2B
}

// ---------------------- //
// Display Implementation //
// ---------------------- //

impl Display for NodeUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeUID::Atom(atom) => write!(f, "{}", atom),
            NodeUID::Residue(residue) => write!(f, "{}", residue),
            NodeUID::Chain(chain) => write!(f, "{}", chain),
            NodeUID::Structure(structure) => write!(f, "{}", structure),
        }
    }
}

impl Display for AtomUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.residue.inscode {
            Some(code) => {
                write!(
                    f,
                    "{}:{}{}:{}",
                    self.residue.chain,
                    self.residue.resnum,
                    code,
                    self.name
                )
            }
            None => write!(
                f,
                "{}:{}:{}",
                self.residue.chain, self.residue.resnum, self.name
            ),
        }
    }
}

impl Display for ResidueUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.inscode {
            Some(code) => {
                write!(f, "{}:{}{}", self.chain, self.resnum, code)
            }
            None => write!(f, "{}:{}", self.chain, self.resnum),
        }
    }
}

impl Display for ChainUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.structure, self.chain)
    }
}
