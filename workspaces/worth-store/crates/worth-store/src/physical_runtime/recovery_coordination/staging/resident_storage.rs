//! Nested physical backing retained by staging settlement evidence.

use super::{
    CompletedPhysicalRecoveryStagingCommand, PhysicalRecoveryStagingCommandDenial,
    PhysicalRecoveryStagingCommandIndeterminate, PhysicalRecoveryStagingMaterialization,
    PhysicalRecoveryStagingMaterializationEvidence,
};

impl CompletedPhysicalRecoveryStagingCommand {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.materialization
            .owned_heap_bytes()?
            .checked_add(self.synchronization.owned_heap_bytes()?)
    }
}

impl PhysicalRecoveryStagingMaterialization {
    fn owned_heap_bytes(&self) -> Option<u64> {
        match self {
            Self::Created(value)
            | Self::RangeWritten(value)
            | Self::CompletedFromExactPrefix(value) => value.owned_heap_bytes(),
            Self::AlreadyMaterialized(value) => value.owned_heap_bytes(),
        }
    }
}

impl PhysicalRecoveryStagingMaterializationEvidence {
    fn owned_heap_bytes(&self) -> Option<u64> {
        match self {
            Self::Performed(value) => value.owned_heap_bytes(),
            Self::PhysicallyCompleted(value) => value.owned_heap_bytes(),
        }
    }
}

impl PhysicalRecoveryStagingCommandDenial {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.materialization
            .as_ref()
            .map_or(Some(0), |value| value.owned_heap_bytes())
    }
}

impl PhysicalRecoveryStagingCommandIndeterminate {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        match self {
            Self::Materialization { physical, .. } => physical.owned_heap_bytes(),
            Self::Synchronization {
                physical,
                materialization,
                ..
            } => physical
                .owned_heap_bytes()?
                .checked_add(materialization.owned_heap_bytes()?),
            Self::Scheduler {
                materialization,
                synchronization,
                ..
            }
            | Self::Signal {
                materialization,
                synchronization,
                ..
            }
            | Self::Yieldpoint {
                materialization,
                synchronization,
                ..
            } => materialization
                .as_ref()
                .map_or(Some(0), |value| value.owned_heap_bytes())?
                .checked_add(
                    synchronization
                        .as_ref()
                        .map_or(Some(0), |value| value.owned_heap_bytes())?,
                ),
        }
    }
}
