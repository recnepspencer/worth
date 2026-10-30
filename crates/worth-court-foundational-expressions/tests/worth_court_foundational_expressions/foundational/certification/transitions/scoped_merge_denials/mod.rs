#[allow(unused_imports)]
use crate::certification::ordinary_context::transitions::scoped_merge_denials::*;
use std::fmt::Debug;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::FoundationalAdmittedMergeScopeEvidence,
    worth_foundational::FoundationalDeniedScopeLocus,
    worth_foundational::FoundationalMergeAdmissionDenial,
    worth_foundational::FoundationalMergeConstructionDenial,
    worth_foundational::FoundationalMergeScope, worth_foundational::FoundationalMergeScopeFamily,
    worth_foundational::FoundationalScopeAdmissionBasis,
    worth_foundational::FoundationalScopedMergeDenialEvidence,
    worth_foundational::FoundationalScopedMergeDenialKind,
    worth_foundational::FoundationalSelectedScopeNoOpCause,
    worth_foundational::FoundationalSkippedOutOfScopeEvidence,
};

use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
use crate::certification::ordinary_context::transitions::fixtures::merge::merge_summary;
use {
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::no_op_for_node,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::scoped_candidate,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_node,
};

#[test]
fn custom_scope_evidence_denies_mismatched_candidate_scope_or_branch_basis() {
    let requested_scope =
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
            .expect("node scope");
    let candidate_scope =
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("material")])
            .expect("candidate scope");
    let evidence_scope_mismatch = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        requested_scope,
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("gear")],
        [],
        [],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect("scope evidence");
    let evidence_branch_mismatch = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/other"),
        branch_id("main"),
        candidate_scope.clone(),
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("material")],
        [],
        [],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect("branch evidence");
    let evidence_target_mismatch = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("release"),
        candidate_scope.clone(),
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("material")],
        [],
        [],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect("target branch evidence");

    assert_admission_denial(
        scoped_candidate(candidate_scope.clone())
            .admit_as_accepted_with_scope_evidence(evidence_scope_mismatch),
        worth_foundational::FoundationalMergeAdmissionDenial::ScopedEvidenceScopeMismatch,
    );
    assert_admission_denial(
        scoped_candidate(candidate_scope)
            .admit_as_accepted_with_scope_evidence(evidence_branch_mismatch),
        worth_foundational::FoundationalMergeAdmissionDenial::ScopedEvidenceSourceBranchMismatch,
    );
    assert_admission_denial(
        scoped_candidate(
            worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("material")])
                .expect("candidate scope"),
        )
        .admit_as_accepted_with_scope_evidence(evidence_target_mismatch),
        worth_foundational::FoundationalMergeAdmissionDenial::ScopedEvidenceTargetBranchMismatch,
    );
}

#[test]
fn custom_scope_evidence_denies_conflict_width_mismatch() {
    let scope = worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
        .expect("node scope");
    let evidence = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        scope.clone(),
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("gear")],
        [],
        [],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(0, None),
        merge_summary().conflict_check_width() + 1,
    )
    .expect("scope evidence");

    assert_admission_denial(
        scoped_candidate(scope).admit_as_accepted_with_scope_evidence(evidence),
        worth_foundational::FoundationalMergeAdmissionDenial::ScopedEvidenceConflictWidthMismatch,
    );
}

#[test]
fn scope_evidence_constructor_denies_loci_outside_requested_scope() {
    let requested_scope =
        worth_foundational::FoundationalMergeScope::selected_aspects([selected_aspect(
            "gear", "teeth",
        )])
        .expect("aspect scope");
    let denial = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        requested_scope,
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [],
        [selected_aspect("gear", "thickness")],
        [],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect_err("outside requested scope must deny");

    assert_eq!(
        denial,
        worth_foundational::FoundationalMergeConstructionDenial::ScopedEvidenceOutsideRequestedScope
    );
}

#[test]
fn scope_evidence_constructor_denies_duplicate_no_ops_and_full_branch_skips() {
    let selected_scope =
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
            .expect("node scope");
    let duplicate_no_op = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_scope,
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [],
        [],
        [
            no_op_for_node(
                "gear",
                worth_foundational::FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
            ),
            no_op_for_node(
                "gear",
                worth_foundational::FoundationalSelectedScopeNoOpCause::UnchangedSourceTruth,
            ),
        ],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect_err("duplicate no-op locus must deny");
    let full_branch_skip = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        worth_foundational::FoundationalMergeScope::full_branch(),
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [],
        [],
        [],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(1, None),
        1,
    )
    .expect_err("full branch cannot skip out of scope");

    assert_eq!(
        duplicate_no_op,
        worth_foundational::FoundationalMergeConstructionDenial::DuplicateSelectedNoOpLocus
    );
    assert_eq!(
        full_branch_skip,
        worth_foundational::FoundationalMergeConstructionDenial::FullBranchScopeCannotSkipOutOfScope
    );
}

#[test]
fn scope_evidence_constructor_denies_one_locus_with_multiple_outcomes() {
    let selected_scope =
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
            .expect("node scope");
    let denial = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_scope,
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [selected_node("gear")],
        [],
        [no_op_for_node(
            "gear",
            worth_foundational::FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
        )],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect_err("one selected locus cannot be admitted and no-op");

    assert_eq!(
        denial,
        worth_foundational::FoundationalMergeConstructionDenial::ScopedEvidenceLocusHasMultipleOutcomes
    );
}

