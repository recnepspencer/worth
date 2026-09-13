//! Stable declarations for checksum-protected physical artifact formats.
//!
//! These values describe persisted meaning. They perform no parsing, I/O,
//! checksum calculation, admission, recovery selection, or lifecycle work.

mod algorithm;
mod coverage;
pub mod families;
mod family;
mod version;

pub use algorithm::PhysicalIntegrityAlgorithm;
pub use coverage::{
    PhysicalIntegrityChecksumDeclaration, PhysicalIntegrityChecksumField,
    PhysicalIntegrityCoverageBoundary, PhysicalIntegrityCoveredRange,
};
pub use family::{PhysicalIntegrityArtifactFamily, PhysicalIntegrityFormatDeclaration};
pub use version::PhysicalIntegrityFormatVersion;
