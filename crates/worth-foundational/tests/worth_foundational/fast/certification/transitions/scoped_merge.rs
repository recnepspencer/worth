use worth_foundational::{
    foundational_merge, FoundationalMergeConstructionDenial, FoundationalMergeScope,
    FoundationalMergeScopeFamily, FoundationalScopeAdmissionBasis, FoundationalSelectedAspectLocus,
    FoundationalSelectedNodeLocus,
};
use worth_proof::TransitionOutcome;

use super::fixtures::branch::{branch_id, staged_candidate};
use super::fixtures::merge::{
    authority_first_merge_candidate, merge_basis, merge_summary, strategy_identity,
};
use super::fixtures::scoped_merge::{selected_aspect, selected_node};

#[test]
fn legacy_merge_candidates_lower_to_explicit_full_branch_scope() {
    let candidate = authority_first_merge_candidate("mesh-update");

    assert_eq!(
        candidate.scope().family(),
        FoundationalMergeScopeFamily::FullBranch
    );
    assert!(candidate.scope().selected_nodes_loci().is_empty());
    assert!(candidate.scope().selected_aspect_loci().is_empty());

    let verdict = match candidate.admit_as_accepted() {
        TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected accepted verdict, got {other:?}"),
    };
    assert_eq!(
        verdict.scope().family(),
        FoundationalMergeScopeFamily::FullBranch
    );
    assert_eq!(
        verdict.scope_evidence().requested_scope().family(),
        FoundationalMergeScopeFamily::FullBranch
    );
    assert_eq!(
        verdict.scope_evidence().breadth().requested_locus_count(),
        1
    );
    assert_eq!(verdict.scope_evidence().breadth().admitted_locus_count(), 1);
    assert_eq!(
        verdict.scope_evidence().admission_basis(),
        FoundationalScopeAdmissionBasis::DirectSourceIdentity
    );
    assert_eq!(
        verdict.scope_evidence().source_branch().as_str(),
        "feature/geometry"
    );
    assert_eq!(verdict.scope_evidence().target_branch().as_str(), "main");
}

#[test]
fn selected_node_scope_is_canonical_across_producer_ordering() {
    let scope = FoundationalMergeScope::selected_nodes([
        selected_node("gear:teeth"),
        selected_node("gear:thickness"),
    ])
    .expect("selected node scope");
    let reverse = FoundationalMergeScope::selected_nodes([
        selected_node("gear:thickness"),
        selected_node("gear:teeth"),
    ])
    .expect("reverse selected node scope");

    assert_eq!(scope, reverse);
    assert_eq!(scope.family(), FoundationalMergeScopeFamily::SelectedNodes);
    assert_eq!(scope.selected_nodes_loci()[0].as_str(), "gear:teeth");
    assert_eq!(scope.selected_nodes_loci()[1].as_str(), "gear:thickness");
}

#[test]
fn selected_aspect_scope_is_canonical_by_node_then_aspect() {
    let scope = FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
        selected_aspect("material", "finish"),
    ])
    .expect("selected aspect scope");
    let reverse = FoundationalMergeScope::selected_aspects([
        selected_aspect("material", "finish"),
        selected_aspect("gear", "thickness"),
        selected_aspect("gear", "teeth"),
    ])
    .expect("reverse selected aspect scope");

    assert_eq!(scope, reverse);
    assert_eq!(
        scope.family(),
        FoundationalMergeScopeFamily::SelectedAspects
    );
    assert_eq!(scope.selected_aspect_loci()[0].node().as_str(), "gear");
    assert_eq!(scope.selected_aspect_loci()[0].aspect().as_str(), "teeth");
    assert_eq!(scope.selected_aspect_loci()[2].node().as_str(), "material");
}

#[test]
fn scoped_merge_candidate_exposes_scope_to_blind_consumers() {
    let scope = FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
    ])
    .expect("scope");
    let candidate = foundational_merge(staged_candidate("mesh-update"))
        .into_target_branch(branch_id("main"))
        .with_intent(worth_foundational::FoundationalMergeIntent::ReconcileIntoTarget)
        .with_structural_summary(merge_summary())
        .with_scope(scope.clone())
        .with_merge_basis(merge_basis("feature/geometry", "main"))
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
        .expect("scoped merge candidate");

    assert_eq!(candidate.scope(), &scope);
    let verdict = match candidate.admit_as_advisory() {
        TransitionOutcome::Success(verdict) => verdict,
        other => panic!("expected advisory verdict, got {other:?}"),
    };
    assert_eq!(verdict.scope(), &scope);
    assert_eq!(verdict.scope_evidence().requested_scope(), &scope);
    assert_eq!(
        verdict.scope_evidence().admission_basis(),
        FoundationalScopeAdmissionBasis::DirectSourceIdentity
    );
    assert_eq!(verdict.scope_evidence().admitted_aspects().len(), 2);
    assert_eq!(
        verdict.scope_evidence().breadth().conflict_check_width(),
        merge_summary().conflict_check_width()
    );
}

#[test]
fn scoped_merge_request_denies_empty_and_duplicate_selection() {
    let empty_node_scope =
        FoundationalMergeScope::selected_nodes([]).expect_err("empty node scope must deny");
    let empty_aspect_scope =
        FoundationalMergeScope::selected_aspects([]).expect_err("empty aspect scope must deny");
    let duplicate_node =
        FoundationalMergeScope::selected_nodes([selected_node("gear"), selected_node("gear")])
            .expect_err("duplicate node scope must deny");
    let duplicate_aspect = FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "teeth"),
    ])
    .expect_err("duplicate aspect scope must deny");

    assert_eq!(
        FoundationalSelectedNodeLocus::new(" ").expect_err("empty node locus"),
        FoundationalMergeConstructionDenial::EmptySelectedNodeLocus
    );
    assert_eq!(
        FoundationalSelectedAspectLocus::new("").expect_err("empty aspect locus"),
        FoundationalMergeConstructionDenial::EmptySelectedAspectLocus
    );
    assert_eq!(
        empty_node_scope,
        FoundationalMergeConstructionDenial::EmptySelectedNodeScope
    );
    assert_eq!(
        empty_aspect_scope,
        FoundationalMergeConstructionDenial::EmptySelectedAspectScope
    );
    assert_eq!(
        duplicate_node,
        FoundationalMergeConstructionDenial::DuplicateSelectedNodeLocus
    );
    assert_eq!(
        duplicate_aspect,
        FoundationalMergeConstructionDenial::DuplicateSelectedAspectLocus
    );
}
