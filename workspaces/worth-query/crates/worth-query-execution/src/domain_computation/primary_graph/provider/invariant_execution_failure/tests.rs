use super::{map_custom_invariant_failure, map_invariant_failure, map_transaction_staging_failure};
use crate::domain_computation::WorthQueryCustomInvariantDenial;
use worth_relational::facade::transactions::{
    CustomInvariantFailureIdentity, CustomInvariantFailurePhase, CustomInvariantRuleId,
    CustomInvariantSemanticIdentity, CustomInvariantSemanticVersion, InvariantViolationFields,
    ResultCustomInvariantFailureKind,
};

#[test]
fn custom_semantic_violation_preserves_owner_identity() {
    let identity = semantic_identity();
    let failure = map_custom_invariant_failure(
        InvariantViolationFields::CustomInvariantViolation {
            identity: identity.clone(),
        },
        "custom semantic violation".to_owned(),
    )
    .unwrap();

    assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryInvariantExecutionDenialKind::CustomInvariantDenied
    );
    assert_eq!(
        failure.custom_invariant_denial(),
        Some(&WorthQueryCustomInvariantDenial::Violation { identity })
    );
    let denial = crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::invariant_execution_denied(
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
            failure.clone(),
        );
    assert_eq!(
        denial.custom_invariant_denial(),
        failure.custom_invariant_denial()
    );
    assert_eq!(denial.invariant_execution_failure(), Some(&failure));
}

#[test]
fn custom_operational_failure_preserves_identity_phase_and_kind() {
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        let identity = CustomInvariantFailureIdentity::new(semantic_identity());
        let failure = map_custom_invariant_failure(
            InvariantViolationFields::CustomInvariantFailure {
                identity: identity.clone(),
                phase: CustomInvariantFailurePhase::Execution,
                failure: ResultCustomInvariantFailureKind::Panic,
                detail: "hostile panic".to_owned(),
            },
            "custom invariant execution failed".to_owned(),
        )
        .unwrap();

        assert_eq!(
            failure.kind(),
            crate::domain_computation::WorthQueryInvariantExecutionDenialKind::CustomInvariantDenied
        );
        assert_eq!(
            failure.custom_invariant_denial(),
            Some(&WorthQueryCustomInvariantDenial::Failure {
                identity,
                phase: CustomInvariantFailurePhase::Execution,
                failure: ResultCustomInvariantFailureKind::Panic,
            })
        );
    });
}

#[test]
fn built_in_invariant_preserves_relational_detail() {
    let failure = map_invariant_failure(
        InvariantViolationFields::None,
        "relation endpoint deletion leaves an incident edge".to_owned(),
    );

    assert_eq!(
        failure.kind(),
        crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderRejected
    );
    assert_eq!(
        failure.detail(),
        "relation endpoint deletion leaves an incident edge"
    );
}

#[test]
fn invariant_materialization_refusal_is_not_exhaustion() {
    for (owner_denial, expected_detail) in [
            (
                worth_relational::facade::mvcc::RelationalTransactionStagingDenial::MaterializationAuthorityRequired,
                "Relational invariant transaction requires materialization authority",
            ),
            (
                worth_relational::facade::mvcc::RelationalTransactionStagingDenial::MaterializationModeMismatch,
                "Relational invariant transaction materialization mode does not match its intents",
            ),
        ] {
            let failure = map_transaction_staging_failure(owner_denial);
            let denial = crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::invariant_execution_denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
                failure,
            );
            let retained = denial.invariant_execution_failure().unwrap();
            assert_eq!(retained.kind(), crate::domain_computation::WorthQueryInvariantExecutionDenialKind::ProviderRejected);
            assert_eq!(retained.posture(), crate::domain_computation::WorthQueryInvariantExecutionFailurePosture::Denied);
            assert_eq!(retained.detail(), expected_detail);
        }
}

fn semantic_identity() -> CustomInvariantSemanticIdentity {
    CustomInvariantSemanticIdentity {
        rule_id: CustomInvariantRuleId::new("query.custom-invariant"),
        semantic_version: CustomInvariantSemanticVersion::new(3, 7),
    }
}
