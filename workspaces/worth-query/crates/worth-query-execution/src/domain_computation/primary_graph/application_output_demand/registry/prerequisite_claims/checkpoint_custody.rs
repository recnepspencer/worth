//! Non-executable custody of exact checkpoint roots verified by native readers.
use std::sync::Arc;

use super::super::{DemandRegistryState, RequiredOutputCustodyCapacity};
use super::prerequisite_denials::{capacity_denial, work_denial};
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::InvalidationEditAdmission, RecordedSettlementIdentity,
};
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;

/// The reader and lineage retain the complete funded facts and output witness.
/// This claim pins their exact settlement address through prerequisite custody.
/// It has no source epoch, producer, refresh route, or Ready delivery authority.
pub(in crate::domain_computation::primary_graph::application_output_demand::registry) struct CheckpointPrerequisiteClaims
{
    identities: Vec<Arc<RecordedSettlementIdentity>>,
    // Field order destroys the backing before refunding its final-owner charge.
    _capacity: RequiredOutputCustodyCapacity,
}

impl CheckpointPrerequisiteClaims {
    pub(super) fn prepare(
        maximum: usize,
        state: &DemandRegistryState,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let bytes = maximum
            .checked_mul(std::mem::size_of::<Arc<RecordedSettlementIdentity>>())
            .ok_or_else(capacity_denial)?;
        // Pay each retained Arc and its eventual final destruction together.
        // Cancellation and branch retirement may have no later request meter.
        let lifetime_work = maximum.checked_mul(2).ok_or_else(work_denial)?;
        admission
            .charge_external_work(lifetime_work as u64)
            .map_err(|_| work_denial())?;
        admission
            .admit_read_scratch(bytes as u64)
            .map_err(|_| capacity_denial())?;
        let capacity = state.reserve_required_custody_capacity(bytes)?;
        let mut identities = Vec::new();
        identities
            .try_reserve_exact(maximum)
            .map_err(|_| capacity_denial())?;
        if identities.capacity() != maximum {
            return Err(capacity_denial());
        }
        Ok(Self {
            identities,
            _capacity: capacity,
        })
    }

    pub(super) fn retain_verified(&mut self, identity: &Arc<RecordedSettlementIdentity>) {
        assert!(self.identities.len() < self.identities.capacity());
        self.identities.push(Arc::clone(identity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    /// Storage funding alone grants no source or Current authority. The native
    /// reopened-consumer court proves the admission gate separately.
    #[test]
    fn cancelled_checkpoint_custody_refunds_its_exact_backing() {
        let mut state = DemandRegistryState::default();
        let bytes = 2 * std::mem::size_of::<Arc<RecordedSettlementIdentity>>();
        state.required_budget_bytes = bytes;
        let mut admission = InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: 1_000_000,
                maximum_preparation_bytes: 8 * 1024 * 1024,
            },
        );
        let claims = CheckpointPrerequisiteClaims::prepare(2, &state, &mut admission).unwrap();
        assert_eq!(
            state
                .required_custody_retained_bytes
                .load(Ordering::Acquire),
            bytes
        );
        assert!(CheckpointPrerequisiteClaims::prepare(1, &state, &mut admission).is_err());
        drop(claims);
        assert_eq!(
            state
                .required_custody_retained_bytes
                .load(Ordering::Acquire),
            0
        );
        assert!(CheckpointPrerequisiteClaims::prepare(2, &state, &mut admission).is_ok());
    }
}
