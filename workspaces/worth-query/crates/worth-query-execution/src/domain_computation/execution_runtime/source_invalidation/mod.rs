//! Query-owned capacity shared by every canonical source subscription in one
//! resource installation. Relational remains the publication authority.

mod retention;

use std::sync::Arc;

use retention::InvalidationRetentionLedger;
pub(in crate::domain_computation) use retention::RetainedInvalidationCapacity;

/// Explicit capacities for canonical delivery and derived invalidation state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInvalidationResourceInstallation {
    pub maximum_marking_work: u64,
    pub maximum_preparation_bytes: u64,
    pub maximum_retained_bytes: u64,
    pub maximum_retained_positions: usize,
}

impl WorthQueryInvalidationResourceInstallation {
    pub const fn bounded(
        maximum_marking_work: u64,
        maximum_preparation_bytes: u64,
        maximum_retained_bytes: u64,
        maximum_retained_positions: usize,
    ) -> Self {
        Self {
            maximum_marking_work,
            maximum_preparation_bytes,
            maximum_retained_bytes,
            maximum_retained_positions,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvalidationResourceDenial {
    ZeroMarkingWork,
    ZeroPreparationCapacity,
    ZeroRetentionCapacity,
    ZeroPositionCapacity,
    RetentionCapacityExhausted {
        requested: u64,
        retained: u64,
        maximum: u64,
    },
}

/// Shared managed retention custody. Cloning this binding shares capacity; it
/// never installs an independent allowance for another source or branch.
#[derive(Clone, Debug)]
pub struct WorthQueryInvalidationResources {
    installation: WorthQueryInvalidationResourceInstallation,
    retention: Arc<InvalidationRetentionLedger>,
}

impl WorthQueryInvalidationResources {
    pub fn install(
        installation: WorthQueryInvalidationResourceInstallation,
    ) -> Result<Self, WorthQueryInvalidationResourceDenial> {
        use WorthQueryInvalidationResourceDenial as Denial;
        if installation.maximum_marking_work == 0 {
            return Err(Denial::ZeroMarkingWork);
        }
        if installation.maximum_preparation_bytes == 0 {
            return Err(Denial::ZeroPreparationCapacity);
        }
        if installation.maximum_retained_bytes == 0 {
            return Err(Denial::ZeroRetentionCapacity);
        }
        if installation.maximum_retained_positions == 0 {
            return Err(Denial::ZeroPositionCapacity);
        }
        Ok(Self {
            installation,
            retention: Arc::new(InvalidationRetentionLedger::new(
                installation.maximum_retained_bytes,
            )),
        })
    }

    pub const fn installation(&self) -> WorthQueryInvalidationResourceInstallation {
        self.installation
    }

    pub fn retained_capacity_bytes(&self) -> u64 {
        self.retention.retained_bytes()
    }

    pub(in crate::domain_computation) fn reserve_retained_capacity(
        &self,
        bytes: u64,
    ) -> Result<RetainedInvalidationCapacity, WorthQueryInvalidationResourceDenial> {
        self.retention.reserve(bytes)
    }

    pub(in crate::domain_computation) const fn preflight_budget(
        &self,
    ) -> worth_relational::facade::mvcc::CompanionPreflightBudget {
        worth_relational::facade::mvcc::CompanionPreflightBudget {
            maximum_work_visits: self.installation.maximum_marking_work,
            maximum_preparation_bytes: self.installation.maximum_preparation_bytes,
        }
    }
}
