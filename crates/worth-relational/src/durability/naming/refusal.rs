//! Typed refusals of the durable persistent-name log.
//!
//! Every refusal is returned before any rebuilt index escapes: a refused log
//! installs no naming state at all.

use std::fmt;

use crate::history::data::CommitId;
use crate::lineage::data::PersistentNameSubjectKind;
use crate::lineage::index::PersistentNameIndexDenial;

/// Which part of the durable log a structural refusal was found in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistentNameWireSection {
    LogHeader,
    FrameHeader,
    FramePayload,
}

/// Where an unverified 32-byte identity appeared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistentNameReferenceRole {
    /// A `kept_name` on a `Create`, `Replace`, `Merge`, or `Restore`.
    KeptName,
    /// A parent identity inside a persisted birth.
    BirthParent,
    /// A name embedded in a persisted birth's canonical discriminator.
    DiscriminatorReference,
    /// An entity or relation name that declares carriers.
    CarrierKey,
    /// A member identity inside a declared carrier set.
    Carrier,
}

/// Typed refusal to encode, decode, readmit, or rebuild durable naming state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistentNameDurabilityError {
    /// The log or a frame declares a format version this build cannot read.
    UnsupportedFormatVersion { found: u8, supported: u8 },
    /// The bytes do not begin with the expected section magic.
    UnrecognizedMagic { section: PersistentNameWireSection },
    /// The bytes end before a declared field or frame is complete.
    Truncated {
        section: PersistentNameWireSection,
        needed: usize,
        remaining: usize,
    },
    /// A frame payload continues after its last declared field.
    TrailingBytes { remaining: usize },
    /// A persisted text field is not UTF-8.
    MalformedUtf8 { field: &'static str },
    /// A persisted tag names no variant.
    InvalidTag { field: &'static str, tag: u8 },
    /// Persisted data is not in its one canonical order or range.
    NonCanonical { field: &'static str },
    /// A field is too large for the format's 32-bit length prefix.
    OversizedField { field: &'static str, len: usize },
    /// A persisted birth's parents are unsorted or duplicated.
    NonCanonicalBirthParents { identity: [u8; 32] },
    /// A persisted birth's discriminator fails the canonical scan.
    MalformedBirthDiscriminator { identity: [u8; 32] },
    /// Re-hashing a persisted birth's typed parts does not reproduce its
    /// stored identity: a birth part or the stored digest was altered.
    CorruptBirthDigest {
        stored: [u8; 32],
        recomputed: [u8; 32],
    },
    /// A 32-byte identity is referenced without a verified persisted birth.
    UnverifiedBareIdentity {
        raw: [u8; 32],
        role: PersistentNameReferenceRole,
    },
    /// A lineage event carries naming payload its kind cannot carry.
    InvalidEventShape { event_id: u64, detail: &'static str },
    /// The derived index refused a decoded, verified commit frame.
    IndexRefused {
        commit: CommitId,
        denial: PersistentNameIndexDenial,
    },
    /// A retained identity matches no persisted birth the rebuilt index holds.
    UnknownPersistedName {
        subject: PersistentNameSubjectKind,
        raw: [u8; 32],
    },
}

impl fmt::Display for PersistentNameDurabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormatVersion { found, supported } => write!(
                formatter,
                "durable persistent-name format version {found} is unsupported (supported: {supported})"
            ),
            Self::UnrecognizedMagic { section } => {
                write!(formatter, "durable persistent-name {section:?} magic is unrecognized")
            }
            Self::Truncated {
                section,
                needed,
                remaining,
            } => write!(
                formatter,
                "durable persistent-name {section:?} needs {needed} bytes but {remaining} remain"
            ),
            Self::TrailingBytes { remaining } => write!(
                formatter,
                "durable persistent-name frame payload has {remaining} trailing bytes"
            ),
            Self::MalformedUtf8 { field } => {
                write!(formatter, "durable persistent-name field `{field}` is not UTF-8")
            }
            Self::InvalidTag { field, tag } => {
                write!(formatter, "durable persistent-name field `{field}` has invalid tag {tag}")
            }
            Self::NonCanonical { field } => {
                write!(formatter, "durable persistent-name field `{field}` is not canonical")
            }
            Self::OversizedField { field, len } => write!(
                formatter,
                "durable persistent-name field `{field}` of {len} items exceeds the format limit"
            ),
            Self::NonCanonicalBirthParents { .. } => {
                write!(formatter, "persisted birth parents are not strictly sorted")
            }
            Self::MalformedBirthDiscriminator { .. } => {
                write!(formatter, "persisted birth discriminator is malformed")
            }
            Self::CorruptBirthDigest { .. } => write!(
                formatter,
                "persisted birth parts do not re-hash to their stored identity"
            ),
            Self::UnverifiedBareIdentity { role, .. } => write!(
                formatter,
                "persisted {role:?} identity has no verified persisted birth"
            ),
            Self::InvalidEventShape { event_id, detail } => {
                write!(formatter, "persisted lineage event {event_id}: {detail}")
            }
            Self::IndexRefused { commit, denial } => write!(
                formatter,
                "persisted naming commit {} refused by the derived index: {denial}",
                commit.0
            ),
            Self::UnknownPersistedName { subject, .. } => write!(
                formatter,
                "retained {subject:?} identity matches no persisted birth"
            ),
        }
    }
}

impl std::error::Error for PersistentNameDurabilityError {}
