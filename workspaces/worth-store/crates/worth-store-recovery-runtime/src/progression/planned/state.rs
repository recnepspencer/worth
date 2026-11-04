use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_recovery_physics::{
    ImmutablePhysicalRedoPlan, RecoveryPlanCost, RecoveryPlanningCounters,
};

use crate::entry::{AdmittedPlatformAuthority, PhysicalRecoverySourceDenial};
use crate::handoff::RecoveryOperationFateSet;
use crate::orchestration::{RecoveryCoordination, ResidentSourceSelection};

use super::{
    PhysicalRecoveryDiscoveryCounters, RecoveryIntegrityEvidence, RecoveryPublicationPlan,
    RecoveryQuiescencePlan, RecoveryStagingLayoutPlan,
};

pub(super) struct PlannedRecoveryState {
    pub(super) authority: AdmittedPlatformAuthority,
    pub(super) coordination: RecoveryCoordination,
    pub(super) selection: ResidentSourceSelection,
    pub(super) custody: crate::progression::PlanningCustody,
    pub(super) verified_selected_tier_custody:
        Option<worth_store_recovery_physics::VerifiedSelectedTierEpochCustody>,
    pub(super) discovery_counters: PhysicalRecoveryDiscoveryCounters,
    pub(super) root_protocol_denials: Vec<PhysicalRecoverySourceDenial>,
    pub(super) integrity: RecoveryIntegrityEvidence,
    pub(super) freshness: StoreRecoveryBindingFreshnessSample,
    pub(super) fates: RecoveryOperationFateSet,
    pub(super) redo: ImmutablePhysicalRedoPlan,
    pub(super) plan_cost: RecoveryPlanCost,
    pub(super) planning_counters: RecoveryPlanningCounters,
    pub(super) root_protocol_counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    pub(super) staging: RecoveryStagingLayoutPlan,
    pub(super) publication: RecoveryPublicationPlan,
    pub(super) quiescence: RecoveryQuiescencePlan,
    pub(super) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;

/// One admitted payload slot; no caller can publish an empty planned receiver.
pub(crate) struct PlannedStateBacking {
    values: Vec<PlannedRecoveryState>,
    staged: crate::progression::StagedStateBacking,
    input: crate::orchestration::RecoveryStagingInputBacking,
    namespace: crate::progression::NamespaceDurableBacking,
    publication: crate::orchestration::PublicationStateBacking,
    reopened: crate::progression::ReopenedBacking,
}

impl PlannedStateBacking {
    pub(crate) fn prepare(
        allowance: &mut super::PlanningResidentAllowance,
        extraction_bytes: u64,
    ) -> Result<Self, super::PlanningMemoryDenial> {
        let payload = super::PlanningResidentAllowance::slot_bytes::<PlannedRecoveryState>(1)?;
        let staged = super::PlanningResidentAllowance::slot_bytes::<
            crate::progression::StagedRecoveryState,
        >(1)?;
        let overflow = || super::PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX };
        let scratch = construction_scratch(extraction_bytes).ok_or_else(overflow)?;
        let input_bytes = super::PlanningResidentAllowance::slot_bytes::<
            crate::orchestration::RecoveryStagingInputState,
        >(1)?;
        let namespace_bytes = super::PlanningResidentAllowance::slot_bytes::<
            crate::progression::NamespaceDurablePayload,
        >(1)?;
        let publication_bytes = super::PlanningResidentAllowance::slot_bytes::<
            crate::orchestration::PublicationPayload,
        >(1)?;
        let reopened_bytes =
            super::PlanningResidentAllowance::slot_bytes::<crate::progression::ReopenedPayload>(1)?;
        let backing = payload
            .checked_add(staged)
            .and_then(|bytes| bytes.checked_add(input_bytes))
            .and_then(|bytes| bytes.checked_add(namespace_bytes))
            .and_then(|bytes| bytes.checked_add(publication_bytes))
            .and_then(|bytes| bytes.checked_add(reopened_bytes))
            .ok_or_else(overflow)?;
        allowance.transient(backing.checked_add(scratch).ok_or_else(overflow)?)?;
        let values = allowance.reserve::<PlannedRecoveryState>(1)?;
        let staged = crate::progression::StagedStateBacking::prepare(allowance)?;
        let input = crate::orchestration::RecoveryStagingInputBacking::prepare(allowance)?;
        let namespace = crate::progression::NamespaceDurableBacking::prepare(allowance)?;
        let publication = crate::orchestration::PublicationStateBacking::prepare(allowance)?;
        let reopened = crate::progression::ReopenedBacking::prepare(allowance)?;
        allowance.transient(scratch)?;
        Ok(Self {
            values,
            staged,
            input,
            namespace,
            publication,
            reopened,
        })
    }

