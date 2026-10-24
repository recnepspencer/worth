//! Anchored records: named "current value" records that a Store home replaces
//! whole, one generation at a time.
//!
//! Each name is a root anchor selecting exactly one record. A publication
//! writes one framed record and advances the anchor from the generation the
//! caller observed to the next one in the same root generation, so the value
//! is current exactly when that root is durably selected. Store treats the
//! payload as opaque owner bytes; the frame binds it to its anchor name and
//! generation.
//!
//! The canonical commit journal is also anchored, but it keeps its own door:
//! its name is reserved and cannot be named here.

use std::collections::BTreeSet;

use worth_store_physical_format::RecordAnchorName;

use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, IndeterminatePhysicalMutation, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationPreparationOutcome,
    PhysicalReadProtectionDenial, PhysicalRecordId, ProvenNoEffectPhysicalMutation,
    RecordAppendDenial, RecordReadError, RecordStreamFailure, ServingPhysicalRuntime,
    RELATIONAL_CANONICAL_COMMITS,
};

mod frame;
mod publication;
mod read;

/// The name of one anchored record: a valid anchor name that no other Store
/// door owns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnchoredRecordName(RecordAnchorName);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchoredRecordNameDenial {
    /// Not 1 to 32 bytes of `[a-z0-9.-]`.
    Malformed,
    /// The name belongs to the canonical commit journal.
    ReservedForCanonicalJournal,
}

impl AnchoredRecordName {
    pub fn new(name: &str) -> Result<Self, AnchoredRecordNameDenial> {
        let anchor = RecordAnchorName::new(name).ok_or(AnchoredRecordNameDenial::Malformed)?;
        if anchor.as_str() == RELATIONAL_CANONICAL_COMMITS {
            return Err(AnchoredRecordNameDenial::ReservedForCanonicalJournal);
        }
        Ok(Self(anchor))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub(in crate::physical_runtime) const fn anchor(self) -> RecordAnchorName {
        self.0
    }
}

/// The identity of one durable value of an anchored record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnchoredRecordVersion {
    name: AnchoredRecordName,
    generation: u64,
    record: PhysicalRecordId,
    payload_sha256: [u8; 32],
    payload_bytes: u64,
}

impl AnchoredRecordVersion {
    pub const fn name(&self) -> AnchoredRecordName {
        self.name
    }
    /// One for the first value; each replacement adds one.
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn record(&self) -> PhysicalRecordId {
        self.record
    }
    pub const fn payload_sha256(&self) -> [u8; 32] {
        self.payload_sha256
    }
    pub const fn payload_bytes(&self) -> u64 {
        self.payload_bytes
    }
}

/// The current value of an anchored record, verified against its anchor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchoredRecord {
    version: AnchoredRecordVersion,
    bytes: Vec<u8>,
}

