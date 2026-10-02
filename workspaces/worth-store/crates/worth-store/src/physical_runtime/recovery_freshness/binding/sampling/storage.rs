//! All vector capacities are constructed under the caller's live reservation.

use super::super::{
    failure::sample_failure_from_evidence,
    manifest_cleanup::ManifestCleanupObservations,
    operations::{reserve_vector, RecoveryBindingOperations},
    tier_epoch::TierEpochObservations,
    StoreRecoveryBindingSampleDenial as Denial, StoreRecoveryBindingSampleFailure,
    StoreRecoveryRetirementObligation, StoreRecoveryWalMember,
};
use super::{
    allocation::{SamplingBacking, StoreRecoveryBindingSampleAllocationDenial as AllocationDenial},
    census::SamplingCapacity,
    group_validation::RecoveryWalGroupBinding,
};
use crate::physical_runtime::durability::RetirementRecord;
use worth_store_wal::WalLsnRange;

pub(super) struct SamplingStorage {
    pub(super) operations: RecoveryBindingOperations,
    pub(super) members: Vec<StoreRecoveryWalMember>,
    pub(super) groups: Vec<RecoveryWalGroupBinding>,
    pub(super) group_scratch: Vec<usize>,
    pub(super) retirement_records: Vec<(usize, RetirementRecord)>,
    pub(super) retirement_output: Vec<StoreRecoveryRetirementObligation>,
    pub(super) copies: Vec<(WalLsnRange, Vec<u8>)>,
    pub(super) cleanups: ManifestCleanupObservations,
    pub(super) cleanup_count: usize,
    pub(super) tier: TierEpochObservations,
    pub(super) redo_bytes: u64,
}

impl SamplingStorage {
    pub(super) fn prepare(
        capacity: SamplingCapacity,
        maximum: u64,
        cleanup_limit: u64,
        backing: &SamplingBacking,
    ) -> Result<Self, AllocationDenial> {
        if backing.bytes() < capacity.requested {
            return Err(AllocationDenial::BackingMismatch);
        }
        let operations = match backing.grant() {
            Some(grant) => RecoveryBindingOperations::prepare(capacity.operations, maximum, grant)?,
            None if capacity.requested == 0 => RecoveryBindingOperations::empty(maximum),
            None => return Err(AllocationDenial::BackingMismatch),
        };
        let members = reserve_vector(capacity.members)?;
        let groups = reserve_vector(capacity.members)?;
        let group_scratch = reserve_vector(capacity.members)?;
        let retirement_records = reserve_vector(capacity.retirements)?;
        let retirement_output = reserve_vector(capacity.retirements)?;
        let copies = reserve_vector(capacity.copies)?;
        let frames = reserve_vector(capacity.cleanups)?;
        let cleanups = ManifestCleanupObservations::prepare(frames, cleanup_limit)
            .map_err(|_| AllocationDenial::BackingMismatch)?;
        Ok(Self {
            operations,
            members,
            groups,
            group_scratch,
            retirement_records,
            retirement_output,
            copies,
            cleanups,
            cleanup_count: 0,
            tier: TierEpochObservations::default(),
            redo_bytes: 0,
        })
    }

    pub(super) fn failure(&self, denial: Denial) -> StoreRecoveryBindingSampleFailure {
        sample_failure_from_evidence(
            denial,
            self.operations.evidence().iter(),
            self.members.len(),
            self.redo_bytes,
        )
    }

    pub(super) fn allocation_failure(
        &self,
        cause: AllocationDenial,
    ) -> StoreRecoveryBindingSampleFailure {
        self.failure(Denial::RecoveryMemoryLimit)
            .with_allocation_denial(cause)
    }
}

pub(super) fn copy_bytes(bytes: &[u8]) -> Result<Vec<u8>, AllocationDenial> {
    let mut retained = reserve_vector(bytes.len())?;
    retained.extend_from_slice(bytes);
    Ok(retained)
}

pub(super) fn push<T>(vector: &mut Vec<T>, value: T) -> Result<(), Denial> {
    if vector.len() == vector.capacity() {
        return Err(Denial::RecoveryMemoryLimit);
    }
    vector.push(value);
    Ok(())
}
