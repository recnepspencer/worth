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
            std::mem::size_of::<worth_store::physical_runtime::CompletedPhysicalRecoveryFreshReopen>(
            ),
        ),
        (
            "runtime_core",
            std::mem::size_of::<worth_store::physical_runtime::RecoveredPhysicalRuntimeCore>(),
        ),
        (
            "serving_handoff",
            std::mem::size_of::<worth_store::physical_runtime::RecoveredPhysicalServingHandoff>(),
        ),
    ] {
        eprintln!("compiler_layout {name}: {bytes}");
    }
    let payload = std::mem::size_of::<PlannedRecoveryState>() as u64;
    for extraction in [0, payload] {
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
        let movement = (payload + extraction)
            .max(payload + 2 * input)
            .max(input + 2 * staged)
            .max(staged + 2 * namespace)
            .max(staged + 2 * publication)
            .max(publication + 2 * namespace)
            .max(namespace + 2 * reopened + 2 * roster)
            .max(3 * namespace + 2 * roster)
            .max(publication_plan + 2 * expectation);
        let handoff = crate::orchestration::RecoveryHandoffBacking::required_bytes().unwrap();
        let required =
            17 + payload + staged + input + namespace + publication + reopened + handoff + movement;
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