impl AnchoredRecord {
    pub const fn version(&self) -> AnchoredRecordVersion {
        self.version
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// One requested replacement: the value at `expected_generation` (zero when
/// the record is absent) becomes `payload` at the next generation.
pub struct AnchoredRecordPublication {
    name: AnchoredRecordName,
    expected_generation: u64,
    payload: Vec<u8>,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
}

impl AnchoredRecordPublication {
    pub fn new(
        name: AnchoredRecordName,
        expected_generation: u64,
        payload: Vec<u8>,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
    ) -> Self {
        Self {
            name,
            expected_generation,
            payload,
            placement,
            deadline,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchoredRecordPublished {
    /// This call made the value durable.
    Published(AnchoredRecordVersion),
    /// The identical value was already durable at the requested generation;
    /// nothing was written.
    AlreadyPublished(AnchoredRecordVersion),
}

impl AnchoredRecordPublished {
    pub const fn version(&self) -> AnchoredRecordVersion {
        match self {
            Self::Published(version) | Self::AlreadyPublished(version) => *version,
        }
    }
}

/// A publication that did not make its value durable. Only `Indeterminate`
/// and `CompletedWithoutRecord` may have had an effect.
#[non_exhaustive]
pub enum AnchoredRecordPublicationFailure {
    EmptyPayload,
    /// The anchor is not at the expected generation. No effect.
    GenerationConflict {
        current: u64,
    },
    /// A new name would exceed the root's anchor table. No effect.
    AnchorTableFull,
    /// An earlier publication to this name has an indeterminate fate; recovery
    /// must establish it before the name admits another. No effect.
    Unresolved,
    /// The record could not form an admissible batch. No effect.
    Batch(RecordAppendDenial),
    Idempotency(PhysicalMutationIdempotencyIssuanceDenial),
    /// Physical preparation refused before any WAL effect.
    Preparation(PhysicalMutationPreparationOutcome),
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    /// The publication may or may not be durable.
    Indeterminate(IndeterminatePhysicalMutation),
    /// The mutation completed but did not report exactly one record. It may
    /// be durable; reopening establishes the current value.
    CompletedWithoutRecord,
    /// Reading the current anchor failed before any effect.
    Read(AnchoredRecordReadFailure),
}

impl std::fmt::Debug for AnchoredRecordPublicationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPayload => f.write_str("EmptyPayload"),
            Self::GenerationConflict { current } => f
                .debug_struct("GenerationConflict")
                .field("current", current)
                .finish(),
            Self::AnchorTableFull => f.write_str("AnchorTableFull"),
            Self::Unresolved => f.write_str("Unresolved"),
            Self::Batch(denial) => f.debug_tuple("Batch").field(denial).finish(),
            Self::Idempotency(_) => f.write_str("Idempotency"),
            Self::Preparation(_) => f.write_str("Preparation"),
            Self::ProvenNoEffect(fate) => f
                .debug_struct("ProvenNoEffect")
                .field("cause", &fate.cause())
                .finish(),
            Self::Indeterminate(fate) => f
                .debug_struct("Indeterminate")
                .field("stage", &fate.stage())
                .finish(),
            Self::CompletedWithoutRecord => f.write_str("CompletedWithoutRecord"),
            Self::Read(failure) => f.debug_tuple("Read").field(failure).finish(),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum AnchoredRecordReadFailure {
    Protection(PhysicalReadProtectionDenial),
    Open(RecordReadError),
    Stream(RecordStreamFailure),
    /// The anchored record failed frame or digest validation.
    Damaged(AnchoredRecordDamage),
    /// The payload exceeds the caller's limit; nothing was allocated for it.
    LimitExceeded {
        payload_bytes: u64,
    },
}

/// Why an anchored record is not the value its anchor selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchoredRecordDamage {
    Truncated,
    WrongMagic,
    /// The frame names a different anchor.
    WrongAnchor,
    /// The frame's generation is not the anchor's.
    WrongGeneration,
    /// The record is longer than its frame declares.
    TrailingBytes,
    /// The whole record does not hash to the anchor's record digest.
    RecordDigestMismatch,
    /// The payload does not hash to the frame's payload digest.
    PayloadDigestMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AnchoredRecordsDenial {
    ServingRequiresInspection,
}

/// Serializes anchored-record publications and remembers, per name, those
/// whose fate is indeterminate until the runtime is reopened. Other names
/// stay publishable: an unresolved value never orders another name's value.
#[derive(Debug, Default)]
pub(in crate::physical_runtime) struct AnchoredRecordState {
    unresolved: BTreeSet<RecordAnchorName>,
}

/// Borrowed access to the home's anchored records.
pub struct AnchoredRecords<'runtime> {
    runtime: &'runtime ServingPhysicalRuntime,
}

impl<'runtime> AnchoredRecords<'runtime> {
    pub(in crate::physical_runtime) const fn new(
        runtime: &'runtime ServingPhysicalRuntime,
    ) -> Self {
        Self { runtime }
    }
}
