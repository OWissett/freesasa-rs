//! Module containing the various result types used by the library.
//!
//! [`SasaResult`] is the simplest result type, it contains the total SASA
//! value and the SASA values for each atom in the structure. This is anologous
//! to the `freesasa_result` struct in the C API.
//!
//! [`SasaTree`] is a more complex result type, it contains a tree of the
//! structure, with each node containing the SASA value for that node. This is
//! analogous to the [`freesasa_sys::freesasa_node`] struct in the C API, obtained by calling
//! [`freesasa_sys::freesasa_calc_tree`].
//!
//! Structs in this library are annotated with the [`serde::Serialize`] attribute, which
//! allows them to be serialized to JSON. This is useful for debugging, and for passing
//! results to other programs.
//!
//! ## Notes
//!
//! This module is expected to be called internally by the library, and
//! the construction of objects in this module should not be performed by the user.
//!
pub mod node;

// Modules to re-export at the top level
mod result_;
mod tree;

pub use self::tree::*;
pub use result_::*;
