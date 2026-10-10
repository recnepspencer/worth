//! Query-owned capacity shared by every canonical source subscription in one
//! resource installation. Relational remains the publication authority.

mod native_reservation_observation;
mod retention;
pub(in crate::domain_computation) use native_reservation_observation::NativeRetainedKind;

use std::sync::Arc;

pub(in crate::domain_computation) use retention::InvalidationRetentionLedger;
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
            retention: InvalidationRetentionLedger::install(installation.maximum_retained_bytes),
        })
    }

    pub const fn installation(&self) -> WorthQueryInvalidationResourceInstallation {
        self.installation
    }

    /// Live Native hint, branch, and completion reservations, including tickets.
    #[doc(hidden)]
    pub fn native_retained_allocations_for_test(&self) -> [(usize, u64); 3] {
        self.retention.native.snapshot()
    }
    /// Each row reads one live ticket in this ledger; provenance adds no custody.
    #[doc(hidden)]
    pub fn retained_custody_breakdown_for_test(&self) -> Vec<(usize, &'static str, u32, u64)> {
        self.retention.native.breakdown()
    }
    // Law 2, scope: the fault arms only this receiver's real retention ledger.
    pub(in crate::domain_computation) fn arm_delivery_exhaustion(&self) {
        use std::sync::atomic::Ordering;
        if crate::domain_computation::execution_observation::is_installed() {
            let receiver = Arc::downgrade(&self.retention);
            crate::domain_computation::execution_observation::register_owned_cleanup(move || {
                if let Some(receiver) = receiver.upgrade() {
                    receiver.exhaust_delivery.store(false, Ordering::Release);
                }
            });
        }
        self.retention
            .exhaust_delivery
            .store(true, Ordering::Release);
    }
    pub(in crate::domain_computation) fn take_delivery_exhaustion(&self) -> bool {
        self.retention
            .exhaust_delivery
            .swap(false, std::sync::atomic::Ordering::AcqRel)
    }
    // Runtime construction, including resources prepared before its fixture guard.
    pub(in crate::domain_computation) fn attach_runtime_observations(&self) {
        self.retention.attach_native_observation();
        crate::domain_computation::invalidation_equivalence_control::attach_constructed_receiver(
            &self.retention,
        );
    }
    pub(in crate::domain_computation) fn retention_receiver(
        &self,
    ) -> &Arc<InvalidationRetentionLedger> {
        &self.retention
    }
    // Law 2, scope: unarmed verification costs one receiver atomic read.
    pub(in crate::domain_computation) fn invalidation_equivalence_is_armed(&self) -> bool {
        self.retention
            .oracle_guards
            .load(std::sync::atomic::Ordering::Acquire)
            != 0
    }
    pub(in crate::domain_computation) fn record_invalidation_shadow_check(&self) {
        self.retention
            .oracle_checks
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    pub(in crate::domain_computation) fn consumed_capacity_observation(
        &self,
        runtime: u64,
    ) -> (usize, u64) {
        self.retention.native.consumed(runtime)
    }
    pub fn retained_capacity_bytes(&self) -> u64 {
        self.retention.retained_bytes()
    }

    /// Reserve live custody within the hard ceiling. Predecessors remain
    /// charged until their last holder actually releases them.
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

#[cfg(test)]
mod control_scope_tests {
    use super::*;
    use crate::domain_computation::execution_observation::WorthQueryExecutionObservation;
    #[test]
    fn a_delivery_fault_cannot_cross_receiver_ledgers() {
        let _guard = WorthQueryExecutionObservation::install_on_this_thread();
        let installation = WorthQueryInvalidationResourceInstallation::bounded(1, 1, 17, 1);
        let controlled = WorthQueryInvalidationResources::install(installation).unwrap();
        let other = WorthQueryInvalidationResources::install(installation).unwrap();
        controlled.arm_delivery_exhaustion();
        assert!(!other.take_delivery_exhaustion());
        assert!(controlled.take_delivery_exhaustion());
        assert!(!controlled.take_delivery_exhaustion());
    }
    #[test]
    fn dropping_an_unreached_delivery_arm_clears_its_receiver() {
        let guard = WorthQueryExecutionObservation::install_on_this_thread();
        let installation = WorthQueryInvalidationResourceInstallation::bounded(1, 1, 17, 1);
        let controlled = WorthQueryInvalidationResources::install(installation).unwrap();
        controlled.arm_delivery_exhaustion();
        drop(guard);
        assert!(!controlled.take_delivery_exhaustion());
    }
}
