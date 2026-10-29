use worth_foundational::{
    CanonicalDigestId, FoundationalAdmittedMergeScopeEvidence, FoundationalMergeScope,
    FoundationalScopeAdmissionBasis, FoundationalSelectedAspectRequestEntry,
    FoundationalSelectedScopeLocus, FoundationalSelectedScopeNoOpCause,
    FoundationalSelectedScopeNoOpEvidence, FoundationalSkippedOutOfScopeEvidence,
};
use worth_proof::TransitionOutcome;

use super::fixtures::branch::branch_id;
use super::fixtures::merge::merge_summary;
use super::fixtures::scoped_merge::{no_op_for_aspect, scoped_candidate, selected_aspect};

#[test]
fn custom_scope_evidence_can_carry_partial_admission_no_ops_and_skipped_candidates() {
    let scope = FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
    ])
    .expect("scope");
    let scope_evidence = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        scope.clone(),
        FoundationalScopeAdmissionBasis::IdentityCorresponded,
        [],
        [selected_aspect("gear", "teeth")],
        [FoundationalSelectedScopeNoOpEvidence::new(
            FoundationalSelectedScopeLocus::Aspect(selected_aspect("gear", "thickness")),
            FoundationalSelectedScopeNoOpCause::UnchangedSourceTruth,
        )],
        FoundationalSkippedOutOfScopeEvidence::new(3, Some(CanonicalDigestId::new([8; 32]))),
        merge_summary().conflict_check_width(),
    )
    .expect("scope evidence");

    let verdict =
        match scoped_candidate(scope).admit_as_accepted_with_scope_evidence(scope_evidence) {
            TransitionOutcome::Success(verdict) => verdict,
            other => panic!("expected accepted verdict, got {other:?}"),
        };

    assert_eq!(
        verdict.scope_evidence().admission_basis(),
        FoundationalScopeAdmissionBasis::IdentityCorresponded
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
            FoundationalSelectedScopeNoOpCause::UnchangedSourceTruth,
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
    let selected_scope = FoundationalMergeScope::selected_aspects([
        selected_aspect("gear", "teeth"),
        selected_aspect("gear", "thickness"),
        selected_aspect("material", "finish"),
    ])
    .expect("aspect scope");
    let evidence = FoundationalAdmittedMergeScopeEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        selected_scope,
        FoundationalScopeAdmissionBasis::DirectSourceIdentity,
        [],
        [
            selected_aspect("material", "finish"),
            selected_aspect("gear", "teeth"),
        ],
        [no_op_for_aspect(
            "gear",
            "thickness",
            FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
        )],
        FoundationalSkippedOutOfScopeEvidence::new(0, None),
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
            FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
        )],
    );
}

#[test]
fn selected_no_op_and_skipped_scope_evidence_are_distinct_compact_artifacts() {
    let no_op = FoundationalSelectedScopeNoOpEvidence::new(
        FoundationalSelectedScopeLocus::Aspect(selected_aspect("gear", "teeth")),
        FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth,
    );
    let skipped =
        FoundationalSkippedOutOfScopeEvidence::new(7, Some(CanonicalDigestId::new([9; 32])));

    assert!(matches!(
        no_op.locus(),
        FoundationalSelectedScopeLocus::Aspect(entry)
            if entry.node().as_str() == "gear" && entry.aspect().as_str() == "teeth"
    ));
    assert_eq!(
        no_op.cause(),
        FoundationalSelectedScopeNoOpCause::EquivalentTargetTruth
    );
    assert_eq!(skipped.skipped_candidate_count(), 7);
    assert_eq!(skipped.skipped_digest().expect("digest").bytes(), &[9; 32]);
}

fn assert_exact_admitted_aspects(
    actual: &[FoundationalSelectedAspectRequestEntry],
    expected: &[(&str, &str)],
) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "admitted aspect width drifted"
    );
    for (actual_entry, (expected_node, expected_aspect)) in actual.iter().zip(expected.iter()) {
        assert_eq!(actual_entry.node().as_str(), *expected_node);
        assert_eq!(actual_entry.aspect().as_str(), *expected_aspect);
    }
}

fn assert_exact_no_op_aspects(
    actual: &[FoundationalSelectedScopeNoOpEvidence],
    expected: &[(&str, &str, FoundationalSelectedScopeNoOpCause)],
) {
    assert_eq!(actual.len(), expected.len(), "no-op aspect width drifted");
    for (actual_entry, (expected_node, expected_aspect, expected_cause)) in
        actual.iter().zip(expected.iter())
    {
        assert_eq!(actual_entry.cause(), *expected_cause);
        assert!(matches!(
            actual_entry.locus(),
            FoundationalSelectedScopeLocus::Aspect(entry)
                if entry.node().as_str() == *expected_node
                    && entry.aspect().as_str() == *expected_aspect
        ));
    }
}
