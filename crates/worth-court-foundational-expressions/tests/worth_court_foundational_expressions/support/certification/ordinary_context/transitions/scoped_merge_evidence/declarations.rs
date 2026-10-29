pub(crate) use {worth_foundational::CanonicalDigestId, worth_foundational::FoundationalAdmittedMergeScopeEvidence, worth_foundational::FoundationalMergeScope, worth_foundational::FoundationalScopeAdmissionBasis, worth_foundational::FoundationalSelectedAspectRequestEntry, worth_foundational::FoundationalSelectedScopeLocus, worth_foundational::FoundationalSelectedScopeNoOpCause, worth_foundational::FoundationalSelectedScopeNoOpEvidence, worth_foundational::FoundationalSkippedOutOfScopeEvidence};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {crate::certification::ordinary_context::transitions::fixtures::branch::branch_id};
pub(crate) use {crate::certification::ordinary_context::transitions::fixtures::merge::merge_summary};
pub(crate) use {crate::certification::ordinary_context::transitions::fixtures::scoped_merge::no_op_for_aspect, crate::certification::ordinary_context::transitions::fixtures::scoped_merge::scoped_candidate, crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect};

pub(crate) fn assert_exact_admitted_aspects(
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

pub(crate) fn assert_exact_no_op_aspects(
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
            worth_foundational::FoundationalSelectedScopeLocus::Aspect(entry)
                if entry.node().as_str() == *expected_node
                    && entry.aspect().as_str() == *expected_aspect
        ));
    }
}