use worth_store_test_support::harness::physical_isolation::epoch_scope as support;

use worth_store_tiering::ColdTierIoPosture;
use {
    worth_store_physical_format::PhysicalReclaimRegion,
    worth_store_physical_format::ReclaimedByteInterpretation,
};
use {
    worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot,
    worth_store_physical_isolation::HazardLeaseTable,
    worth_store_physical_isolation::HazardLeaseTableCapacity,
    worth_store_physical_isolation::ReclaimDenial,
    worth_store_physical_isolation::ReclaimEligibilityProof,
};
use {
    worth_store_reclaim_policy::ReclaimLaterHandoffPolicy,
    worth_store_reclaim_policy::ReclaimPermit, worth_store_reclaim_policy::ReclaimPolicyAdmission,
    worth_store_reclaim_policy::ReclaimPolicyDenialKind,
    worth_store_reclaim_policy::ReclaimPolicyExecutionObservation,
    worth_store_reclaim_policy::ReclaimPolicyProofAuthority,
    worth_store_reclaim_policy::ReclaimPolicyReachabilityDenial,
    worth_store_reclaim_policy::ReclaimPolicyRequest,
    worth_store_reclaim_policy::ReclaimPolicySecurityScope,
    worth_store_reclaim_policy::ReclaimPolicyViolationKind,
};

use {
    crate::io_scheduling::reclaim_policy::reclaim_support::admitted_backend,
    crate::io_scheduling::reclaim_policy::reclaim_support::admitted_policy_for_region,
    crate::io_scheduling::reclaim_policy::reclaim_support::backend_without_reclaim_posture,
    crate::io_scheduling::reclaim_policy::reclaim_support::base_real_chain_request,
    crate::io_scheduling::reclaim_policy::reclaim_support::execute_policy_with_observation,
    crate::io_scheduling::reclaim_policy::reclaim_support::internal_security_scope,
    crate::io_scheduling::reclaim_policy::reclaim_support::reachability_from_physical_isolation_removal,
    crate::io_scheduling::reclaim_policy::reclaim_support::real_reachability_for_region,
    crate::io_scheduling::reclaim_policy::reclaim_support::region_for_generation,
    crate::io_scheduling::reclaim_policy::reclaim_support::S6ReclaimFixture,
};

#[test]
fn reclaim_policy_consumes_real_physical_isolation_reachability_removal() {
    let generation = 333;
    let region = region_for_generation(generation);
    let reachability = real_reachability_for_region(generation, region);
    let backend = admitted_backend();
    let authority =
        worth_store_reclaim_policy::ReclaimPolicyProofAuthority::for_admitted_backend(&backend);
    let admitted = worth_store_reclaim_policy::ReclaimPolicyAdmission::admit(
        authority,
        worth_store_reclaim_policy::ReclaimPolicyRequest::new()
            .for_region(region)
            .with_posture(
                authority
                    .cold_tier_io_posture(
                        worth_store_physical_format::ReclaimedByteInterpretation::NonObservableReclaimedStorage,
                    )
                    .unwrap(),
            )
            .with_reachability(reachability)
            .with_security_scope(internal_security_scope())
            .with_reclaim_permit(worth_store_reclaim_policy::ReclaimPermit::new(1).unwrap())
            .with_later_handoff_policy(authority.non_claim_later_handoff()),
    )
    .unwrap();
    let receipt = execute_policy_with_observation(
        admitted,
        worth_store_reclaim_policy::ReclaimPolicyExecutionObservation::new(
            region,
            worth_store_physical_format::ReclaimedByteInterpretation::NonObservableReclaimedStorage,
            internal_security_scope(),
            true,
        ),
    )
    .unwrap()
    .clone();

    assert_eq!(
        receipt.observed_interpretation(),
        worth_store_physical_format::ReclaimedByteInterpretation::NonObservableReclaimedStorage
    );
    assert_eq!(
        worth_store_tiering::ColdTierIoPosture::from_reclaim_receipt(receipt)
            .unwrap()
            .reclaim_receipt()
            .policy()
            .posture()
            .operation(),
        worth_store_reclaim_policy::ReclaimPolicyOperation::ColdTierMovementPosture,
    );
}

