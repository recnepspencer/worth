use std::fmt::Debug;
use worth_foundational::{
    FoundationalAdmittedMergeScopeEvidence, FoundationalDeniedScopeLocus,
    FoundationalMergeAdmissionDenial, FoundationalMergeConstructionDenial, FoundationalMergeScope,
    FoundationalMergeScopeFamily, FoundationalScopeAdmissionBasis,
    FoundationalScopedMergeDenialEvidence, FoundationalScopedMergeDenialKind,
    FoundationalSelectedScopeNoOpCause, FoundationalSkippedOutOfScopeEvidence,
};
use worth_proof::TransitionOutcome;

use super::fixtures::branch::branch_id;
use super::fixtures::merge::merge_summary;
use super::fixtures::scoped_merge::{
    no_op_for_node, scoped_candidate, selected_aspect, selected_node,
};

#[test]
fn custom_scope_evidence_denies_mismatched_candidate_scope_or_branch_basis() {
    let requested_scope =
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope");
    let candidate_scope = FoundationalMergeScope::selected_nodes([selected_node("material")])
        .expect("candidate scope");
    let evidence_scope_mismatch = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        requested_scope,
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("gear")],
        [],
        [],
        FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect("scope evidence");
    let evidence_branch_mismatch = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/other"),
        branch_id("main"),
        candidate_scope.clone(),
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("material")],
        [],
        [],
        FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect("branch evidence");
    let evidence_target_mismatch = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("release"),
        candidate_scope.clone(),
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("material")],
        [],
        [],
        FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect("target branch evidence");

    assert_admission_denial(
        scoped_candidate(candidate_scope.clone())
            .admit_as_accepted_with_scope_evidence(evidence_scope_mismatch),
        FoundationalMergeAdmissionDenial::ScopedEvidenceScopeMismatch,
    );
    assert_admission_denial(
        scoped_candidate(candidate_scope)
            .admit_as_accepted_with_scope_evidence(evidence_branch_mismatch),
        FoundationalMergeAdmissionDenial::ScopedEvidenceSourceBranchMismatch,
    );
    assert_admission_denial(
        scoped_candidate(
            FoundationalMergeScope::selected_nodes([selected_node("material")])
                .expect("candidate scope"),
        )
        .admit_as_accepted_with_scope_evidence(evidence_target_mismatch),
        FoundationalMergeAdmissionDenial::ScopedEvidenceTargetBranchMismatch,
    );
}

#[test]
fn custom_scope_evidence_denies_conflict_width_mismatch() {
    let scope =
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope");
    let evidence = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        scope.clone(),
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("gear")],
        [],
        [],
        FoundationalSkippedOutOfScopeEvidence::new(0, None),
        merge_summary().conflict_check_width() + 1,
    )
    .expect("scope evidence");

    assert_admission_denial(
        scoped_candidate(scope).admit_as_accepted_with_scope_evidence(evidence),
        FoundationalMergeAdmissionDenial::ScopedEvidenceConflictWidthMismatch,
    );
}

#[test]
fn scope_evidence_constructor_denies_loci_outside_requested_scope() {
    let requested_scope =
        FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
            .expect("aspect scope");
    let denial = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        requested_scope,
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [],
        [selected_aspect("gear", "thickness")],
        [],
        FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect_err("outside requested scope must deny");

    assert_eq!(
        denial,
        FoundationalMergeConstructionDenial::ScopedEvidenceOutsideRequestedScope
    );
}

#[test]
fn scope_evidence_constructor_denies_duplicate_no_ops_and_full_branch_skips() {
    let selected_scope =
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope");
    let duplicate_no_op = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_scope,
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [],
        [],
        [
            no_op_for_node(
                "gear",
                FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
            ),
            no_op_for_node(
                "gear",
                FoundationalSelectedScopeNoOpCause::UnchangedSourceTruth,
            ),
        ],
        FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect_err("duplicate no-op locus must deny");
    let full_branch_skip = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        FoundationalMergeScope::full_branch(),
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [],
        [],
        [],
        FoundationalSkippedOutOfScopeEvidence::new(1, None),
        1,
    )
    .expect_err("full branch cannot skip out of scope");

    assert_eq!(
        duplicate_no_op,
        FoundationalMergeConstructionDenial::DuplicateSelectedNoOpLocus
    );
    assert_eq!(
        full_branch_skip,
        FoundationalMergeConstructionDenial::FullBranchScopeCannotSkipOutOfScope
    );
}

#[test]
fn scope_evidence_constructor_denies_one_locus_with_multiple_outcomes() {
    let selected_scope =
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope");
    let denial = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_scope,
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("gear")],
        [],
        [no_op_for_node(
            "gear",
            FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
        )],
        FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect_err("one selected locus cannot be admitted and no-op");

    assert_eq!(
        denial,
        FoundationalMergeConstructionDenial::ScopedEvidenceLocusHasMultipleOutcomes
    );
}

#[test]
fn scope_evidence_constructor_denies_selected_loci_without_outcomes() {
    let selected_scope = FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
    ])
    .expect("aspect scope");
    let denial = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_scope,
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [],
        [selected_aspect("gear", "teeth")],
        [],
        FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect_err("every requested selected locus needs an explicit outcome");

    assert_eq!(
        denial,
        FoundationalMergeConstructionDenial::ScopedEvidenceMissingSelectedOutcome
    );
}