    pub(super) fn finish(mut self, state: PlannedRecoveryState) -> super::PlannedPhysicalRecovery {
        assert!(self.values.is_empty() && self.values.capacity() >= 1);
        self.values.push(state);
        super::PlannedPhysicalRecovery { state: self }
    }

    pub(super) const fn state(&self) -> &PlannedRecoveryState {
        &self.values.as_slice()[0]
    }

    pub(super) fn into_state(mut self) -> PlannedRecoveryState {
        assert_eq!(
            self.values.len(),
            1,
            "sealed payload owner contains one value"
        );
        self.values.remove(0)
    }

    pub(super) fn into_staging_parts(
        mut self,
    ) -> (
        PlannedRecoveryState,
        crate::orchestration::RecoveryStagingInputBacking,
        crate::progression::StagedStateBacking,
        crate::progression::NamespaceDurableBacking,
        crate::orchestration::PublicationStateBacking,
        crate::progression::ReopenedBacking,
    ) {
        assert_eq!(
            self.values.len(),
            1,
            "sealed planned owner contains one state"
        );
        let state = self.values.remove(0);
        (
            state,
            self.input,
            self.staged,
            self.namespace,
            self.publication,
            self.reopened,
        )
    }
}

fn construction_scratch(extraction_bytes: u64) -> Option<u64> {
    let planned = std::mem::size_of::<PlannedRecoveryState>() as u64;
    let staged = std::mem::size_of::<crate::progression::StagedRecoveryState>() as u64;
    let input = std::mem::size_of::<crate::orchestration::RecoveryStagingInputState>() as u64;
    let namespace = std::mem::size_of::<crate::progression::NamespaceDurablePayload>() as u64;
    let publication = std::mem::size_of::<crate::orchestration::PublicationPayload>() as u64;
    let reopened = std::mem::size_of::<crate::progression::ReopenedPayload>() as u64;
    let roster = std::mem::size_of::<
        Option<worth_store_recovery_physics::VerifiedEffectiveReleaseHeadRosterV14>,
    >() as u64;
    let expectation = std::mem::size_of::<crate::progression::ExpectationPayload>() as u64;
    let publication_plan =
        std::mem::size_of::<crate::progression::RecoveryPublicationPlan>() as u64;
    Some(
        planned
            .checked_add(extraction_bytes)?
            .max(planned.checked_add(input.checked_mul(2)?)?)
            .max(input.checked_add(staged.checked_mul(2)?)?)
            .max(staged.checked_add(namespace.checked_mul(2)?)?)
            .max(staged.checked_add(publication.checked_mul(2)?)?)
            .max(publication.checked_add(namespace.checked_mul(2)?)?)
            .max(
                namespace
                    .checked_add(reopened.checked_mul(2)?)?
                    .checked_add(roster.checked_mul(2)?)?,
            )
            .max(
                namespace
                    .checked_mul(3)?
                    .checked_add(roster.checked_mul(2)?)?,
            )
            .max(publication_plan.checked_add(expectation.checked_mul(2)?)?),
    )
}