#[test]
fn reclaim_policy_executes_distinct_byte_interpretations_through_real_physical_isolation_removal() {
    let cases = [
        worth_store_physical_format::ReclaimedByteInterpretation::PhysicalZeros,
        worth_store_physical_format::ReclaimedByteInterpretation::LogicalHole,
        worth_store_physical_format::ReclaimedByteInterpretation::UnavailableBytes,
        worth_store_physical_format::ReclaimedByteInterpretation::NonObservableReclaimedStorage,
    ];

    for (index, interpretation) in cases.into_iter().enumerate() {
        let generation = 430 + index as u64;
        let region = region_for_generation(generation);
        let policy = admitted_policy_for_region(region, interpretation);
        let receipt = execute_policy_with_observation(
            policy,
            worth_store_reclaim_policy::ReclaimPolicyExecutionObservation::new(
                region,
                interpretation,
                internal_security_scope(),
                true,
            ),
        )
        .unwrap();

        assert_eq!(receipt.observed_interpretation(), interpretation);
        assert_eq!(receipt.policy().posture().interpretation(), interpretation);
        assert_eq!(receipt.counters().executed(), 1);
    }
}

#[test]
fn reclaim_policy_denies_region_not_covered_by_physical_isolation_reachability_removal() {
    let world = crate::io_scheduling::reclaim_policy::reclaim_support::S6ReclaimFixture::new(337);
    let proof = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
        world.executed_reachability(),
        worth_store_physical_isolation::HazardLeaseTable::with_capacity(
            worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(1).unwrap(),
        )
        .live_index_snapshot(),
        worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();
    let removal = proof.admit_reachability_removal().unwrap();
    let wrong_region = region_for_generation(338);

    assert_eq!(
        reachability_from_physical_isolation_removal(
            removal
                .lower_for_io_qos_reclaim_policy(region_for_generation(337))
                .unwrap(),
            wrong_region,
        )
        .unwrap_err(),
        worth_store_reclaim_policy::ReclaimPolicyReachabilityDenial::RegionNotCoveredByReachabilityRemoval
    );
}

#[test]
fn reclaim_policy_denies_same_owner_region_reuse() {
    let region = region_for_generation(339);
    let different_region =
        worth_store_physical_format::PhysicalReclaimRegion::new(region.reference(), 8192).unwrap();
    let world = crate::io_scheduling::reclaim_policy::reclaim_support::S6ReclaimFixture::new(339);
    let proof = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
        world.executed_reachability(),
        worth_store_physical_isolation::HazardLeaseTable::with_capacity(
            worth_store_physical_isolation::HazardLeaseTableCapacity::bounded_slots(1).unwrap(),
        )
        .live_index_snapshot(),
        worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();
    let removal = proof.admit_reachability_removal().unwrap();

    assert_eq!(
        reachability_from_physical_isolation_removal(
            removal.lower_for_io_qos_reclaim_policy(region).unwrap(),
            different_region,
        )
        .unwrap_err(),
        worth_store_reclaim_policy::ReclaimPolicyReachabilityDenial::RegionNotCoveredByReachabilityRemoval
    );
}

#[test]
fn reclaim_policy_denies_live_hazard_before_io_qos_admission_exists() {
    let world = crate::io_scheduling::reclaim_policy::reclaim_support::S6ReclaimFixture::new(441);
    let proof = worth_store_physical_isolation::ReclaimEligibilityProof::admit(
        world.executed_reachability(),
        world.live_hazard_snapshot(),
        worth_store_physical_isolation::BackupReachabilityLeaseIndexSnapshot::empty(),
    )
    .unwrap();

    assert!(matches!(
        proof.admit_reachability_removal().unwrap_err(),
        worth_store_physical_isolation::ReclaimDenial::BlockedByLiveHazardLease { .. }
    ));
}

