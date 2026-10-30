use worth_foundational::foundational_merge;

use {
    crate::certification::ordinary_context::transitions::fixtures::branch::branch_id,
    crate::certification::ordinary_context::transitions::fixtures::branch::staged_candidate,
};
use {
    crate::certification::ordinary_context::transitions::fixtures::merge::authority_first_merge_candidate,
    crate::certification::ordinary_context::transitions::fixtures::merge::conflict_locus,
    crate::certification::ordinary_context::transitions::fixtures::merge::merge_basis,
    crate::certification::ordinary_context::transitions::fixtures::merge::merge_summary,
    crate::certification::ordinary_context::transitions::fixtures::merge::projection_shaped_merge_candidate,
    crate::certification::ordinary_context::transitions::fixtures::merge::stale_target_advanced,
    crate::certification::ordinary_context::transitions::fixtures::merge::strategy_identity,
};

#[test]
fn independent_merge_producers_preserve_candidate_meaning() {
    let authority_first = authority_first_merge_candidate("mesh-update");
    let projection_shaped = projection_shaped_merge_candidate("mesh-update");

    assert_eq!(
        authority_first.source_branch(),
        projection_shaped.source_branch()
    );
    assert_eq!(
        authority_first.target_branch(),
        projection_shaped.target_branch()
    );
    assert_eq!(authority_first.intent(), projection_shaped.intent());
    assert_eq!(
        authority_first.structural_summary(),
        projection_shaped.structural_summary()
    );
    assert_eq!(
        authority_first.merge_basis(),
        projection_shaped.merge_basis()
    );
    assert_eq!(
        authority_first.merge_base_selection_basis(),
        projection_shaped.merge_base_selection_basis()
    );
    assert_eq!(
        authority_first.strategy_identity(),
        projection_shaped.strategy_identity()
    );
    assert_eq!(
        authority_first.strategy_descriptor_digest(),
        projection_shaped.strategy_descriptor_digest()
    );
    assert_eq!(
        authority_first.strategy_contract_basis(),
        projection_shaped.strategy_contract_basis()
    );
    assert_eq!(
        authority_first.strategy_basis(),
        projection_shaped.strategy_basis()
    );
    assert_eq!(
        authority_first.correspondence_basis(),
        projection_shaped.correspondence_basis()
    );
    assert_eq!(
        authority_first.remap_basis(),
        projection_shaped.remap_basis()
    );
    assert_eq!(authority_first.payload(), projection_shaped.payload());
}

#[test]
fn merge_admission_preserves_success_denial_and_stale_topology() {
    let accepted = authority_first_merge_candidate("mesh-update").admit_as_accepted();
    let conflict =
        authority_first_merge_candidate("mesh-update").admit_as_conflict(vec![conflict_locus()]);
    let denied = authority_first_merge_candidate("mesh-update").deny("policy blocked auto-merge");
    let stale = authority_first_merge_candidate("mesh-update").stale(stale_target_advanced());
    let deferred = authority_first_merge_candidate("mesh-update")
        .defer("waiting for supporting replay evidence");
    let rebind = authority_first_merge_candidate("mesh-update")
        .require_rebind("merge base moved to a new trust boundary");
    let failed = authority_first_merge_candidate("mesh-update")
        .fail("merge planner crashed before verdict materialization");

    assert!(matches!(
        accepted,
        worth_proof::TransitionOutcome::Success(_)
    ));
    assert!(matches!(
        conflict,
        worth_proof::TransitionOutcome::Success(_)
    ));
    assert!(matches!(
        denied,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalMergeAdmissionDenial::PolicyDenied { .. }
        )
    ));
    assert!(matches!(stale, worth_proof::TransitionOutcome::Stale(_)));
    assert!(matches!(
        deferred,
        worth_proof::TransitionOutcome::Deferred(_)
    ));
    assert!(matches!(
        rebind,
        worth_proof::TransitionOutcome::RebindRequired(_)
    ));
    assert!(matches!(failed, worth_proof::TransitionOutcome::Failed(_)));
}

#[test]
fn conflict_and_stale_surfaces_remain_blind_consumer_readable() {
    let conflict = match authority_first_merge_candidate("mesh-update")
        .admit_as_conflict(vec![conflict_locus()])
    {
        worth_proof::TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected success, got {other:?}"),
    };
    let stale = stale_target_advanced();

    assert_eq!(
        conflict.kind(),
        worth_foundational::FoundationalMergeVerdictKind::Conflict
    );
    assert_eq!(conflict.conflict_loci().len(), 1);
    assert_eq!(conflict.conflict_loci()[0].category(), "geometry-face");
    assert_eq!(conflict.conflict_loci()[0].source_detail(), "source:face-7");
    assert_eq!(conflict.conflict_loci()[0].target_detail(), "target:face-7");
    assert_eq!(
        stale.kind(),
        worth_foundational::FoundationalBranchBasisDriftKind::TargetAdvanced
    );
    assert_eq!(
        stale.verdict_kind(),
        worth_foundational::FoundationalMergeVerdictKind::StaleBasis
    );
    assert!(stale.reason().contains("target branch advanced"));
}

