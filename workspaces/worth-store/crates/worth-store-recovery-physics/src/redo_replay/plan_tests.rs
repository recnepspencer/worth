use super::*;
use crate::{
    reconcile_materialized_operation_fates, reconcile_operation_fates, RecoveryBindingFreshness,
    RecoveryOperationEvidenceInput, RecoveryOperationFate, RecoveryOperationIdentity,
    RecoveryPageSource,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::RecordArtifactFile;

#[path = "plan_tests/fixtures.rs"]
mod fixtures;
#[path = "plan_tests/group_atomic.rs"]
mod group_atomic;
#[path = "plan_tests/observation_membership.rs"]
mod observation_membership;
#[path = "plan_tests/projection_mutants.rs"]
mod projection_mutants;

use fixtures::*;

fn plan_physical_redo(
    members: Vec<PhysicalRedoMemberInput>,
    observations: Vec<RecoveryPageObservation>,
    maximum_targets: u64,
) -> Result<ImmutablePhysicalRedoPlan, PhysicalRedoPlanningDenial> {
    super::plan_physical_redo(members, observations, maximum_targets, test_store())
}

fn test_store() -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x51; 16]).unwrap(),
    )
    .published_identity()
}

#[test]
fn page_lsn_and_operation_fate_make_one_fixed_apply_or_skip_decision() {
    let range = range();
    let indeterminate = PhysicalRedoMemberInput::new(
        range,
        [1; 32],
        RecoveryOperationFate::Indeterminate,
        &encoded_redo(),
    );
    let prior = observation(1, 9, [0; 32]);
    let applied = plan_physical_redo(vec![indeterminate.clone()], vec![prior], 1).unwrap();
    assert_eq!(
        applied.decisions()[0].kind(),
        PhysicalRedoDecisionKind::Apply,
        "MUTANT_PREDICATE:c8-page-lsn-apply-skip-inverted"
    );

    let wrong_digest = observation(2, 10, [6; 32]);
    assert_eq!(
        plan_physical_redo(vec![indeterminate.clone()], vec![wrong_digest], 1),
        Err(PhysicalRedoPlanningDenial::PageDigestMismatch)
    );

    let current = observation(2, 10, result_digest());
    let skipped = plan_physical_redo(vec![indeterminate.clone()], vec![current], 1).unwrap();
    assert_eq!(
        skipped.decisions()[0].kind(),
        PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn,
        "MUTANT_PREDICATE:c8-page-lsn-apply-skip-inverted"
    );
    assert_eq!(
        skipped,
        plan_physical_redo(
            vec![indeterminate.clone()],
            vec![observation(2, 10, result_digest())],
            1,
        )
        .unwrap()
    );

    let fates = reconcile_operation_fates(
        3,
        vec![RecoveryOperationEvidenceInput::new(
            RecoveryOperationIdentity::new([1; 16], 1, 1, 1, [1; 32]).unwrap(),
            [2; 32],
            1,
            4,
            RecoveryBindingFreshness::Retained,
            RecoveryOperationFate::Indeterminate,
        )],
        1,
    )
    .unwrap();
    let promoted = reconcile_materialized_operation_fates(fates, &skipped);
    assert_eq!(promoted.durable_unacknowledged(), 1);
    assert_eq!(promoted.indeterminate(), 0);
    assert_eq!(
        plan_physical_redo(vec![indeterminate], Vec::new(), 0),
        Err(PhysicalRedoPlanningDenial::TargetLimit)
    );

    let durable = PhysicalRedoMemberInput::new(
        range,
        [1; 32],
        RecoveryOperationFate::DurableUnacknowledged,
        &encoded_redo(),
    );
    let materialized = plan_physical_redo(vec![durable], Vec::new(), 1).unwrap();
    assert_eq!(
        materialized.decisions()[0].kind(),
        PhysicalRedoDecisionKind::SkipOperationAlreadyMaterialized
    );
}

#[test]
fn absence_cannot_turn_a_wal_attempt_into_no_effect() {
    let member = PhysicalRedoMemberInput::new(
        range(),
        [1; 32],
        RecoveryOperationFate::ProvenNoEffect,
        &encoded_redo(),
    );
    assert_eq!(
        plan_physical_redo(vec![member], Vec::new(), 1),
        Err(PhysicalRedoPlanningDenial::ProvenNoEffectHasWalAttempt),
        "MUTANT_PREDICATE:c8-no-effect-proof-promoted-from-wal-attempt"
    );
}

