use {
    worth_store_branch_deltas::admit_continuation_layout_support,
    worth_store_branch_deltas::admit_stable_basis_layout_support,
    worth_store_branch_deltas::reject_broadened_continuation_receipt,
    worth_store_branch_deltas::reject_stable_basis_layout_descriptor,
};
use {
    worth_store_live_query::live_query_semantic_authority, worth_store_live_query::StableBasisId,
};

#[test]
fn stable_basis_and_continuation_families_bind_support_to_admitted_windows() {
    let stable_basis_id = StableBasisId(41);
    let stable_basis_plan = live_query_semantic_authority().declare_stable_basis_support(
        stable_basis_id,
        6,
        worth_store_live_query::ContinuationRetentionStatus::Retained,
    );
    let continuation_plan = live_query_semantic_authority().declare_continuation_window(
        stable_basis_id,
        4,
        worth_store_live_query::ContinuationRetentionStatus::RetentionRebindRequired,
    );
    let broadened_plan = live_query_semantic_authority().declare_continuation_window(
        stable_basis_id,
        10,
        worth_store_live_query::ContinuationRetentionStatus::RetentionRebindRequired,
    );
    let retained_plan = live_query_semantic_authority().declare_continuation_window(
        stable_basis_id,
        4,
        worth_store_live_query::ContinuationRetentionStatus::Retained,
    );

    let support =
        admit_stable_basis_layout_support(&stable_basis_plan).expect("stable basis report");
    assert_eq!(
        support.family_id(),
        worth_store_contracts::DurableArtifactFamilyId::PlacementStableBasis
    );
    assert_eq!(support.stable_basis_id(), stable_basis_id);
    assert_eq!(support.declared_support_rows(), 6);
    assert_eq!(
        support.retention_status(),
        worth_store_live_query::ContinuationRetentionStatus::Retained
    );
    assert_eq!(support.support_estimate().planned_point_lookups(), 1);
    assert_eq!(support.support_estimate().planned_maintenance_reads(), 1);

    let denial = reject_stable_basis_layout_descriptor(stable_basis_id).unwrap_err();
    assert_eq!(
        denial.kind(),
        worth_store_branch_deltas::BranchDeltaLayoutAccessDenialKind::StableBasisDescriptorCannotStandInForLayoutAuthority
    );

    let report =
        admit_continuation_layout_support(&continuation_plan).expect("continuation report");
    assert_eq!(
        report.family_id(),
        worth_store_contracts::DurableArtifactFamilyId::SupportCursor
    );
    assert_eq!(report.stable_basis_id(), stable_basis_id);
    assert_eq!(report.declared_window_rows(), 4);
    assert_eq!(
        report.retention_status(),
        worth_store_live_query::ContinuationRetentionStatus::RetentionRebindRequired
    );
    assert_eq!(report.support_estimate().planned_range_lookups(), 1);
    assert_eq!(report.support_estimate().planned_range_steps(), 4);

    let denial = reject_broadened_continuation_receipt(
        &live_query_semantic_authority().record_broadened_batch(&broadened_plan, 10),
    )
    .unwrap_err();
    assert_eq!(
        denial.kind(),
        worth_store_branch_deltas::BranchDeltaLayoutAccessDenialKind::BroadenedContinuationCannotStandInForBoundedSupport
    );

    let denial = report
        .resume_bounded_continuation(&live_query_semantic_authority().admit_narrow_batch(
            &live_query_semantic_authority().declare_continuation_window(
                stable_basis_id,
                4,
                worth_store_live_query::ContinuationRetentionStatus::RetentionRebindRequired,
            ),
            4,
        ))
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        worth_store_branch_deltas::BranchDeltaLayoutAccessDenialKind::ContinuationRebindRequired
    );

    let retained_report =
        admit_continuation_layout_support(&retained_plan).expect("retained continuation report");
    retained_report
        .resume_bounded_continuation(
            &live_query_semantic_authority().admit_narrow_batch(&retained_plan, 4),
        )
        .expect("admitted narrow window");

    let denial = retained_report
        .resume_bounded_continuation(
            &live_query_semantic_authority().admit_narrow_batch(&retained_plan, 5),
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        worth_store_branch_deltas::BranchDeltaLayoutAccessDenialKind::ContinuationWindowOutOfRange
    );
}
