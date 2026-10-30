pub(crate) use std::fmt::Debug;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
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

pub(crate) use crate::certification::ordinary_context::transitions::fixtures::branch::branch_id;
pub(crate) use crate::certification::ordinary_context::transitions::fixtures::merge::merge_summary;
pub(crate) use {
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::no_op_for_node,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::scoped_candidate,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect,
    crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_node,
};

pub(crate) fn assert_admission_denial<T: Debug>(
    actual: worth_foundational::FoundationalMergeAdmissionOutcome<T>,
    expected: FoundationalMergeAdmissionDenial,
) {
    match actual {
        worth_proof::TransitionOutcome::Denied(denial) => assert_eq!(denial, expected),
        other => panic!("expected scoped evidence admission denial, got {other:?}"),
    }
}

pub(crate) fn denial_case(
    denial_kind: FoundationalScopedMergeDenialKind,
    requested_scope: FoundationalMergeScope,
    denied_locus: FoundationalDeniedScopeLocus,
) -> FoundationalScopedMergeDenialEvidence {
    scoped_denial_evidence(denial_kind, requested_scope, denied_locus)
}

pub(crate) fn scoped_denial_evidence(
    denial_kind: FoundationalScopedMergeDenialKind,
    requested_scope: FoundationalMergeScope,
    denied_locus: FoundationalDeniedScopeLocus,
) -> FoundationalScopedMergeDenialEvidence {
    worth_foundational::FoundationalScopedMergeDenialEvidence::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        requested_scope,
        denial_kind,
        denied_locus,
    )
    .expect("scoped denial evidence")
}
