use worth_foundational::{
    FoundationalAuthorityTransitionClass, FoundationalAuthorityTransitionDenial,
    FoundationalAuthorityTransitionOutcomeKind, FoundationalCommittedAuthorityConstructionDenial,
    FoundationalNoOpCause,
};

use super::fixtures::committed::{
    accepted_verdict, advisory_verdict, committed_authority, conflict_verdict,
    metadata_only_commit_input, no_op_input, ordinary_commit_input, ordinary_delta_summary,
    parent_basis, promotion_commit_input, replay_revalidated_commit_input, superseded_verdict,
    unary_parentage,
};

#[test]
fn proof_bearing_committed_authority_preserves_merge_strategy_and_basis_meaning() {
    let artifact = accepted_verdict("mesh-update")
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("accepted verdict should admit committed authority");

    assert_eq!(
        artifact.transition_outcome_kind(),
        FoundationalAuthorityTransitionOutcomeKind::Committed
    );
    assert_eq!(
        artifact.transition_class(),
        FoundationalAuthorityTransitionClass::Commit
    );
    assert_eq!(artifact.parent_basis().basis_id().get(), 401);
    assert_eq!(
        artifact.strategy_identity().family().as_str(),
        "relational-merge"
    );
    assert_eq!(
        artifact.strategy_identity().semantic_name().as_str(),
        "geometry-aware-reconcile"
    );
    assert_eq!(
        artifact.strategy_descriptor_digest().digest_id().bytes(),
        &[77; 32]
    );
    assert_eq!(artifact.strategy_contract_basis().basis_id().get(), 61);
    assert_eq!(artifact.strategy_basis().basis_id().get(), 59);
    assert_eq!(std::mem::size_of_val(artifact.proofs()), 0);
    assert_eq!(
        artifact.admission_basis().outcome_kind(),
        FoundationalAuthorityTransitionOutcomeKind::Committed
    );
    assert_eq!(artifact.payload(), &"mesh-update");
}

#[test]
fn authority_transition_classes_and_no_op_causes_remain_distinct() {
    let metadata_only = accepted_verdict("mesh-update")
        .commit_with(metadata_only_commit_input(), committed_authority())
        .expect("metadata-only verdict");
    let promotion = advisory_verdict("mesh-update")
        .commit_with(promotion_commit_input(), committed_authority())
        .expect("promotion verdict");
    let replay = accepted_verdict("mesh-update")
        .commit_with(replay_revalidated_commit_input(), committed_authority())
        .expect("replay revalidated verdict");
    let no_op = accepted_verdict("mesh-update")
        .commit_with(
            no_op_input(FoundationalNoOpCause::BasisEquivalent),
            committed_authority(),
        )
        .expect("no-op verdict");

    assert_eq!(
        metadata_only.transition_class(),
        FoundationalAuthorityTransitionClass::MetadataOnlyCommit
    );
    assert_eq!(
        promotion.transition_class(),
        FoundationalAuthorityTransitionClass::PromotionCommit
    );
    assert_eq!(
        replay.transition_class(),
        FoundationalAuthorityTransitionClass::ReplayRevalidatedCommit
    );
    assert_eq!(
        no_op.transition_class(),
        FoundationalAuthorityTransitionClass::NoOp
    );
    assert_eq!(
        no_op.no_op_cause(),
        Some(FoundationalNoOpCause::BasisEquivalent)
    );
    assert_eq!(
        no_op.transition_outcome_kind(),
        FoundationalAuthorityTransitionOutcomeKind::NoOp
    );
    assert_eq!(no_op.committed_delta_summary().delta_count(), 0);
}

#[test]
fn unary_and_multi_parent_transitions_preserve_canonical_parentage_and_merge_ancestry() {
    let unary = accepted_verdict("mesh-update")
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("unary parent commit");
    let multiparent = advisory_verdict("mesh-update")
        .commit_with(promotion_commit_input(), committed_authority())
        .expect("multi-parent promotion");

    assert_eq!(unary.parentage().parents(), &[parent_basis(401)]);
    assert_eq!(
        multiparent.parentage().parents(),
        &[parent_basis(401), parent_basis(403), parent_basis(406)]
    );
    assert_eq!(
        multiparent
            .merge_ancestry_basis()
            .expect("merge ancestry basis should stay explicit")
            .basis_id()
            .get(),
        499
    );
}

#[test]
fn conflict_and_superseded_verdicts_are_not_commit_eligible() {
    let conflict_denial = match conflict_verdict("mesh-update")
        .commit_with(ordinary_commit_input(), committed_authority())
    {
        Ok(_) => panic!("conflict verdict must not admit committed authority"),
        Err(denial) => denial,
    };
    let superseded_denial = match superseded_verdict("mesh-update")
        .commit_with(ordinary_commit_input(), committed_authority())
    {
        Ok(_) => panic!("superseded verdict must not admit committed authority"),
        Err(denial) => denial,
    };

    assert_eq!(
        conflict_denial,
        FoundationalAuthorityTransitionDenial::MergeVerdictNotCommitEligible {
            verdict_kind: worth_foundational::FoundationalMergeVerdictKind::Conflict,
        }
    );
    assert_eq!(
        superseded_denial,
        FoundationalAuthorityTransitionDenial::MergeVerdictNotCommitEligible {
            verdict_kind: worth_foundational::FoundationalMergeVerdictKind::Superseded,
        }
    );
}

#[test]
fn committed_authority_input_fail_closes_no_op_and_parentage_misuse() {
    let missing_parent_basis = worth_foundational::FoundationalCommittedAuthorityInput::new(
        FoundationalAuthorityTransitionClass::Commit,
        None,
        parent_basis(999),
        unary_parentage(),
        None,
        ordinary_delta_summary(),
    )
    .expect_err("primary parent basis must be present");
    let no_op_missing_cause = worth_foundational::FoundationalCommittedAuthorityInput::new(
        FoundationalAuthorityTransitionClass::NoOp,
        None,
        parent_basis(401),
        unary_parentage(),
        None,
        worth_foundational::FoundationalCommitDeltaSummary::new(Vec::new()),
    )
    .expect_err("no-op must carry explicit cause");

    assert_eq!(
        missing_parent_basis,
        FoundationalCommittedAuthorityConstructionDenial::PrimaryParentBasisNotInParentage
    );
    assert_eq!(
        no_op_missing_cause,
        FoundationalCommittedAuthorityConstructionDenial::NoOpTransitionRequiresCause
    );
}
