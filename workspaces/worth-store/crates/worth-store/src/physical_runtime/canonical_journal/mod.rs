//! The canonical commit journal: the ordered, position-keyed sequence of owner
//! commit artifacts that one Store home makes durable.
//!
//! Store treats each payload as opaque owner bytes. A publication is admitted
//! only at the next position, writes one journal record and advances the
//! journal's root anchor in the same root generation, so the commit point is
//! the durable selection of that root. Positions are the attempt identity: after
//! recovery, `resolve` answers from durable state alone.

use worth_store_physical_format::RecordAnchorName;

use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationPreparationOutcome,
    PhysicalReadProtectionDenial, PhysicalRecordId, RecordReadError, RecordStreamFailure,
    ServingPhysicalRuntime,
};

mod frame;
mod lookup;
mod publication;

/// The anchor of the journal holding Relational's canonical commit artifacts.
pub const RELATIONAL_CANONICAL_COMMITS: &str = "relational.canonical-commits";

/// One durable journal entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalCommitEntry {
    position: u64,
    record: PhysicalRecordId,
    payload_sha256: [u8; 32],
    payload_bytes: u64,
}

impl CanonicalCommitEntry {
    pub const fn position(&self) -> u64 {
        self.position
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

/// The owner payload of one entry, verified against its recorded digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCommitPayload {
    entry: CanonicalCommitEntry,
    bytes: Vec<u8>,
}

impl CanonicalCommitPayload {
    pub const fn entry(&self) -> CanonicalCommitEntry {
        self.entry
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// What durable state says about a position and payload digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalCommitResolution {
    /// The journal holds this position with this payload.
    Committed(CanonicalCommitEntry),
    /// The journal's head is below this position; nothing was committed there.
    NotCommitted { head: u64 },
    /// The position holds a different payload.
    Conflict(CanonicalCommitEntry),
    /// A publication of this position has an indeterminate fate in this
    /// runtime incarnation. Recovery establishes it; nothing else may.
    Unresolved,
}

/// One requested publication at `position`, which must be the head plus one.
pub struct CanonicalCommitPublication {
    position: u64,
    payload: Vec<u8>,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
}

impl CanonicalCommitPublication {
    pub fn new(
        position: u64,
        payload: Vec<u8>,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
    ) -> Self {
        Self {
            position,
            payload,
            placement,
            deadline,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalCommitPublished {
    /// This call made the entry durable.
    Committed(CanonicalCommitEntry),
    /// The identical entry was already durable; nothing was written.
    AlreadyCommitted(CanonicalCommitEntry),
}

impl CanonicalCommitPublished {
    pub const fn entry(&self) -> CanonicalCommitEntry {
        match self {
            Self::Committed(entry) | Self::AlreadyCommitted(entry) => *entry,
        }
    }
}

/// A publication that did not make its entry durable. Every variant states its
/// effect posture: only `Indeterminate` may have had an effect.
#[non_exhaustive]
pub enum CanonicalCommitPublicationFailure {
    EmptyPayload,
    /// The position is not the head plus one. No effect.
    PositionGap {
        head: u64,
        requested: u64,
    },
    /// The position already holds a different payload. No effect.
    PositionConflict(CanonicalCommitEntry),
    /// An earlier publication's fate is indeterminate; recovery must establish
    /// it before the journal admits another. No effect.
    Unresolved {
        position: u64,
    },
    /// The journal record could not form an admissible batch. No effect.
    Batch(crate::physical_runtime::RecordAppendDenial),
    Idempotency(PhysicalMutationIdempotencyIssuanceDenial),
    /// Physical preparation refused before any WAL effect.
    Preparation(PhysicalMutationPreparationOutcome),
    ProvenNoEffect(crate::physical_runtime::ProvenNoEffectPhysicalMutation),
    /// The publication may or may not be durable.
    Indeterminate(crate::physical_runtime::IndeterminatePhysicalMutation),
    /// Reading the current head failed before any effect.
    Read(CanonicalCommitReadFailure),
}

impl std::fmt::Debug for CanonicalCommitPublicationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPayload => f.write_str("EmptyPayload"),
            Self::PositionGap { head, requested } => f
                .debug_struct("PositionGap")
                .field("head", head)
                .field("requested", requested)
                .finish(),
            Self::PositionConflict(entry) => {
                f.debug_tuple("PositionConflict").field(entry).finish()
            }
            Self::Unresolved { position } => f
                .debug_struct("Unresolved")
                .field("position", position)
                .finish(),
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
            Self::Read(failure) => f.debug_tuple("Read").field(failure).finish(),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CanonicalCommitReadFailure {
    Protection(PhysicalReadProtectionDenial),
    Open(RecordReadError),
    Stream(RecordStreamFailure),
    /// A journal record failed frame, linkage or digest validation.
    Damaged,
    /// The position is beyond the journal head.
    NotFound {
        head: u64,
    },
    /// The payload exceeds the caller's limit.
    LimitExceeded {
        payload_bytes: u64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CanonicalCommitJournalDenial {
    ServingRequiresInspection,
}

/// Serializes publications and remembers an indeterminate one until reopen.
#[derive(Debug, Default)]
pub(in crate::physical_runtime) struct CanonicalJournalState {
    unresolved: Option<u64>,
}

/// Borrowed access to the home's canonical commit journal.
pub struct CanonicalCommitJournal<'runtime> {
    runtime: &'runtime ServingPhysicalRuntime,
    name: RecordAnchorName,
}

impl<'runtime> CanonicalCommitJournal<'runtime> {
    pub(in crate::physical_runtime) fn relational(
        runtime: &'runtime ServingPhysicalRuntime,
    ) -> Self {
        Self {
            runtime,
            name: RecordAnchorName::new(RELATIONAL_CANONICAL_COMMITS)
                .expect("the journal name is a valid anchor name"),
        }
    }
}