#[test]
fn scope_evidence_constructor_denies_selected_loci_without_outcomes() {
    let selected_scope = worth_foundational::FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
    ])
    .expect("aspect scope");
    let denial = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_scope,
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [],
        [selected_aspect("gear", "teeth")],
        [],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect_err("every requested selected locus needs an explicit outcome");

    assert_eq!(
        denial,
        worth_foundational::FoundationalMergeConstructionDenial::ScopedEvidenceMissingSelectedOutcome
    );
}

#[test]
fn selected_scope_denials_preserve_every_scope_specific_denial_family() {
    let cases = [
        denial_case(
            worth_foundational::FoundationalScopedMergeDenialKind::UnknownSelectedNode,
            worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
            worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            worth_foundational::FoundationalScopedMergeDenialKind::UnknownSelectedAspect,
            worth_foundational::FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
                .expect("aspect scope"),
            worth_foundational::FoundationalDeniedScopeLocus::Aspect(selected_aspect("gear", "teeth")),
        ),
        denial_case(
            worth_foundational::FoundationalScopedMergeDenialKind::SelectedNodeMissingFromSourceScope,
            worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
            worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            worth_foundational::FoundationalScopedMergeDenialKind::SelectedNodeDeletedBeforeAdmission,
            worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
            worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            worth_foundational::FoundationalScopedMergeDenialKind::SelectedTargetCorrespondenceAmbiguous,
            worth_foundational::FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
                .expect("aspect scope"),
            worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            worth_foundational::FoundationalScopedMergeDenialKind::SelectedTargetCorrespondenceRejectedByDeclaration,
            worth_foundational::FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
                .expect("aspect scope"),
            worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            worth_foundational::FoundationalScopedMergeDenialKind::SelectedNodeNonAdoptable,
            worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")]).expect("node scope"),
            worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
        ),
        denial_case(
            worth_foundational::FoundationalScopedMergeDenialKind::SelectedAspectUnsupportedByNodeOrStrategy,
            worth_foundational::FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
                .expect("aspect scope"),
            worth_foundational::FoundationalDeniedScopeLocus::Aspect(selected_aspect("gear", "teeth")),
        ),
        denial_case(
            worth_foundational::FoundationalScopedMergeDenialKind::ScopeFamilyRejectedByDeclaration,
            worth_foundational::FoundationalMergeScope::selected_aspects([selected_aspect("gear", "teeth")])
                .expect("aspect scope"),
            worth_foundational::FoundationalDeniedScopeLocus::ScopeFamily(
                worth_foundational::FoundationalMergeScopeFamily::SelectedAspects,
            ),
        ),
    ];

    for expected in cases {
        let actual = scoped_candidate(expected.requested_scope().clone())
            .deny_selected_scope(expected.clone());
        assert_admission_denial(
            actual,
            worth_foundational::FoundationalMergeAdmissionDenial::ScopedSelectionDenied(expected),
        );
    }
}

#[test]
fn selected_scope_denial_evidence_denies_wrong_locus_kind_or_out_of_scope_locus() {
    let selected_node_scope =
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
            .expect("node scope");
    let wrong_locus_kind = worth_foundational::FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_node_scope.clone(),
        worth_foundational::FoundationalScopedMergeDenialKind::UnknownSelectedAspect,
        worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
    )
    .expect_err("aspect denial cannot name a node locus");
    let out_of_scope_locus = worth_foundational::FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_node_scope,
        worth_foundational::FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("other")),
    )
    .expect_err("denied locus must belong to requested scope");

    assert_eq!(
        wrong_locus_kind,
        worth_foundational::FoundationalMergeConstructionDenial::ScopedDenialLocusMismatch
    );
    assert_eq!(
        out_of_scope_locus,
        worth_foundational::FoundationalMergeConstructionDenial::ScopedEvidenceOutsideRequestedScope
    );
}

#[test]
fn selected_scope_denial_admission_denies_mismatched_scope_or_branch_basis() {
    let candidate_scope =
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("gear")])
            .expect("node scope");
    let evidence_scope =
        worth_foundational::FoundationalMergeScope::selected_nodes([selected_node("material")])
            .expect("node scope");
    let scope_mismatch = scoped_denial_evidence(
        worth_foundational::FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        evidence_scope,
        worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("material")),
    );
    let source_mismatch = worth_foundational::FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/other"),
        branch_id("main"),
        candidate_scope.clone(),
        worth_foundational::FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
    )
    .expect("source mismatch evidence");
    let target_mismatch = worth_foundational::FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/geometry"),
        branch_id("release"),
        candidate_scope.clone(),
        worth_foundational::FoundationalScopedMergeDenialKind::UnknownSelectedNode,
        worth_foundational::FoundationalDeniedScopeLocus::Node(selected_node("gear")),
    )
    .expect("target mismatch evidence");

    assert_admission_denial(
        scoped_candidate(candidate_scope.clone()).deny_selected_scope(scope_mismatch),
        worth_foundational::FoundationalMergeAdmissionDenial::ScopedEvidenceScopeMismatch,
    );
    assert_admission_denial(
        scoped_candidate(candidate_scope.clone()).deny_selected_scope(source_mismatch),
        worth_foundational::FoundationalMergeAdmissionDenial::ScopedEvidenceSourceBranchMismatch,
    );
    assert_admission_denial(
        scoped_candidate(candidate_scope).deny_selected_scope(target_mismatch),
        worth_foundational::FoundationalMergeAdmissionDenial::ScopedEvidenceTargetBranchMismatch,
    );
}
