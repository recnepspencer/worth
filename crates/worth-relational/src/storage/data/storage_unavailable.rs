use serde::{Deserialize, Serialize};

use crate::identity::data::{LineageId, PartitionId};
use crate::indexes::data::DerivedIndexId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::symbols::data::Symbol;

/// A committed value could not be made resident. This is never record absence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalStorageUnavailable {
    pub subject: RelationalStorageSubject,
    pub cause: RelationalStorageUnavailableCause,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalStorageSubject {
    Region(PartitionId),
    Envelope(PatchStreamPosition),
    Symbols { first: Symbol, last: Symbol },
    Lineage { first: LineageId, last: LineageId },
    Index(DerivedIndexId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalStorageUnavailableCause {
    SourceUnavailable,
    Damaged,
    IntegrityMismatch,
    LimitExceeded { length: u64, limit: u64 },
    MemoryExhausted,
}