#[test]
fn reclaim_policy_denies_real_chain_unsupported_backend_and_missing_security_scope() {
    let region = region_for_generation(442);
    let reachability = real_reachability_for_region(442, region);
    let supported = admitted_backend();
    let unsupported = backend_without_reclaim_posture();
    let supported_authority =
        worth_store_reclaim_policy::ReclaimPolicyProofAuthority::for_admitted_backend(&supported);
    let unsupported_authority =
        worth_store_reclaim_policy::ReclaimPolicyProofAuthority::for_admitted_backend(&unsupported);
    let unsupported_request = worth_store_reclaim_policy::ReclaimPolicyRequest::new()
        .for_region(region)
        .with_posture(
            supported_authority
                .punch_hole_posture(
                    worth_store_physical_format::ReclaimedByteInterpretation::LogicalHole,
                )
                .unwrap(),
        )
        .with_reachability(reachability.clone())
        .with_security_scope(internal_security_scope())
        .with_reclaim_permit(worth_store_reclaim_policy::ReclaimPermit::new(1).unwrap())
        .with_later_handoff_policy(unsupported_authority.non_claim_later_handoff());

    assert_eq!(
        worth_store_reclaim_policy::ReclaimPolicyAdmission::admit(
            unsupported_authority,
            unsupported_request
        )
        .unwrap_err()
        .kind(),
        &worth_store_reclaim_policy::ReclaimPolicyDenialKind::UnsupportedBackendPosture
    );

    let missing_security = base_real_chain_request(region, reachability)
        .with_posture(
            supported_authority
                .punch_hole_posture(
                    worth_store_physical_format::ReclaimedByteInterpretation::LogicalHole,
                )
                .unwrap(),
        )
        .with_reclaim_permit(worth_store_reclaim_policy::ReclaimPermit::new(1).unwrap())
        .with_later_handoff_policy(supported_authority.non_claim_later_handoff());

    assert_eq!(
        worth_store_reclaim_policy::ReclaimPolicyAdmission::admit(
            supported_authority,
            missing_security
        )
        .unwrap_err()
        .kind(),
        &worth_store_reclaim_policy::ReclaimPolicyDenialKind::MissingSecurityScope
    );
}

#[test]
fn reclaim_policy_reports_real_chain_execution_scope_and_handoff_violations() {
    let region = region_for_generation(443);
    let wrong_region = region_for_generation(444);
    let wrong_scope = worth_store_reclaim_policy::ReclaimPolicySecurityScope::from_admitted_scope(
        &worth_store_security::admitted_wrong_io_qos_security_scope_for_test(),
    );

    let wrong_region_violation = execute_policy_with_observation(
        admitted_policy_for_region(
            region,
            worth_store_physical_format::ReclaimedByteInterpretation::LogicalHole,
        ),
        worth_store_reclaim_policy::ReclaimPolicyExecutionObservation::new(
            wrong_region,
            worth_store_physical_format::ReclaimedByteInterpretation::LogicalHole,
            internal_security_scope(),
            true,
        ),
    )
    .unwrap_err();
    assert_eq!(
        wrong_region_violation.kind(),
        worth_store_reclaim_policy::ReclaimPolicyViolationKind::ProtectedReachabilityLost
    );

    let wrong_scope_violation = execute_policy_with_observation(
        admitted_policy_for_region(
            region,
            worth_store_physical_format::ReclaimedByteInterpretation::LogicalHole,
        ),
        worth_store_reclaim_policy::ReclaimPolicyExecutionObservation::new(
            region,
            worth_store_physical_format::ReclaimedByteInterpretation::LogicalHole,
            wrong_scope,
            true,
        ),
    )
    .unwrap_err();
    assert_eq!(
        wrong_scope_violation.kind(),
        worth_store_reclaim_policy::ReclaimPolicyViolationKind::SecurityScopeLost
    );

    let strengthened_handoff = execute_policy_with_observation(
        admitted_policy_for_region(
            region,
            worth_store_physical_format::ReclaimedByteInterpretation::LogicalHole,
        ),
        worth_store_reclaim_policy::ReclaimPolicyExecutionObservation::new(
            region,
            worth_store_physical_format::ReclaimedByteInterpretation::LogicalHole,
            internal_security_scope(),
            false,
        ),
    )
    .unwrap_err();
    assert_eq!(
        strengthened_handoff.kind(),
        worth_store_reclaim_policy::ReclaimPolicyViolationKind::LaterHandoffStrengthened
    );
}

#[test]
fn reclaim_policy_denies_real_chain_later_lifecycle_claims() {
    let region = region_for_generation(445);
    let backend = admitted_backend();
    let authority =
        worth_store_reclaim_policy::ReclaimPolicyProofAuthority::for_admitted_backend(&backend);
    let request = base_real_chain_request(region, real_reachability_for_region(445, region))
        .with_posture(
            authority
                .cold_tier_io_posture(worth_store_physical_format::ReclaimedByteInterpretation::NonObservableReclaimedStorage)
                .unwrap(),
        )
        .with_security_scope(internal_security_scope())
        .with_reclaim_permit(worth_store_reclaim_policy::ReclaimPermit::new(1).unwrap())
        .with_later_handoff_policy(worth_store_reclaim_policy::ReclaimLaterHandoffPolicy::claims_later_lifecycle_for_denial());

    assert_eq!(
        worth_store_reclaim_policy::ReclaimPolicyAdmission::admit(authority, request)
            .unwrap_err()
            .kind(),
        &worth_store_reclaim_policy::ReclaimPolicyDenialKind::LaterLifecycleClaimAttempted
    );
}