#[test]
fn selected_scope_denials_preserve_every_scope_specific_denial_family() {
    let cases = [
        denial_case(
            FoundationalScopedMergeDenialKind::UnknownSelectedNode,
            FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
            FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            FoundationalScopedMergeDenialKind::UnknownSelectedAspect,
            FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
                .expect("aspect scope"),
            FoundationalDeniedScopeLocus::Aspect(selected_aspect("gear", "teeth")),
        ),
        denial_case(
            FoundationalScopedMergeDenialKind::SelectedNodeMissingFromSourceScope,
            FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
            FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            FoundationalScopedMergeDenialKind::SelectedNodeDeletedBeforeAdmission,
            FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
            FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            FoundationalScopedMergeDenialKind::SelectedTargetCorrespondenceAmbiguous,
            FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
                .expect("aspect scope"),
            FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            FoundationalScopedMergeDenialKind::SelectedTargetCorrespondenceRejectedByDeclaration,
            FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
                .expect("aspect scope"),
            FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            FoundationalScopedMergeDenialKind::SelectedNodeNonAdoptable,
            FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
            FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            FoundationalScopedMergeDenialKind::SelectedAspectUnsupportedByNodeOrStrategy,
            FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
                .expect("aspect scope"),
            FoundationalDeniedScopeLocus::Aspect(selected_aspect("gear", "teeth")),
        ),
        denial_case(
            FoundationalScopedMergeDenialKind::ScopeFamilyRejectedByDeclaration,
            FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
                .expect("aspect scope"),
            FoundationalDeniedScopeLocus::ScopeFamily(
                FoundationalMergeScopeFamily::SelectedAspects,
            ),
        ),
    ];

    for expected in cases {
        let actual = scoped_candidate(expected.requested_scope().clone())
            .deny_selected_scope(expected.clone());
        assert_admission_denial(
            actual,
            FoundationalMergeAdmissionDenial::ScopedSelectionDenied(expected),
        );
    }
}

#[test]
fn selected_scope_denial_evidence_denies_wrong_locus_kind_or_out_of_scope_locus() {
    let selected_node_scope =
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope");
    let wrong_locus_kind = FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_node_scope.clone(),
        FoundationalScopedMergeDenialKind::UnknownSelectedAspect,
        FoundationalDeniedScopeLocus::Node(selected_node("gear")),
    )
    .expect_err("aspect denial cannot name a node locus");
    let out_of_scope_locus = FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_node_scope,
        FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        FoundationalDeniedScopeLocus::Node(selected_node("other")),
    )
    .expect_err("denied locus must belong to requested scope");

    assert_eq!(
        wrong_locus_kind,
        FoundationalMergeConstructionDenial::ScopedDenialLocusMismatch
    );
    assert_eq!(
        out_of_scope_locus,
        FoundationalMergeConstructionDenial::ScopedEvidenceOutsideRequestedScope
    );
}

#[test]
fn selected_scope_denial_admission_denies_mismatched_scope_or_branch_basis() {
    let candidate_scope =
        FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope");
    let evidence_scope =
        FoundationalMergeScope::selected_nodes([selected_node("material")]).expect("node scope");
    let scope_mismatch = scoped_denial_evidence(
        FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        evidence_scope,
        FoundationalDeniedScopeLocus::Node(selected_node("material")),
    );
    let source_mismatch = FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/other"),
        branch_id("main"),
        candidate_scope.clone(),
        FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        FoundationalDeniedScopeLocus::Node(selected_node("gear")),
    )
    .expect("source mismatch evidence");
    let target_mismatch = FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/geometry"),
        branch_id("release"),
        candidate_scope.clone(),
        FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        FoundationalDeniedScopeLocus::Node(selected_node("gear")),
    )
    .expect("target mismatch evidence");

    assert_admission_denial(
        scoped_candidate(candidate_scope.clone()).deny_selected_scope(scope_mismatch),
        FoundationalMergeAdmissionDenial::ScopedEvidenceScopeMismatch,
    );
    assert_admission_denial(
        scoped_candidate(candidate_scope.clone()).deny_selected_scope(source_mismatch),
        FoundationalMergeAdmissionDenial::ScopedEvidenceSourceBranchMismatch,
    );
    assert_admission_denial(
        scoped_candidate(candidate_scope).deny_selected_scope(target_mismatch),
        FoundationalMergeAdmissionDenial::ScopedEvidenceTargetBranchMismatch,
    );
}

fn assert_admission_denial<T: Debug>(
    actual: worth_foundational::FoundationalMergeAdmissionOutcome<T>,
    expected: FoundationalMergeAdmissionDenial,
) {
    match actual {
        TransitionOutcome::Denied(denial) => assert_eq!(denial, expected),
        other => panic!("expected scoped evidence admission denial, got {other:?}"),
    }
}

fn denial_case(
    denial_kind: FoundationalScopedMergeDenialKind,
    requested_scope: FoundationalMergeScope,
    denied_locus: FoundationalDeniedScopeLocus,
) -> FoundationalScopedMergeDenialEvidence {
    scoped_denial_evidence(denial_kind, requested_scope, denied_locus)
}

fn scoped_denial_evidence(
    denial_kind: FoundationalScopedMergeDenialKind,
    requested_scope: FoundationalMergeScope,
    denied_locus: FoundationalDeniedScopeLocus,
) -> FoundationalScopedMergeDenialEvidence {
    FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        requested_scope,
        denial_kind,
        denied_locus,
    )
    .expect("scoped denial evidence")
}
