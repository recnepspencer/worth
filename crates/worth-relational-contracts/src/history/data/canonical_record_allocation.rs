use serde::{Deserialize, Serialize};

use crate::transactions::data::primitives::RecordRef;

pub use super::record_allocation_class::RecordAllocationClass;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordAllocationOrigin {
    #[default]
    AppendFrontier,
    Reclaimed {
        prior_generation: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalRecordAllocation {
    ordinal: u64,
    record: RecordRef,
    #[serde(default)]
    origin: RecordAllocationOrigin,
}

impl CanonicalRecordAllocation {
    pub(crate) fn new(ordinal: u64, record: RecordRef) -> Self {
        Self::with_origin(ordinal, record, RecordAllocationOrigin::AppendFrontier)
    }

    pub fn with_origin(ordinal: u64, record: RecordRef, origin: RecordAllocationOrigin) -> Self {
        Self {
            ordinal,
            record,
            origin,
        }
    }

    pub fn ordinal(&self) -> u64 {
        self.ordinal
    }

    pub fn record(&self) -> &RecordRef {
        &self.record
    }

    pub const fn origin(&self) -> RecordAllocationOrigin {
        self.origin
    }

    pub fn class(&self) -> RecordAllocationClass {
        match self.record {
            RecordRef::Entity(_) => RecordAllocationClass::Entity,
            RecordRef::Relation(_) => RecordAllocationClass::Relation,
        }
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait CanonicalRecordAllocationFixturePort: Sized {
    fn canonical_allocation_for_test(ordinal: u64, record: RecordRef) -> Self;
}

impl CanonicalRecordAllocationFixturePort for CanonicalRecordAllocation {
    fn canonical_allocation_for_test(ordinal: u64, record: RecordRef) -> Self {
        CanonicalRecordAllocation::new(ordinal, record)
    }
}
