//! Ordered journals: named, position-keyed sequences of opaque owner
//! artifacts that one Store home makes durable. Relational's canonical commits
//! and its branch transitions are two such journals, each with its own
//! contiguous sequence under its own root anchor.
//!
//! Store treats each payload as opaque owner bytes. A publication is admitted
//! only at its journal's next position, writes one journal record and advances
//! that journal's root anchor in the same root generation, so the commit point
//! is the durable selection of that root. Positions are the attempt identity:
//! after recovery, `resolve` answers from durable state alone.
//!
//! The journal name is bound everywhere a record is reached: the record frame
//! carries it, every header read (head, spine step, payload read) requires it,
//! and the idempotency material includes it. A record read under another
//! journal's name is damage, never an entry.

use std::collections::BTreeMap;

use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationPreparationOutcome,
    PhysicalReadProtectionDenial, PhysicalRecordId, RecordReadError, RecordStreamFailure,
    ServingPhysicalRuntime,
};

mod frame;
mod lookup;
mod name;
mod publication;

pub(in crate::physical_runtime) use name::is_ordered_journal_name;
pub use name::{
    OrderedJournalName, OrderedJournalNameDenial, RELATIONAL_BRANCH_TRANSITIONS,
    RELATIONAL_CANONICAL_COMMITS,
};

/// One durable journal entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderedJournalEntry {
    position: u64,
    record: PhysicalRecordId,
    payload_sha256: [u8; 32],
    payload_bytes: u64,
}

impl OrderedJournalEntry {
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
pub struct OrderedJournalPayload {
    entry: OrderedJournalEntry,
    bytes: Vec<u8>,
}

impl OrderedJournalPayload {
    pub const fn entry(&self) -> OrderedJournalEntry {
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
pub enum OrderedJournalResolution {
    /// The journal holds this position with this payload.
    Committed(OrderedJournalEntry),
    /// The journal's head is below this position; nothing was committed there.
    NotCommitted { head: u64 },
    /// The position holds a different payload.
    Conflict(OrderedJournalEntry),
    /// A publication of this position to this journal has an indeterminate
    /// fate in this runtime incarnation. Recovery establishes it; nothing else
    /// may.
    Unresolved,
}

/// One requested publication at `position`, which must be the head plus one.
pub struct OrderedJournalPublication {
    position: u64,
    payload: Vec<u8>,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
}

impl OrderedJournalPublication {
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
pub enum OrderedJournalPublished {
    /// This call made the entry durable.
    Committed(OrderedJournalEntry),
    /// The identical entry was already durable; nothing was written.
    AlreadyCommitted(OrderedJournalEntry),
}

impl OrderedJournalPublished {
    pub const fn entry(&self) -> OrderedJournalEntry {
        match self {
            Self::Committed(entry) | Self::AlreadyCommitted(entry) => *entry,
        }
    }
}

/// A publication that did not make its entry durable. Every variant states its
/// effect posture: only `Indeterminate` may have had an effect.
#[non_exhaustive]
pub enum OrderedJournalPublicationFailure {
    EmptyPayload,
    /// The position is not the head plus one. No effect.
    PositionGap {
        head: u64,
        requested: u64,
    },
    /// The position already holds a different payload. No effect.
    PositionConflict(OrderedJournalEntry),
    /// The journal's first entry needs an anchor and the root's anchor table
    /// is full. No effect.
    AnchorTableFull,
    /// An earlier publication to this journal has an indeterminate fate;
    /// recovery must establish it before the journal admits another. No
    /// effect.
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
    /// The mutation completed but did not report exactly one record. It may
    /// be durable; `resolve` establishes its fate after reopen.
    CompletedWithoutRecord,
    /// Reading the current head failed before any effect.
    Read(OrderedJournalReadFailure),
}

impl std::fmt::Debug for OrderedJournalPublicationFailure {
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
            Self::AnchorTableFull => f.write_str("AnchorTableFull"),
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
            Self::CompletedWithoutRecord => f.write_str("CompletedWithoutRecord"),
            Self::Read(failure) => f.debug_tuple("Read").field(failure).finish(),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum OrderedJournalReadFailure {
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
pub enum OrderedJournalDenial {
    ServingRequiresInspection,
}

/// Serializes publications across journals and remembers, per journal, the
/// position of a publication whose fate is indeterminate until the runtime is
/// reopened. Other journals stay publishable: an unresolved entry orders only
/// its own journal.
#[derive(Debug, Default)]
pub(in crate::physical_runtime) struct OrderedJournalState {
    unresolved: BTreeMap<OrderedJournalName, u64>,
}

/// Borrowed access to one of the home's ordered journals.
pub struct OrderedJournal<'runtime> {
    runtime: &'runtime ServingPhysicalRuntime,
    name: OrderedJournalName,
}

impl<'runtime> OrderedJournal<'runtime> {
    pub(in crate::physical_runtime) const fn new(
        runtime: &'runtime ServingPhysicalRuntime,
        name: OrderedJournalName,
    ) -> Self {
        Self { runtime, name }
    }

    pub const fn name(&self) -> OrderedJournalName {
        self.name
    }
}