#[test]
fn strategy_and_basis_surfaces_remain_visible_on_verdicts() {
    let verdict = match authority_first_merge_candidate("mesh-update").admit_as_advisory() {
        worth_proof::TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected success, got {other:?}"),
    };

    assert_eq!(
        verdict.kind(),
        worth_foundational::FoundationalMergeVerdictKind::Advisory
    );
    assert_eq!(verdict.structural_summary(), merge_summary());
    assert_eq!(
        verdict.strategy_identity().family().as_str(),
        "relational-merge"
    );
    assert_eq!(
        verdict.strategy_identity().semantic_name().as_str(),
        "geometry-aware-reconcile"
    );
    assert_eq!(verdict.strategy_identity().version().as_str(), "v1");
    assert_eq!(
        verdict.strategy_descriptor_digest().digest_id().bytes(),
        &[77; 32]
    );
    assert_eq!(verdict.strategy_contract_basis().basis_id().get(), 61);
    assert_eq!(verdict.strategy_basis().basis_id().get(), 59);
    assert_eq!(verdict.merge_basis().identity().basis_id().get(), 73);
    assert_eq!(verdict.merge_basis().family().as_str(), "geometry-kernel");
    assert_eq!(verdict.merge_basis().version().as_str(), "2026-05");
    assert_eq!(verdict.merge_base_selection_basis().basis_id().get(), 57);
    assert_eq!(
        verdict
            .correspondence_basis()
            .expect("correspondence basis should stay explicit")
            .basis_id()
            .get(),
        67
    );
    assert_eq!(
        verdict
            .remap_basis()
            .expect("remap basis should stay explicit")
            .basis_id()
            .get(),
        71
    );
}

#[test]
fn conflict_verdict_requires_explicit_conflict_loci() {
    let denial = match authority_first_merge_candidate("mesh-update").admit_as_conflict(Vec::new())
    {
        worth_proof::TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denial, got {other:?}"),
    };

    assert_eq!(
        denial,
        worth_foundational::FoundationalMergeAdmissionDenial::EmptyConflictLoci
    );
    assert_eq!(
        denial.verdict_kind(),
        worth_foundational::FoundationalMergeVerdictKind::Denied
    );
}

#[test]
fn merge_plan_denies_basis_and_comparison_target_mismatches() {
    let merge_basis_source_mismatch = foundational_merge(staged_candidate("mesh-update"))
        .into_target_branch(branch_id("main"))
        .with_intent(worth_foundational::FoundationalMergeIntent::ReconcileIntoTarget)
        .with_structural_summary(merge_summary())
        .with_merge_basis(merge_basis("feature/other", "main"))
        .with_merge_base_selection_basis(
            authority_first_merge_candidate("mesh-update").merge_base_selection_basis(),
        )
        .under_strategy(strategy_identity())
        .with_strategy_descriptor_digest(
            authority_first_merge_candidate("mesh-update").strategy_descriptor_digest(),
        )
        .with_strategy_contract_basis(
            authority_first_merge_candidate("mesh-update").strategy_contract_basis(),
        )
        .with_strategy_basis(authority_first_merge_candidate("mesh-update").strategy_basis())
        .plan()
        .expect_err("mismatched source branch must be denied");
    assert_eq!(
        merge_basis_source_mismatch,
        worth_foundational::FoundationalMergeConstructionDenial::MergeBasisSourceBranchMismatch
    );

    let merge_basis_target_mismatch = foundational_merge(staged_candidate("mesh-update"))
        .into_target_branch(branch_id("main"))
        .with_intent(worth_foundational::FoundationalMergeIntent::ReconcileIntoTarget)
        .with_structural_summary(merge_summary())
        .with_merge_basis(merge_basis("feature/geometry", "release"))
        .with_merge_base_selection_basis(
            authority_first_merge_candidate("mesh-update").merge_base_selection_basis(),
        )
        .under_strategy(strategy_identity())
        .with_strategy_descriptor_digest(
            authority_first_merge_candidate("mesh-update").strategy_descriptor_digest(),
        )
        .with_strategy_contract_basis(
            authority_first_merge_candidate("mesh-update").strategy_contract_basis(),
        )
        .with_strategy_basis(authority_first_merge_candidate("mesh-update").strategy_basis())
        .plan()
        .expect_err("mismatched target branch must be denied");
    assert_eq!(
        merge_basis_target_mismatch,
        worth_foundational::FoundationalMergeConstructionDenial::MergeBasisTargetBranchMismatch
    );

    let comparison_basis_target_mismatch = foundational_merge(staged_candidate("mesh-update"))
        .into_target_branch(branch_id("release"))
        .with_intent(worth_foundational::FoundationalMergeIntent::ReconcileIntoTarget)
        .with_structural_summary(merge_summary())
        .with_merge_basis(merge_basis("feature/geometry", "release"))
        .with_merge_base_selection_basis(
            authority_first_merge_candidate("mesh-update").merge_base_selection_basis(),
        )
        .under_strategy(strategy_identity())
        .with_strategy_descriptor_digest(
            authority_first_merge_candidate("mesh-update").strategy_descriptor_digest(),
        )
        .with_strategy_contract_basis(
            authority_first_merge_candidate("mesh-update").strategy_contract_basis(),
        )
        .with_strategy_basis(authority_first_merge_candidate("mesh-update").strategy_basis())
        .plan()
        .expect_err("comparison basis target drift must be denied");
    assert_eq!(
        comparison_basis_target_mismatch,
        worth_foundational::FoundationalMergeConstructionDenial::ComparisonBasisTargetBranchMismatch
    );
}
