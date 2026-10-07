use super::*;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;

#[test]
fn resource_pressure_remains_typed_across_the_effect_boundary() {
    let staging = transaction_staging(
        RelationalTransactionStagingDenial::OverlayCapacityExhausted {
            maximum_bytes: 64,
            required_bytes: 65,
        },
    );
    assert!(matches!(
        staging,
        RelationalEffectExecutionFailure::Denied {
            kind: EffectExecutionDenialKind::TransactionOverlayBudgetExceeded {
                maximum_bytes: 64,
                required_bytes: 65,
            },
            ..
        }
    ));

    let publication = publication(RelationalPublicationDeferred::CandidateCapacityExhausted {
        maximum_candidates: 7,
    });
    assert!(matches!(
        publication,
        RelationalEffectExecutionFailure::Deferred {
            kind: EffectExecutionDeferredKind::CandidateCapacityExhausted {
                maximum_candidates: 7,
            },
            ..
        }
    ));
}

#[test]
fn interruption_and_retention_admission_remain_typed() {
    assert!(matches!(
        transaction_admission(
            RelationalBranchTransactionAdmissionDenial::RetentionCapacityExhausted
        ),
        RelationalEffectExecutionFailure::Deferred {
            kind: EffectExecutionDeferredKind::TransactionRetentionCapacityExhausted,
            ..
        }
    ));
    assert!(matches!(
        transaction_admission(RelationalBranchTransactionAdmissionDenial::Cancelled),
        RelationalEffectExecutionFailure::ControlStopped {
            kind: crate::effect_lifecycle::EffectExecutionControlStopKind::Cancelled,
            ..
        }
    ));
}

#[test]
fn materialization_staging_denials_preserve_authority_meaning() {
    assert!(matches!(
        transaction_staging(RelationalTransactionStagingDenial::MaterializationAuthorityRequired),
        RelationalEffectExecutionFailure::Denied {
            kind: EffectExecutionDenialKind::TransactionMaterializationAuthorityRequired,
            ..
        }
    ));
    assert!(matches!(
        transaction_staging(RelationalTransactionStagingDenial::MaterializationModeMismatch),
        RelationalEffectExecutionFailure::Denied {
            kind: EffectExecutionDenialKind::TransactionMaterializationModeMismatch,
            ..
        }
    ));
}

#[test]
fn leased_execution_is_typed_at_the_effect_boundary() {
    use worth_query_execution::facade::primary_graph::WorthQueryProviderSessionDenialKind as Kind;
    let observed = transaction_commit(crate::relational_execution_refusal::work_exhausted());
    assert!(matches!(
        observed,
        RelationalEffectExecutionFailure::Denied {
            kind: EffectExecutionDenialKind::RelationalExecutionDenied(Kind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: Some(1),
                policy_ancestor: None,
            }),
            ..
        }
    ));
}
