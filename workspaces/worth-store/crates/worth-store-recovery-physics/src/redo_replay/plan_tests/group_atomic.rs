use super::*;

#[test]
fn segment_update_page_count_must_equal_the_projected_segment_frames() {
    let member = PhysicalRedoMemberInput::new(
        range(),
        [1; 32],
        RecoveryOperationFate::Indeterminate,
        &encoded_redo_with_segment_page_count(2),
    );
    assert_eq!(
        plan_physical_redo(vec![member], vec![observation(1, 9, [0; 32])], 1),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
}

#[test]
fn durability_group_promotes_only_when_every_member_is_materialized() {
    let base = plan_physical_redo(
        vec![PhysicalRedoMemberInput::new(
            range(),
            [1; 32],
            RecoveryOperationFate::Indeterminate,
            &encoded_redo(),
        )],
        vec![observation(2, 10, result_digest())],
        1,
    )
    .unwrap();
    let group = [7; 32];
    let complete = plan_with_group_decisions(&base, group, false);
    let partial = plan_with_group_decisions(&base, group, true);

    let initial = two_indeterminate_fates();
    let fully_materialized = reconcile_materialized_operation_fates(initial.clone(), &complete);
    assert_eq!(fully_materialized.durable_unacknowledged(), 2);
    assert_eq!(fully_materialized.indeterminate(), 0);

    let partly_materialized = reconcile_materialized_operation_fates(initial, &partial);
    assert_eq!(partly_materialized.durable_unacknowledged(), 0);
    assert_eq!(partly_materialized.indeterminate(), 2);
}

#[test]
fn incomplete_group_carriage_cannot_obtain_observation_authority() {
    let group = PhysicalRedoGroupBinding::new([7; 32], [8; 32], 1, 2, [9; 32]).unwrap();
    let member = PhysicalRedoMemberInput::new_grouped(
        range(),
        [1; 32],
        group,
        RecoveryOperationFate::Indeterminate,
        &encoded_redo(),
    );
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let denied = admit_physical_redo_members(
        vec![member],
        test_store(),
        format,
        PhysicalRedoAdmissionLimits {
            recovery_memory_bytes: u64::MAX,
            targets: 1,
            distinct_targets: 1,
            projection: PhysicalRecoveryProjectionDecodeLimits {
                frames: 1,
                record_identities: 1,
                placements: 1,
                segment_updates: 1,
                manifests: 1,
                total_entries: 3,
                inline_allocations: 1,
            },
        },
    )
    .unwrap_err();
    assert_eq!(
        denied,
        PhysicalRedoPlanningDenial::InvalidRecoveryProjection
    );
}

fn plan_with_group_decisions(
    base: &ImmutablePhysicalRedoPlan,
    group_identity: [u8; 32],
    second_applies: bool,
) -> ImmutablePhysicalRedoPlan {
    let first_group =
        PhysicalRedoGroupBinding::new(group_identity, [1; 32], 1, 2, [3; 32]).unwrap();
    let second_group =
        PhysicalRedoGroupBinding::new(group_identity, [2; 32], 2, 2, [3; 32]).unwrap();
    let materialization = base.projections[0].materialization.clone();
    let first = PhysicalRedoProjection {
        operation: [1; 32],
        group: first_group,
        fate: RecoveryOperationFate::Indeterminate,
        materialization: materialization.clone(),
    };
    let second = PhysicalRedoProjection {
        operation: [2; 32],
        group: second_group,
        fate: RecoveryOperationFate::Indeterminate,
        materialization,
    };
    let prior = base.decisions[0].prior;
    let decision = |operation, kind| PhysicalRedoDecision {
        kind,
        prior,
        operation,
        record_index: 0,
        target_index: 0,
    };
    ImmutablePhysicalRedoPlan {
        scratch_bytes: base.supersession_scratch_bytes() * 2,
        records: base.records.clone(),
        decisions: vec![
            decision(
                [1; 32],
                PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn,
            ),
            decision(
                [2; 32],
                if second_applies {
                    PhysicalRedoDecisionKind::Apply
                } else {
                    PhysicalRedoDecisionKind::SkipPageAlreadyAtOrBeyondLsn
                },
            ),
        ]
        .into_boxed_slice(),
        projections: vec![first, second].into_boxed_slice(),
        recovery_root_allocation_bytes: 0,
        counters: base.counters,
    }
}

fn two_indeterminate_fates() -> crate::ReconciledOperationFates {
    reconcile_operation_fates(
        3,
        vec![evidence([1; 16], 1, [1; 32]), evidence([2; 16], 2, [2; 32])],
        2,
    )
    .unwrap()
}

fn evidence(
    allocation: [u8; 16],
    ordinal: u64,
    operation: [u8; 32],
) -> RecoveryOperationEvidenceInput {
    RecoveryOperationEvidenceInput::new(
        RecoveryOperationIdentity::new(allocation, ordinal, 1, ordinal, operation).unwrap(),
        operation,
        1,
        4,
        RecoveryBindingFreshness::Retained,
        RecoveryOperationFate::Indeterminate,
    )
}
