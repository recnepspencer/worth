use serde::{Deserialize, Serialize};

use crate::transactions::data::RecordRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// Separates entity and relation allocation domains within each partition.
pub enum RecordAllocationClass {
    Entity,
    Relation,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RecordAllocationOrigin {
    #[default]
    AppendFrontier,
    Reclaimed {
        prior_generation: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalRecordAllocation {
    ordinal: u64,
    record: RecordRef,
    #[serde(default)]
    origin: RecordAllocationOrigin,
}

impl CanonicalRecordAllocation {
    #[cfg(test)]
    pub(crate) fn new(ordinal: u64, record: RecordRef) -> Self {
        Self {
            ordinal,
            record,
            origin: RecordAllocationOrigin::AppendFrontier,
        }
    }

    pub(crate) fn with_origin(
        ordinal: u64,
        record: RecordRef,
        origin: RecordAllocationOrigin,
    ) -> Self {
        Self {
            ordinal,
            record,
            origin,
        }
    }

    pub(crate) fn ordinal(&self) -> u64 {
        self.ordinal
    }

    pub(crate) fn record(&self) -> &RecordRef {
        &self.record
    }

    pub(crate) const fn origin(&self) -> RecordAllocationOrigin {
        self.origin
    }

    pub(crate) fn class(&self) -> RecordAllocationClass {
        match self.record {
            RecordRef::Entity(_) => RecordAllocationClass::Entity,
            RecordRef::Relation(_) => RecordAllocationClass::Relation,
        }
    }
}
