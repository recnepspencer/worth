use super::{
    StoreRecoveryBindingFreshness, StoreRecoveryBindingSampleDenial, StoreRecoveryOperationEvidence,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreRecoveryBindingSampleFailure {
    denial: StoreRecoveryBindingSampleDenial,
    operation_bindings_observed: u64,
    freshness_retained: u64,
    freshness_expired: u64,
    wal_members_observed: u64,
    redo_bytes_observed: u64,
    allocation: Option<super::sampling::allocation::StoreRecoveryBindingSampleAllocationDenial>,
}

impl StoreRecoveryBindingSampleFailure {
    pub const fn denial(&self) -> StoreRecoveryBindingSampleDenial {
        self.denial
    }
    pub const fn operation_bindings_observed(&self) -> u64 {
        self.operation_bindings_observed
    }
    pub const fn freshness_retained(&self) -> u64 {
        self.freshness_retained
    }
    pub const fn freshness_expired(&self) -> u64 {
        self.freshness_expired
    }
    pub const fn wal_members_observed(&self) -> u64 {
        self.wal_members_observed
    }
    pub const fn redo_bytes_observed(&self) -> u64 {
        self.redo_bytes_observed
    }

    pub fn allocation_denial(
        &self,
    ) -> Option<&super::sampling::allocation::StoreRecoveryBindingSampleAllocationDenial> {
        self.allocation.as_ref()
    }

    pub(super) fn with_allocation_denial(
        mut self,
        cause: super::sampling::allocation::StoreRecoveryBindingSampleAllocationDenial,
    ) -> Self {
        self.denial = StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit;
        self.allocation = Some(cause);
        self
    }
}

pub(super) fn empty_failure(
    denial: StoreRecoveryBindingSampleDenial,
) -> StoreRecoveryBindingSampleFailure {
    StoreRecoveryBindingSampleFailure {
        denial,
        operation_bindings_observed: 0,
        freshness_retained: 0,
        freshness_expired: 0,
        wal_members_observed: 0,
        redo_bytes_observed: 0,
        allocation: None,
    }
}

pub(super) fn sample_failure_from_evidence<'evidence>(
    denial: StoreRecoveryBindingSampleDenial,
    operations: impl Iterator<Item = &'evidence StoreRecoveryOperationEvidence>,
    wal_members_observed: usize,
    redo_bytes_observed: u64,
) -> StoreRecoveryBindingSampleFailure {
    let (count, freshness_retained, freshness_expired) = operations.fold(
        (0_u64, 0_u64, 0_u64),
        |(count, retained, expired), evidence| match evidence.freshness {
            StoreRecoveryBindingFreshness::Retained => (count + 1, retained + 1, expired),
            StoreRecoveryBindingFreshness::ExpiredAtSelectedCheckpoint => {
                (count + 1, retained, expired + 1)
            }
        },
    );
    StoreRecoveryBindingSampleFailure {
        denial,
        operation_bindings_observed: count
            + u64::from(denial == StoreRecoveryBindingSampleDenial::OperationBindingLimit),
        freshness_retained,
        freshness_expired,
        wal_members_observed: wal_members_observed as u64,
        redo_bytes_observed,
        allocation: None,
    }
}