#[test]
fn canonical_record_bytes_must_equal_the_projected_frame_payload() {
    let mut redo = encoded_redo();
    replace_first(&mut redo, b"redo-record", b"same-length");
    let member = PhysicalRedoMemberInput::new(
        range(),
        [1; 32],
        RecoveryOperationFate::Indeterminate,
        &redo,
    );
    assert_eq!(
        plan_physical_redo(vec![member], vec![observation(1, 9, [0; 32])], 1),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
}

#[test]
fn legacy_redo_domains_cannot_masquerade_as_the_v3_projection_grammar() {
    for legacy in [
        b"store.physical.wal.canonical-redo.v1".as_slice(),
        b"store.physical.wal.canonical-redo.v2".as_slice(),
    ] {
        let mut redo = encoded_redo();
        replace_first(&mut redo, b"store.physical.wal.canonical-redo.v3", legacy);
        let member = PhysicalRedoMemberInput::new(
            range(),
            [1; 32],
            RecoveryOperationFate::Indeterminate,
            &redo,
        );
        assert_eq!(
            plan_physical_redo(vec![member], vec![observation(1, 9, [0; 32])], 1),
            Err(PhysicalRedoPlanningDenial::WrongDomain)
        );
    }
}

#[test]
fn legacy_projection_domains_cannot_masquerade_as_the_v3_grammar() {
    let target = canonical_target_bytes_with_generations(1, 2);
    for legacy in [
        b"store.physical.recovery-projection.v1".as_slice(),
        b"store.physical.recovery-projection.v2".as_slice(),
    ] {
        let mut projection = projection_with_generations(1, 1, 2).encode();
        replace_first(
            &mut projection,
            b"store.physical.recovery-projection.v3",
            legacy,
        );
        let member = PhysicalRedoMemberInput::new(
            range(),
            [1; 32],
            RecoveryOperationFate::Indeterminate,
            &encoded_redo_with_projection_bytes_and_digest(
                &target,
                &projection,
                result_digest_for_page_generation(1),
            ),
        );
        assert_eq!(
            plan_physical_redo(vec![member], vec![], 1),
            Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
        );
    }
}

#[test]
fn inline_page_and_segment_artifact_generations_remain_independent() {
    let target = canonical_target_bytes_with_generations(1, 2);
    let projection = projection_with_generations(1, 1, 2);
    let redo = encoded_redo_with_projection_and_digest(
        &target,
        projection,
        result_digest_for_page_generation(1),
    );
    let member = PhysicalRedoMemberInput::new(
        range(),
        [1; 32],
        RecoveryOperationFate::Indeterminate,
        &redo,
    );
    let observed = physical_redo_observation_targets(std::slice::from_ref(&member), 1).unwrap();
    assert_eq!(
        observed[0].artifact(),
        RecordArtifactFile::Segment {
            segment: 1,
            generation: 2
        }
    );
    let absent = RecoveryPageObservation::absent(&observed[0], [8; 32]);
    let plan = plan_physical_redo(vec![member], vec![absent], 1).unwrap();
    assert_eq!(plan.decisions()[0].kind(), PhysicalRedoDecisionKind::Apply);
    let PhysicalRedoDecisionPrior::Page(prior) = plan.decisions()[0].prior() else {
        panic!("the apply decision retains the exact absent-prior proof")
    };
    let RecoveryPageSource::AbsentTarget { coordinate, .. } = prior.source() else {
        panic!("the new page is admitted from exact selected-root absence")
    };
    assert_eq!(
        coordinate.artifact(),
        RecordArtifactFile::Segment {
            segment: 1,
            generation: 2,
        }
    );
}

#[test]
fn selected_allocation_capacity_rejects_a_coordinated_foreign_root_projection() {
    let exact = PhysicalRedoMemberInput::new(
        range(),
        [1; 32],
        RecoveryOperationFate::Indeterminate,
        &encoded_redo(),
    );
    let exact_plan = plan_physical_redo(vec![exact], vec![observation(1, 9, [0; 32])], 1).unwrap();
    assert!(exact_plan
        .admit_inline_allocation_truth(&[], Some((1, 1)))
        .is_ok());

    // Keep the projection internally lawful: one projected page occupies one
    // slot in a coherently enlarged capacity-two allocation. Only the
    // independently admitted selected-root capacity remains capacity one.
    let matching_projection = projection_with_allocation(1, 2, 2, 2, 1);
    let matching = PhysicalRedoMemberInput::new(
        range(),
        [1; 32],
        RecoveryOperationFate::Indeterminate,
        &encoded_redo_with_projection(&canonical_target_bytes(), matching_projection),
    );
    let matching_plan =
        plan_physical_redo(vec![matching], vec![observation(1, 9, [0; 32])], 1).unwrap();
    assert!(matching_plan
        .admit_inline_allocation_truth(&[], Some((1, 2)))
        .is_ok());

    let foreign_projection = projection_with_allocation(1, 2, 2, 2, 1);
    let foreign = PhysicalRedoMemberInput::new(
        range(),
        [1; 32],
        RecoveryOperationFate::Indeterminate,
        &encoded_redo_with_projection(&canonical_target_bytes(), foreign_projection),
    );
    let foreign_plan =
        plan_physical_redo(vec![foreign], vec![observation(1, 9, [0; 32])], 1).unwrap();
    assert_eq!(
        foreign_plan.admit_inline_allocation_truth(&[], Some((1, 1))),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
}
