pub(crate) use {std::fmt::Debug};
pub(crate) use {worth_foundational::FoundationalMergeAdmissionDenial, worth_foundational::FoundationalMergeConstructionDenial, worth_foundational::FoundationalMergeScope, worth_foundational::FoundationalScopedMergeUnavailableOutcomeCategory, worth_foundational::FoundationalScopedMergeUnavailablePosture, worth_foundational::FoundationalScopedMergeUnavailableReason};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) use {crate::certification::ordinary_context::transitions::fixtures::branch::branch_id};
pub(crate) use {crate::certification::ordinary_context::transitions::fixtures::scoped_merge::scoped_candidate, crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_aspect, crate::certification::ordinary_context::transitions::fixtures::scoped_merge::selected_node};

pub(crate) fn assert_deferred_unavailable(
    scope: FoundationalMergeScope,
    reason: FoundationalScopedMergeUnavailableReason,
) {
    match scoped_candidate(scope.clone()).scope_unavailable(scoped_unavailable(scope, reason)) {
        worth_proof::TransitionOutcome::Deferred(deferred) => assert_posture(
            deferred.scope_unavailable_posture(),
            reason,
            worth_foundational::FoundationalScopedMergeUnavailableOutcomeCategory::Deferred,
        ),
        other => panic!("expected deferred unavailable posture, got {other:?}"),
    }
}

pub(crate) fn assert_stale_unavailable(
    scope: FoundationalMergeScope,
    reason: FoundationalScopedMergeUnavailableReason,
) {
    match scoped_candidate(scope.clone()).scope_unavailable(scoped_unavailable(scope, reason)) {
        worth_proof::TransitionOutcome::Stale(stale) => assert_posture(
            stale.scope_unavailable_posture(),
            reason,
            worth_foundational::FoundationalScopedMergeUnavailableOutcomeCategory::Stale,
        ),
        other => panic!("expected stale unavailable posture, got {other:?}"),
    }
}

pub(crate) fn assert_rebind_unavailable(
    scope: FoundationalMergeScope,
    reason: FoundationalScopedMergeUnavailableReason,
) {
    match scoped_candidate(scope.clone()).scope_unavailable(scoped_unavailable(scope, reason)) {
        worth_proof::TransitionOutcome::RebindRequired(rebind) => assert_posture(
            rebind.scope_unavailable_posture(),
            reason,
            worth_foundational::FoundationalScopedMergeUnavailableOutcomeCategory::RebindRequired,
        ),
        other => panic!("expected rebind unavailable posture, got {other:?}"),
    }
}

pub(crate) fn assert_failed_unavailable(
    scope: FoundationalMergeScope,
    reason: FoundationalScopedMergeUnavailableReason,
) {
    match scoped_candidate(scope.clone()).scope_unavailable(scoped_unavailable(scope, reason)) {
        worth_proof::TransitionOutcome::Failed(failure) => assert_posture(
            failure.scope_unavailable_posture(),
            reason,
            worth_foundational::FoundationalScopedMergeUnavailableOutcomeCategory::Failed,
        ),
        other => panic!("expected failed unavailable posture, got {other:?}"),
    }
}

pub(crate) fn assert_posture(
    posture: Option<&FoundationalScopedMergeUnavailablePosture>,
    reason: FoundationalScopedMergeUnavailableReason,
    outcome_category: FoundationalScopedMergeUnavailableOutcomeCategory,
) {
    let posture = posture.expect("scope unavailable posture");
    assert_eq!(posture.source_branch(), &branch_id("feature/geometry"));
    assert_eq!(posture.target_branch(), &branch_id("main"));
    assert_eq!(posture.reason(), reason);
    assert_eq!(posture.outcome_category(), outcome_category);
}

pub(crate) fn assert_admission_denial<T: Debug>(
    actual: worth_foundational::FoundationalMergeAdmissionOutcome<T>,
    expected: FoundationalMergeAdmissionDenial,
) {
    match actual {
        worth_proof::TransitionOutcome::Denied(denial) => assert_eq!(denial, expected),
        other => panic!("expected scoped unavailable admission denial, got {other:?}"),
    }
}

pub(crate) fn scoped_unavailable(
    requested_scope: FoundationalMergeScope,
    reason: FoundationalScopedMergeUnavailableReason,
) -> FoundationalScopedMergeUnavailablePosture {
    worth_foundational::FoundationalScopedMergeUnavailablePosture::new(
        branch_id("feature/geometry"),
        branch_id("main"),
        requested_scope,
        reason,
    )
    .expect("scoped unavailable posture")
}