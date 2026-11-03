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
mod tests {
    use super::super::{PlanningMemoryDenial, PlanningResidentAllowance};
    use super::*;

    #[test]
    fn planned_backing_refuses_combined_construction_before_allocating() {
        for (name, bytes) in [
            (
                "planning_custody",
                std::mem::size_of::<crate::progression::PlanningCustody>(),
            ),
            (
                "final_custody",
                std::mem::size_of::<crate::progression::CustodyState>(),
            ),
            (
                "selected_tier_custody",
                std::mem::size_of::<
                    Option<worth_store_recovery_physics::VerifiedSelectedTierEpochCustody>,
                >(),
            ),
            (
                "namespace_state",
                std::mem::size_of::<crate::progression::NamespaceDurableState>(),
            ),
            (
                "final_namespace_state",
                std::mem::size_of::<
                    crate::progression::NamespaceDurableState<crate::progression::CustodyState>,
                >(),
            ),
            (
                "base_image_plan",
                std::mem::size_of::<crate::progression::RecoveryBaseImagePlan>(),
            ),
            (
                "publication_plan",
                std::mem::size_of::<crate::progression::RecoveryPublicationPlan>(),
            ),
            (
                "expectation_payload",
                std::mem::size_of::<crate::progression::ExpectationPayload>(),
            ),
            (
                "expectation_owner",
                std::mem::size_of::<crate::progression::RecoveryPublicationExpectation>(),
            ),
            (
                "platform_authority",
                std::mem::size_of::<crate::entry::AdmittedPlatformAuthority>(),
            ),
            (
                "fresh_reopen",
                std::mem::size_of::<
                    worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen,
                >(),
            ),
            (
                "runtime_core",
                std::mem::size_of::<worth_store::physical_runtime::RecoveredPhysicalRuntimeCore>(),
            ),
            (
                "serving_handoff",
                std::mem::size_of::<worth_store::physical_runtime::RecoveredPhysicalServingHandoff>(
                ),
            ),
        ] {
            eprintln!("compiler_layout {name}: {bytes}");
        }
        let payload = std::mem::size_of::<PlannedRecoveryState>() as u64;
        for extraction in [0, payload] {
            let staged = std::mem::size_of::<crate::progression::StagedRecoveryState>() as u64;
            let input =
                std::mem::size_of::<crate::orchestration::RecoveryStagingInputState>() as u64;
            let namespace =
                std::mem::size_of::<crate::progression::NamespaceDurablePayload>() as u64;
            let publication =
                std::mem::size_of::<crate::orchestration::PublicationPayload>() as u64;
            let reopened = std::mem::size_of::<crate::progression::ReopenedPayload>() as u64;
            let roster = std::mem::size_of::<
                Option<worth_store_recovery_physics::VerifiedEffectiveReleaseHeadRosterV14>,
            >() as u64;
            let expectation = std::mem::size_of::<crate::progression::ExpectationPayload>() as u64;
            let publication_plan =
                std::mem::size_of::<crate::progression::RecoveryPublicationPlan>() as u64;
            let movement = (payload + extraction)
                .max(payload + 2 * input)
                .max(input + 2 * staged)
                .max(staged + 2 * namespace)
                .max(staged + 2 * publication)
                .max(publication + 2 * namespace)
                .max(namespace + 2 * reopened + 2 * roster)
                .max(3 * namespace + 2 * roster)
                .max(publication_plan + 2 * expectation);
            let required =
                17 + payload + staged + input + namespace + publication + reopened + movement;
            let mut allowance = PlanningResidentAllowance::new(17, required - 1).unwrap();
            assert!(matches!(
                PlannedStateBacking::prepare(&mut allowance, extraction),
                Err(PlanningMemoryDenial::RecoveryMemoryBytes { observed }) if observed == required
            ));
            assert_eq!(allowance.used(), 17);
            assert_eq!(allowance.peak(), 17);
        }
    }

    #[test]
    fn planned_backing_peak_keeps_actual_capacity_and_construction_overlap() {
        let payload = std::mem::size_of::<PlannedRecoveryState>() as u64;
        for extraction in [0, payload] {
            let mut allowance = PlanningResidentAllowance::new(17, u64::MAX).unwrap();
            let backing = PlannedStateBacking::prepare(&mut allowance, extraction).unwrap();
            let planned_backing = PlanningResidentAllowance::vector_bytes(&backing.values).unwrap();
            let staged_backing = backing.staged.backing_bytes().unwrap();
            let staged = std::mem::size_of::<crate::progression::StagedRecoveryState>() as u64;
            let input =
                std::mem::size_of::<crate::orchestration::RecoveryStagingInputState>() as u64;
            let namespace =
                std::mem::size_of::<crate::progression::NamespaceDurablePayload>() as u64;
            let publication =
                std::mem::size_of::<crate::orchestration::PublicationPayload>() as u64;
            let reopened = std::mem::size_of::<crate::progression::ReopenedPayload>() as u64;
            let roster = std::mem::size_of::<
                Option<worth_store_recovery_physics::VerifiedEffectiveReleaseHeadRosterV14>,
            >() as u64;
            let expectation = std::mem::size_of::<crate::progression::ExpectationPayload>() as u64;
            let publication_plan =
                std::mem::size_of::<crate::progression::RecoveryPublicationPlan>() as u64;
            let movement = (payload + extraction)
                .max(payload + 2 * input)
                .max(input + 2 * staged)
                .max(staged + 2 * namespace)
                .max(staged + 2 * publication)
                .max(publication + 2 * namespace)
                .max(namespace + 2 * reopened + 2 * roster)
                .max(3 * namespace + 2 * roster)
                .max(publication_plan + 2 * expectation);
            let input_backing = backing.input.backing_bytes().unwrap();
            let namespace_backing = backing.namespace.backing_bytes().unwrap();
            let publication_backing = backing.publication.backing_bytes().unwrap();
            let actual = planned_backing
                + staged_backing
                + input_backing
                + namespace_backing
                + publication_backing
                + backing.reopened.backing_bytes().unwrap();
            assert!(planned_backing >= payload);
            assert!(staged_backing >= staged);
            assert!(input_backing >= input);
            assert!(namespace_backing >= namespace);
            assert!(publication_backing >= publication);
            assert!(backing.reopened.backing_bytes().unwrap() >= reopened);
            assert_eq!(allowance.used(), 17 + actual);
            assert_eq!(allowance.peak(), 17 + actual + movement);
            assert!(backing.values.is_empty());
            drop(backing);
            allowance.release(actual);
            assert_eq!(allowance.used(), 17);
            assert_eq!(allowance.peak(), 17 + actual + movement);
        }
    }
}

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
