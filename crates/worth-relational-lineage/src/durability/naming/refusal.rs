//! Typed refusals of the durable group lineage frame.
//!
//! Every refusal is returned before any recovered group state escapes: a
//! refused frame installs no group lineage at all.

use std::fmt;

/// Which part of the durable encoding a structural refusal was found in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistentNameWireSection {
    /// A commit envelope's canonical group lineage frame.
    GroupLineage,
}

/// Typed refusal to encode or decode a durable group lineage frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistentNameDurabilityError {
    /// The frame declares a format version this build cannot read.
    UnsupportedFormatVersion { found: u8, supported: u8 },
    /// The bytes do not begin with the expected section magic.
    UnrecognizedMagic { section: PersistentNameWireSection },
    /// The bytes end before a declared field is complete.
    Truncated {
        section: PersistentNameWireSection,
        needed: usize,
        remaining: usize,
    },
    /// The frame continues after its last declared field.
    TrailingBytes { remaining: usize },
    /// A persisted text field is not UTF-8.
    MalformedUtf8 { field: &'static str },
    /// A persisted tag names no variant.
    InvalidTag { field: &'static str, tag: u8 },
    /// Persisted data is not in its one canonical order or range.
    NonCanonical { field: &'static str },
    /// A field is too large for the format's 32-bit length prefix.
    OversizedField { field: &'static str, len: usize },
}

impl fmt::Display for PersistentNameDurabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormatVersion { found, supported } => write!(
                formatter,
                "durable group lineage format version {found} is unsupported (supported: {supported})"
            ),
            Self::UnrecognizedMagic { section } => {
                write!(formatter, "durable {section:?} magic is unrecognized")
            }
            Self::Truncated {
                section,
                needed,
                remaining,
            } => write!(
                formatter,
                "durable {section:?} needs {needed} bytes but {remaining} remain"
            ),
            Self::TrailingBytes { remaining } => {
                write!(formatter, "durable group lineage frame has {remaining} trailing bytes")
            }
            Self::MalformedUtf8 { field } => {
                write!(formatter, "durable group lineage field `{field}` is not UTF-8")
            }
            Self::InvalidTag { field, tag } => {
                write!(formatter, "durable group lineage field `{field}` has invalid tag {tag}")
            }
            Self::NonCanonical { field } => {
                write!(formatter, "durable group lineage field `{field}` is not canonical")
            }
            Self::OversizedField { field, len } => write!(
                formatter,
                "durable group lineage field `{field}` of {len} items exceeds the format limit"
            ),
        }
    }
}

impl std::error::Error for PersistentNameDurabilityError {}
