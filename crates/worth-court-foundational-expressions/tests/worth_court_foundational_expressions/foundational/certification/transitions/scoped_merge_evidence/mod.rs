#[allow(unused_imports)]
use crate::certification::ordinary_context::transitions::scoped_merge_evidence::*;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::CanonicalDigestId,
    worth_foundational::FoundationalAdmittedMergeScopeEvidence,
    worth_foundational::FoundationalMergeScope,
    worth_foundational::FoundationalScopeAdmissionBasis,
    worth_foundational::FoundationalSelectedAspectRequestEntry,
    worth_foundational::FoundationalSelectedScopeLocus,
    worth_foundational::FoundationalSelectedScopeNoOpCause,
    worth_foundational::FoundationalSelectedScopeNoOpEvidence,
    worth_foundational::FoundationalSkippedOutOfScopeEvidence,
};

use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
use crate::certification::ordinary_context::transitions::fixtures::merge::merge_summary;
use {
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::no_op_for_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::scoped_candidate,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect,
};

#[test]
fn custom_scope_evidence_can_carry_partial_admission_no_ops_and_skipped_candidates() {
    let scope = worth_foundational::FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
    ])
    .expect("scope");
    let scope_evidence = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        scope.clone(),
        worth_foundational::FoundationalScopeAdmissionBasis::IdentityCorresponded,
        [],
        [selected_aspect("gear", "teeth")],
        [
            worth_foundational::FoundationalSelectedScopeNoOpEvidence::new(
                worth_foundational::FoundationalSelectedScopeLocus::Aspect(selected_aspect(
                    "gear",
                    "thickness",
                )),
                worth_foundational::FoundationalSelectedScopeNoOpCause::UnchangedSourceTruth,
            ),
        ],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(
            3,
            Some(worth_foundational::CanonicalDigestId::new([8; 32])),
        ),
        merge_summary().conflict_check_width(),
    )
    .expect("scope evidence");

    let verdict =
        match scoped_candidate(scope).admit_as_accepted_with_scope_evidence(scope_evidence) {
            worth_proof::TransitionOutcome::Success(verdict) => verdict,
            other => panic!("expected accepted verdict, got {other:?}"),
        };

    assert_eq!(
        verdict.scope_evidence().admission_basis(),
        worth_foundational::FoundationalScopeAdmissionBasis::IdentityCorresponded
    );
    assert_exact_admitted_aspects(
        verdict.scope_evidence().admitted_aspects(),
        &[("gear", "teeth")],
    );
    assert_exact_no_op_aspects(
        verdict.scope_evidence().selected_no_ops(),
        &[(
            "gear",
            "thickness",
            worth_foundational::FoundationalSelectedScopeNoOpCause::UnchangedSourceTruth,
        )],
    );
    assert_eq!(
        verdict.scope_evidence().skipped().skipped_candidate_count(),
        3
    );
    assert_eq!(
        verdict.scope_evidence().breadth().requested_locus_count(),
        2
    );
    assert_eq!(verdict.scope_evidence().breadth().admitted_locus_count(), 1);
    assert_eq!(verdict.scope_evidence().breadth().no_op_locus_count(), 1);
    assert_eq!(
        verdict.scope_evidence().breadth().skipped_candidate_count(),
        3
    );
}

#[test]
fn scope_evidence_constructor_canonicalizes_admitted_and_no_op_loci() {
    let selected_scope = worth_foundational::FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
        selected_aspect("material", "finish"),
    ])
    .expect("aspect scope");
    let evidence = worth_foundational::FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_scope,
        worth_foundational::FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [],
        [
            selected_aspect("material", "finish"),
            selected_aspect("gear", "teeth"),
        ],
        [no_op_for_aspect(
            "gear",
            "thickness",
            worth_foundational::FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
        )],
        worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(0, None),
        1,
    )
    .expect("scope evidence");

    assert_exact_admitted_aspects(
        evidence.admitted_aspects(),
        &[("gear", "teeth"), ("material", "finish")],
    );
    assert_exact_no_op_aspects(
        evidence.selected_no_ops(),
        &[(
            "gear",
            "thickness",
            worth_foundational::FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
        )],
    );
}

#[test]
fn selected_no_op_and_skipped_scope_evidence_are_distinct_compact_artifacts() {
    let no_op = worth_foundational::FoundationalSelectedScopeNoOpEvidence::new(
        worth_foundational::FoundationalSelectedScopeLocus::Aspect(selected_aspect(
            "gear", "teeth",
        )),
        worth_foundational::FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
    );
    let skipped = worth_foundational::FoundationalSkippedOutOfScopeEvidence::new(
        7,
        Some(worth_foundational::CanonicalDigestId::new([9; 32])),
    );

    assert!(matches!(
        no_op.locus(),
        worth_foundational::FoundationalSelectedScopeLocus::Aspect(entry)
            if entry.node().as_str() == "gear" && entry.aspect().as_str() == "teeth"
    ));
    assert_eq!(
        no_op.cause(),
        worth_foundational::FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth
    );
    assert_eq!(skipped.skipped_candidate_count(), 7);
    assert_eq!(skipped.skipped_digest().expect("digest").bytes(), &[9; 32]);
}
