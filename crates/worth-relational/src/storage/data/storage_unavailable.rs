use serde::{Deserialize, Serialize};

use crate::identity::data::{LineageId, PartitionId};
use crate::indexes::data::DerivedIndexId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::symbols::data::Symbol;

/// A committed value could not be made resident. This is never record absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalStorageUnavailable {
    pub subject: RelationalStorageSubject,
    pub cause: RelationalStorageUnavailableCause,
}

/// The committed region, journal envelope, or page whose readmission failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalStorageSubject {
    Region(PartitionId),
    Envelope(PatchStreamPosition),
    Symbols { first: Symbol, last: Symbol },
    Lineage { first: LineageId, last: LineageId },
    Index(DerivedIndexId),
}

/// Why the cold source or its authenticated contents could not be admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalStorageUnavailableCause {
    /// The selected durable directory names content proven absent.
    MissingMember,
    /// The source refused or failed without proving durable damage. Retryable.
    SourceUnavailable,
    Damaged,
    IntegrityMismatch,
    LimitExceeded {
        length: u64,
        limit: u64,
    },
    MemoryExhausted,
}

impl RelationalStorageUnavailableCause {
    pub(crate) fn is_proven_damage(self) -> bool {
        matches!(
            self,
            Self::MissingMember | Self::Damaged | Self::IntegrityMismatch
        )
    }
}
